//! `waitUntil` 底层原语。
//!
//! 这个模块提供一个很小的宿主生命周期延长机制：上层宿主操作已经可以产出
//! 结果，但仍希望 runtime 跟踪一组后台异步工作，直到这些工作完成、超时，
//! 或者宿主显式关闭登记入口。
//!
//! 典型用途：
//! - HTTP handler 返回 `Response` 后继续记录日志、写审计、刷新指标。
//! - 未来 Worker / Event API 中实现 `event.waitUntil(promise)`。
//! - 测试运行器或 runtime shutdown 等待后台工作 drain。
//!
//! 设计边界：
//! - `WaitUntil` 只跟踪 pending 计数和 drain，不负责执行任务。
//! - `WaitUntil` 不处理 JS `Promise` rejection；上层 adapter 应自行转发到
//!   `reportError`、server error event 或调用方错误处理。
//! - `WaitUntil` 不是 cancellation token，也不是 async context storage。
//!
//! 约束：
//! - `WaitUntil` 是进程内内存原语，不提供跨进程持久化或分布式任务能力。
//! - `WaitUntil` 不绑定具体线程；guard 可以在不同 tokio task / OS thread 中
//!   完成，以适配多线程 async runtime。
//! - `WaitUntil` 不知道 JS runtime、HTTP request 或 Worker event 的所有权。
//!   这些绑定关系必须由上层 adapter 校验，例如 `waitUntil(request, promise)`
//!   只能接受当前 server 创建的 request。

use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
    time::Duration,
};

use tokio::sync::Notify;

#[derive(Clone, Debug)]
pub struct WaitUntil {
    inner: Arc<WaitUntilInner>,
}

/// runtime 级 `WaitUntil` scope 注册表。
///
/// `UntilRegistry` 本身不代表一个具体生命周期，而是负责监督多个
/// `WaitUntil` scope。每个 HTTP request、Worker event 或显式
/// `new WaitUntil({ managed: true })` 都应该创建自己的 scope，并把它登记
/// 到 registry 中。
///
/// 注意：
/// - registry 使用 `Weak` 持有 scope，不阻止 scope 被释放。
/// - registry 的 `drain()` 会等待当前和 drain 过程中新增的 live scope。
/// - registry 的 `close()` 会关闭当前 live scope，并让后续登记的 scope
///   立即进入 closed 状态。
#[derive(Clone, Debug)]
pub struct UntilRegistry {
    inner: Arc<UntilRegistryInner>,
}

#[derive(Debug)]
struct WaitUntilInner {
    pending: AtomicUsize,
    closed: AtomicBool,
    notify: Notify,
}

#[derive(Debug)]
struct UntilRegistryInner {
    scopes: Mutex<Vec<Weak<WaitUntilInner>>>,
    closed: AtomicBool,
}

#[derive(Debug)]
pub struct WaitUntilGuard {
    inner: Arc<WaitUntilInner>,
    active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitUntilError {
    Closed,
    Timeout,
}

impl WaitUntil {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(WaitUntilInner {
                pending: AtomicUsize::new(0),
                closed: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        }
    }

    pub fn begin(&self) -> Result<WaitUntilGuard, WaitUntilError> {
        if self.inner.closed.load(Ordering::SeqCst) {
            return Err(WaitUntilError::Closed);
        }

        self.inner.pending.fetch_add(1, Ordering::SeqCst);
        if self.inner.closed.load(Ordering::SeqCst) {
            self.inner.pending.fetch_sub(1, Ordering::SeqCst);
            self.inner.notify.notify_waiters();
            return Err(WaitUntilError::Closed);
        }

        Ok(WaitUntilGuard {
            inner: self.inner.clone(),
            active: true,
        })
    }

    pub fn pending_count(&self) -> usize {
        self.inner.pending.load(Ordering::SeqCst)
    }

    pub fn is_idle(&self) -> bool {
        self.pending_count() == 0
    }

    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::SeqCst)
    }

    pub fn close(&self) {
        self.inner.closed.store(true, Ordering::SeqCst);
        self.inner.notify.notify_waiters();
    }

    pub async fn drain(&self, timeout: Duration) -> Result<(), WaitUntilError> {
        if self.is_idle() {
            return Ok(());
        }

        let wait = self.drain_until_idle();

        tokio::time::timeout(timeout, wait)
            .await
            .map_err(|_| WaitUntilError::Timeout)
    }

    async fn drain_until_idle(&self) {
        while !self.is_idle() {
            self.inner.notify.notified().await;
        }
    }
}

impl Default for WaitUntil {
    fn default() -> Self {
        Self::new()
    }
}

impl UntilRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(UntilRegistryInner {
                scopes: Mutex::new(Vec::new()),
                closed: AtomicBool::new(false),
            }),
        }
    }

    pub fn create_scope(&self) -> WaitUntil {
        let scope = WaitUntil::new();
        self.manage(&scope);
        scope
    }

    pub fn manage(&self, scope: &WaitUntil) -> bool {
        if self.is_closed() {
            scope.close();
            return false;
        }

        let mut scopes = self
            .inner
            .scopes
            .lock()
            .expect("UntilRegistry mutex poisoned");
        scopes.push(Arc::downgrade(&scope.inner));
        true
    }

    pub fn pending_count(&self) -> usize {
        self.live_scopes()
            .iter()
            .map(WaitUntil::pending_count)
            .sum()
    }

    pub fn scope_count(&self) -> usize {
        self.live_scopes().len()
    }

    pub fn is_idle(&self) -> bool {
        self.pending_count() == 0
    }

    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::SeqCst)
    }

    pub fn close(&self) {
        self.inner.closed.store(true, Ordering::SeqCst);
        for scope in self.live_scopes() {
            scope.close();
        }
    }

    pub async fn drain(&self, timeout: Duration) -> Result<(), WaitUntilError> {
        if self.is_idle() {
            return Ok(());
        }

        let wait = async {
            loop {
                let scopes = self.live_scopes();
                if scopes.iter().all(WaitUntil::is_idle) {
                    return;
                }

                for scope in scopes {
                    scope.drain_until_idle().await;
                }
            }
        };

        tokio::time::timeout(timeout, wait)
            .await
            .map_err(|_| WaitUntilError::Timeout)
    }

    fn live_scopes(&self) -> Vec<WaitUntil> {
        let mut scopes = self
            .inner
            .scopes
            .lock()
            .expect("UntilRegistry mutex poisoned");
        let mut live = Vec::with_capacity(scopes.len());

        scopes.retain(|scope| {
            if let Some(inner) = scope.upgrade() {
                live.push(WaitUntil { inner });
                true
            } else {
                false
            }
        });

        live
    }
}

impl Default for UntilRegistry {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for UntilRegistry {
    type Changed<'to> = UntilRegistry;
}

impl<'js> rquickjs::class::Trace<'js> for UntilRegistry {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl WaitUntilGuard {
    pub fn finish(mut self) {
        self.finish_inner();
    }

    fn finish_inner(&mut self) {
        if !self.active {
            return;
        }

        self.active = false;
        self.inner.pending.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |count| Some(count.saturating_sub(1)),
        ).ok();
        self.inner.notify.notify_waiters();
    }
}

impl Drop for WaitUntilGuard {
    fn drop(&mut self) {
        self.finish_inner();
    }
}

const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<UntilRegistry>();
    assert_send_sync::<WaitUntil>();
    assert_send_sync::<WaitUntilGuard>();
};

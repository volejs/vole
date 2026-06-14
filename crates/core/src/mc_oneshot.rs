// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//! 可多订阅的一次性通知通道。
//!
//! `mc_oneshot` 类似 oneshot channel，但 sender 可以派生多个 receiver，
//! 所有 receiver 都会在同一个值发送后被唤醒。它适合 JS/Rust 异步桥接：
//! Rust 侧某个事件只发生一次，但多个 JS/Rust 持有者都需要等待它。
//!
//! 与 `tokio::sync::oneshot` 的差异：
//! - 支持多个 receiver。
//! - `Sender<Value<'js>>` 实现了 rquickjs [`Trace`]，因此如果发送的值是 JS
//!   value，GC 可以正确追踪它。
//! - 发送后值会保存在共享状态中，后续订阅者也能读取同一个值。
//!
//! 注意事项：
//! - `send` 只生效一次，后续发送会被忽略。
//! - `T` 必须实现 `Clone`，因为每个 receiver 都会拿到一份值。
//! - 该模块只负责同步原语，不负责 Promise resolve/reject 或 runtime 错误分发。

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use rquickjs::{
    class::{Trace, Tracer},
    Value,
};
use std::ops::Deref;
use tokio::sync::Notify;

#[derive(Debug)]
/// channel 内部共享状态。
///
/// `is_sent` 控制一次性发送语义，`value` 保存发送后的值，`notify` 唤醒所有
/// 等待中的 receiver。
pub struct Shared<T> {
    is_sent: AtomicBool,
    value: RwLock<Option<T>>,
    notify: Notify,
}

#[derive(Clone, Debug)]
/// 一次性发送端。
///
/// 克隆 sender 不会允许多次发送；所有 sender 共享同一个 `is_sent` 标志。
pub struct Sender<T: Clone>(Arc<Shared<T>>);

impl<T: Clone> Deref for Sender<T> {
    type Target = Arc<Shared<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'js> Trace<'js> for Sender<Value<'js>> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        if let Ok(v) = self.value.read() {
            if let Some(v) = v.as_ref() {
                tracer.mark(v)
            }
        }
    }
}

impl<T: Clone> Sender<T> {
    /// 发送最终值并唤醒所有等待者。
    ///
    /// 如果已经发送过值，本次调用会被忽略。
    pub fn send(&self, value: T) {
        if !self.is_sent.load(Ordering::Relaxed) {
            self.value.write().unwrap().replace(value);
            self.is_sent.store(true, Ordering::Release);
            self.notify.notify_waiters();
        }
    }

    /// 创建新的订阅 receiver。
    ///
    /// 即使值已经发送，新的 receiver 仍然可以通过 `recv` 读取已保存的值。
    pub fn subscribe(&self) -> Receiver<T> {
        Receiver(Arc::clone(&self.0))
    }
}

#[derive(Clone, Debug)]
/// 一次性接收端。
pub struct Receiver<T: Clone>(Arc<Shared<T>>);

impl<T: Clone> Deref for Receiver<T> {
    type Target = Arc<Shared<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> Receiver<T> {
    /// 等待并返回发送值。
    ///
    /// 如果值已经发送，会立即返回；否则等待 sender 唤醒。
    pub async fn recv(&self) -> T {
        if !self.is_sent.load(Ordering::Acquire) {
            self.notify.notified().await;
        }
        self.value.read().unwrap().clone().unwrap()
    }
}

/// 创建一组 sender/receiver。
pub fn channel<T: Clone>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        is_sent: AtomicBool::new(false),
        value: RwLock::new(None),
        notify: Notify::new(),
    });

    (Sender(Arc::clone(&shared)), Receiver(shared))
}

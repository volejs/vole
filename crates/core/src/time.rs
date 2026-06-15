//! 时间源与高精度时间 helper。
//!
//! Web API 中有两类常见时间：
//! - `timeOrigin`：当前 JS realm 的起始 Unix 时间，单位毫秒。
//! - `DOMHighResTimeStamp`：相对 `timeOrigin` 的高精度单调时间，单位毫秒。
//!
//! 本模块只提供底层时间源，不维护 `performance.mark()` 这类 API 状态。
//! 具体 PerformanceEntry、Event timeStamp 等业务状态应保留在对应 binding crate。

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use rquickjs::{
    Ctx, JsLifetime, Result,
    class::{Trace, Tracer},
    runtime::UserDataGuard,
};

use crate::result::ResultExt;

/// 每个 JS context 的时间起点。
///
/// `instant` 用于单调时间，不受系统时间回拨影响。
/// `unix_time_origin_ms` 用于暴露 `performance.timeOrigin`。
#[derive(Clone)]
pub struct TimeOrigin {
    instant: Instant,
    unix_time_origin_ms: f64,
}

unsafe impl<'js> JsLifetime<'js> for TimeOrigin {
    type Changed<'to> = TimeOrigin;
}

impl<'js> Trace<'js> for TimeOrigin {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl TimeOrigin {
    /// 初始化当前 context 的时间起点。
    ///
    /// 多次调用是幂等的；首次调用决定该 context 后续的 `timeOrigin`。
    pub fn init(ctx: &Ctx<'_>) -> Result<()> {
        if ctx.userdata::<Self>().is_none() {
            ctx.store_userdata(Self::new())?;
        }
        Ok(())
    }

    /// 获取当前 context 的时间起点。
    pub fn get<'a>(ctx: &'a Ctx<'_>) -> Result<UserDataGuard<'a, Self>> {
        ctx.userdata::<Self>()
            .or_throw_msg(ctx, "TimeOrigin is not initialized")
    }

    /// 获取或初始化当前 context 的时间起点。
    pub fn get_or_init<'a>(ctx: &'a Ctx<'_>) -> Result<UserDataGuard<'a, Self>> {
        Self::init(ctx)?;
        Self::get(ctx)
    }

    fn new() -> Self {
        Self {
            instant: Instant::now(),
            unix_time_origin_ms: unix_timestamp_ms(),
        }
    }

    /// `performance.timeOrigin`，单位毫秒。
    pub fn time_origin(&self) -> f64 {
        self.unix_time_origin_ms
    }

    /// 相对 `timeOrigin` 的单调时间，单位毫秒。
    pub fn now(&self) -> f64 {
        self.instant.elapsed().as_secs_f64() * 1000.0
    }
}

/// 当前 Unix 时间，单位毫秒。
pub fn unix_timestamp_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

/// 当前 context 的 `performance.now()` 值。
pub fn now(ctx: &Ctx<'_>) -> Result<f64> {
    Ok(TimeOrigin::get_or_init(ctx)?.now())
}

/// 当前 context 的 `performance.timeOrigin` 值。
pub fn time_origin(ctx: &Ctx<'_>) -> Result<f64> {
    Ok(TimeOrigin::get_or_init(ctx)?.time_origin())
}

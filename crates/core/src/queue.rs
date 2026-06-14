//! Web Streams 风格的 queue-with-sizes 抽象。
//!
//! Streams 规范中的 readable/writable controller 都维护一个
//! “value with size” 队列，用于计算 desiredSize 和 backpressure。
//! 这个结构本身不依赖 Streams 状态机，因此放在 core 中复用。
//!
//! 注意事项：
//! - [`QueueWithSizes`] 只负责队列和 size 累加，不负责 close/error/pull/write
//!   等状态转换。
//! - size 必须是有限、非 NaN、非负数；否则按规范抛 `RangeError`。
//! - 浮点累减可能产生极小负数，因此 dequeue 后会把负的 total size 修正为 0。

use std::collections::VecDeque;

use rquickjs::{class::Trace, Ctx, Exception, FromJs, JsLifetime, Result, Value};

/// size algorithm 的返回值。
///
/// 用户自定义 size function 可能返回任意 JS 值，因此需要保留 `Value` 变体；
/// 内建 size algorithm 可以直接使用 native `f64`，减少 JS 转换。
pub enum SizeValue<'js> {
    Value(Value<'js>),
    Native(f64),
}

impl SizeValue<'_> {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Value(value) => value.as_number(),
            Self::Native(size) => Some(*size),
        }
    }
}

impl<'js> FromJs<'js> for SizeValue<'js> {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if let Some(size) = value.as_number() {
            return Ok(Self::Native(size));
        }
        Ok(Self::Value(value))
    }
}

/// Streams 规范中的 queue-with-sizes。
///
/// 参考：https://streams.spec.whatwg.org/#queue-with-sizes
#[derive(JsLifetime, Trace, Default)]
/// 带 size 累加值的 FIFO 队列。
pub struct QueueWithSizes<'js> {
    pub queue: VecDeque<ValueWithSize<'js>>,
    pub queue_total_size: f64,
}

impl<'js> QueueWithSizes<'js> {
    /// 创建空队列。
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            queue_total_size: 0.0,
        }
    }

    /// 入队一个 value-with-size。
    ///
    /// 如果 size 不是有限、非 NaN、非负数，会抛 `RangeError`。
    pub fn enqueue_value_with_size(
        &mut self,
        ctx: &Ctx<'js>,
        value: Value<'js>,
        size: SizeValue<'js>,
    ) -> Result<()> {
        let size = match is_non_negative_number(size) {
            None => {
                // 规范要求 size 必须通过 IsNonNegativeNumber 检查。
                return Err(Exception::throw_range(
                    ctx,
                    "Size must be a finite, non-NaN, non-negative number.",
                ));
            },
            Some(size) => size,
        };

        // 正无穷也不是合法 size，必须抛 RangeError。
        if size.is_infinite() {
            return Err(Exception::throw_range(
                ctx,
                "Size must be a finite, non-NaN, non-negative number.",
            ));
        };

        // 向 container.[[queue]] 追加 value-with-size。
        self.queue.push_back(ValueWithSize { value, size });

        // 同步维护 container.[[queueTotalSize]]。
        self.queue_total_size += size;

        Ok(())
    }

    /// 出队一个 value，并同步扣减 total size。
    pub fn dequeue_value(&mut self) -> Value<'js> {
        // 取出并移除 container.[[queue]][0]。
        let value_with_size = self
            .queue
            .pop_front()
            .expect("DequeueValue called with empty queue");
        // 按出队元素的 size 扣减 queueTotalSize。
        self.queue_total_size -= value_with_size.size;
        // 浮点误差可能导致轻微负数，规范要求修正为 0。
        if self.queue_total_size < 0.0 {
            self.queue_total_size = 0.0
        }
        value_with_size.value
    }

    /// 清空队列并重置 total size。
    pub fn reset_queue(&mut self) {
        // 清空 container.[[queue]]。
        self.queue.clear();
        // 重置 container.[[queueTotalSize]]。
        self.queue_total_size = 0.0;
    }
}

#[derive(JsLifetime, Trace, Clone)]
/// 队列中的一个元素及其 size。
pub struct ValueWithSize<'js> {
    pub value: Value<'js>,
    size: f64,
}

fn is_non_negative_number(value: SizeValue<'_>) -> Option<f64> {
    // 非 number 不通过检查。
    let number = value.as_number()?;
    // NaN 不通过检查。
    if number.is_nan() {
        return None;
    }

    // 负数不通过检查。
    if number < 0.0 {
        return None;
    }

    // 其余有限非负 number 通过检查。
    Some(number)
}

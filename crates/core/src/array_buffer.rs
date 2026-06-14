// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//! `ArrayBuffer` 的底层零拷贝辅助函数。
//!
//! 这个模块直接使用 QuickJS-NG 的底层 `qjs` API。原因是 `rquickjs`
//! 目前没有完整封装 QuickJS-NG 的 immutable ArrayBuffer 能力，而 vole
//! 在 Blob、Fetch body、Response body 等场景中需要安全地共享字节存储。
//!
//! 提供两个核心能力：
//! - [`set_immutable`]：把现有 `ArrayBuffer` 标记为不可变，防止 JS 用户
//!   通过 typed array / DataView 修改底层字节。
//! - [`shared_array_buffer_view`]：创建一个新的不可变 `ArrayBuffer`，
//!   让它零拷贝地引用另一个 `ArrayBuffer` 的字节区间。
//!
//! 关键安全约束：
//! - 共享视图必须是 immutable，否则用户可以修改生产者持有的原始字节。
//! - 共享视图必须阻止 `.transfer()`，否则 QuickJS 可能丢失用于保持源
//!   `ArrayBuffer` 存活的 opaque 指针。
//! - 本模块只处理内存别名和生命周期，不负责 Blob/Fetch/Streams 的业务语义。

use std::ffi::c_void;

use rquickjs::{qjs, ArrayBuffer, Ctx, Exception, Result, Value};

/// 将 `ArrayBuffer` 标记为 immutable。
///
/// 标记之后，通过 `Uint8Array` / `DataView` 写入会失败，严格模式下会抛出
/// `TypeError`；调用 `.transfer()` 也会被 QuickJS 拒绝。
///
/// 注意：这个函数直接操作 QuickJS 内部标志位。调用者必须确保传入的是当前
/// context 中仍然有效的 `ArrayBuffer`。对 detached buffer 调用是无害的 no-op。
pub fn set_immutable(ab: &ArrayBuffer<'_>) {
    // 安全性：`JSValue` 由 `ab` 持有；这里仅修改底层 `JSArrayBuffer`
    // 结构上的 immutable 标志位。
    unsafe {
        qjs::JS_SetImmutableArrayBuffer(ab.as_value().as_raw(), true);
    }
}

/// 创建一个新的 immutable `ArrayBuffer`，零拷贝引用 `source` 的一段字节。
///
/// 返回值会持有一份 dup 出来的源 `JSValue` 引用，因此只要共享视图还活着，
/// 源 buffer 的底层内存就不会被释放。
///
/// 调用者必须保证：
/// - `offset + len` 不超过源 buffer 长度。
/// - 源 buffer 没有 detached。
/// - 返回的共享视图不应该被改回 mutable。
///
/// 如果未来需要“可变共享视图”，不能复用这个函数；需要重新设计释放策略，
/// 例如使用指针索引的 side table 或修改 QuickJS 行为。
pub fn shared_array_buffer_view<'js>(
    ctx: &Ctx<'js>,
    source: &ArrayBuffer<'js>,
    offset: usize,
    len: usize,
) -> Result<ArrayBuffer<'js>> {
    let raw = source
        .as_raw()
        .ok_or_else(|| Exception::throw_type(ctx, "cannot view a detached ArrayBuffer"))?;
    debug_assert!(
        offset.checked_add(len).is_some_and(|e| e <= raw.len),
        "shared_array_buffer_view: slice out of range"
    );
    let ptr = unsafe { raw.ptr.as_ptr().add(offset) };

    // 复制源 `JSValue` 引用。返回的 ArrayBuffer 在 free callback 中释放
    // 这份引用，从而保证源 buffer 的底层存储在共享视图存活期间不会释放。
    let ctx_ptr = ctx.as_raw().as_ptr();
    let rt = unsafe { qjs::JS_GetRuntime(ctx_ptr) };
    let source_val = unsafe { qjs::JS_DupValueRT(rt, source.as_value().as_raw()) };
    let opaque = Box::into_raw(Box::new(source_val)) as *mut c_void;

    extern "C" fn free_shared(rt: *mut qjs::JSRuntime, opaque: *mut c_void, _ptr: *mut c_void) {
        // `opaque` is guaranteed non-null: the only QuickJS code path
        // that loses it is `.transfer()`, which is blocked by the
        // immutability flag we set below.
        unsafe {
            let boxed = Box::from_raw(opaque as *mut qjs::JSValue);
            qjs::JS_FreeValueRT(rt, *boxed);
        }
    }

    let view = unsafe {
        let val = qjs::JS_NewArrayBuffer(
            ctx_ptr,
            ptr,
            len as _,
            Some(free_shared),
            opaque,
            /*is_shared=*/ false,
        );
        if qjs::JS_IsException(val) {
            // QuickJS 没有接管 `opaque`，这里需要自行释放。
            let boxed = Box::from_raw(opaque as *mut qjs::JSValue);
            qjs::JS_FreeValueRT(rt, *boxed);
            return Err(ctx.throw(ctx.catch()));
        }
        let value = Value::from_raw(ctx.clone(), val);
        ArrayBuffer::from_value(value)
            .ok_or_else(|| Exception::throw_type(ctx, "expected ArrayBuffer"))?
    };

    set_immutable(&view);
    Ok(view)
}

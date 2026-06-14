// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//! Rust `Result` / `Option` 到 JS 异常的转换扩展。
//!
//! native binding 中经常会从 Rust API 得到 `std::result::Result<T, E>` 或
//! `Option<T>`。Web API 需要把这些失败统一转换成 JS exception，例如
//! `TypeError`、`RangeError` 或普通 `Error`。
//!
//! 本模块提供两个扩展 trait：
//! - [`ResultExt`]：把 Rust 错误或缺失值转成 JS exception。
//! - [`OptionExt`]：补充一些“返回 Result 的 Option 链式操作”。
//!
//! 注意：这里只负责抛出 JS exception，不负责把 exception 转成 rejected promise。
#![allow(clippy::uninlined_format_args)]

use std::{fmt::Write, result::Result as StdResult};

use rquickjs::{Ctx, Exception, Result};

/// 将 Rust `Result` / `Option` 转换为 rquickjs `Result` 的扩展。
pub trait ResultExt<T> {
    fn or_throw_msg(self, ctx: &Ctx, msg: &str) -> Result<T>;
    fn or_throw_range(self, ctx: &Ctx, msg: &str) -> Result<T>;
    fn or_throw_type(self, ctx: &Ctx, msg: &str) -> Result<T>;
    fn or_throw(self, ctx: &Ctx) -> Result<T>;
}

/// `Option` 的 fallible 链式辅助。
pub trait OptionExt<T> {
    fn and_then_ok<U, E, F>(self, f: F) -> StdResult<Option<U>, E>
    where
        F: FnOnce(T) -> StdResult<Option<U>, E>;

    fn unwrap_or_else_ok<E, F>(self, f: F) -> StdResult<T, E>
    where
        F: FnOnce() -> StdResult<T, E>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for StdResult<T, E> {
    fn or_throw_msg(self, ctx: &Ctx, msg: &str) -> Result<T> {
        self.map_err(|e| {
            let mut message = String::with_capacity(100);
            message.push_str(msg);
            message.push_str(". ");
            write!(message, "{}", e).unwrap();
            Exception::throw_message(ctx, &message)
        })
    }

    fn or_throw_range(self, ctx: &Ctx, msg: &str) -> Result<T> {
        self.map_err(|e| {
            let mut message = String::with_capacity(100);
            if !message.is_empty() {
                message.push_str(msg);
                message.push_str(". ");
            }
            write!(message, "{}", e).unwrap();
            Exception::throw_range(ctx, &message)
        })
    }

    fn or_throw_type(self, ctx: &Ctx, msg: &str) -> Result<T> {
        self.map_err(|e| {
            let mut message = String::with_capacity(100);
            if !msg.is_empty() {
                message.push_str(msg);
                message.push_str(". ");
            }
            write!(message, "{}", e).unwrap();
            Exception::throw_type(ctx, &message)
        })
    }

    fn or_throw(self, ctx: &Ctx) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<T> ResultExt<T> for Option<T> {
    fn or_throw_msg(self, ctx: &Ctx, msg: &str) -> Result<T> {
        self.ok_or_else(|| Exception::throw_message(ctx, msg))
    }

    fn or_throw_range(self, ctx: &Ctx, msg: &str) -> Result<T> {
        self.ok_or_else(|| Exception::throw_range(ctx, msg))
    }

    fn or_throw_type(self, ctx: &Ctx, msg: &str) -> Result<T> {
        self.ok_or_else(|| Exception::throw_type(ctx, msg))
    }

    fn or_throw(self, ctx: &Ctx) -> Result<T> {
        self.ok_or_else(|| Exception::throw_message(ctx, "Value is not present"))
    }
}

impl<T> OptionExt<T> for Option<T> {
    fn and_then_ok<U, E, F>(self, f: F) -> StdResult<Option<U>, E>
    where
        F: FnOnce(T) -> StdResult<Option<U>, E>,
    {
        match self {
            Some(v) => f(v),
            None => Ok(None),
        }
    }

    fn unwrap_or_else_ok<E, F>(self, f: F) -> StdResult<T, E>
    where
        F: FnOnce() -> StdResult<T, E>,
    {
        match self {
            Some(v) => Ok(v),
            None => f(),
        }
    }
}

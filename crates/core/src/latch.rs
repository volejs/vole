// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//! 简单异步计数闩锁。
//!
//! [`Latch`] 用于等待一组异步任务或资源操作全部完成。调用者在开始任务前
//! `increment`，任务结束时 `decrement`，需要等待全部完成时调用 `wait`。
//!
//! 适用场景：
//! - 等待 socket/server/timer 等资源关闭。
//! - 等待多个 native async 操作 drain 完成。
//! - 在 binding 内部协调生命周期，避免提前释放共享状态。
//!
//! 注意事项：
//! - 这是轻量工具，不记录每个任务的错误；错误传播需要调用者自行处理。
//! - `decrement` 次数不能超过 `increment`，否则 `AtomicUsize` 会下溢。
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::Notify;

#[derive(Default)]
/// 基于计数器和 `tokio::sync::Notify` 的异步 latch。
pub struct Latch {
    count: AtomicUsize,
    notify: Notify,
}

impl Latch {
    pub fn increment(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement(&self) {
        let previous = self.count.fetch_sub(1, Ordering::Relaxed);
        if previous == 1 {
            self.notify.notify_waiters();
        }
    }

    pub async fn wait(&self) {
        if self.count.load(Ordering::Relaxed) > 0 {
            self.notify.notified().await;
        }
    }
}

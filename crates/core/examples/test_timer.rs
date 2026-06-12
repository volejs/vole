use std::time::Duration;
use rquickjs::{Ctx, Function};

fn set_timeout_impl<'js>(ctx: Ctx<'js>, callback: Function<'js>, delay: Option<u64>) -> u64 {
    let delay = delay.unwrap_or(0);
    let cb = callback.clone();
    ctx.spawn(async move {
        tokio::time::sleep(Duration::from_millis(delay)).await;
        let _ = cb.call::<_, ()>(());
    });
    1
}

#[tokio::main]
async fn main() {
    let rt = rquickjs::AsyncRuntime::new().unwrap();
    let ctx = rquickjs::AsyncContext::full(&rt).await.unwrap();
    let _ = ctx.with(|ctx| {
        ctx.globals().set("setTimeout", rquickjs::Function::new(ctx.clone(), set_timeout_impl).unwrap()).unwrap();
        ctx.eval::<(), _>("setTimeout(() => console.log('delayed'), 100)")
    }).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
}

use std::future::Future;

use rquickjs::Ctx;

pub(crate) trait CtxExtension<'js> {
    fn spawn_exit_simple<F>(&self, future: F)
    where
        F: Future<Output = rquickjs::Result<()>> + 'js;
}

impl<'js> CtxExtension<'js> for Ctx<'js> {
    fn spawn_exit_simple<F>(&self, future: F)
    where
        F: Future<Output = rquickjs::Result<()>> + 'js,
    {
        self.spawn(async move {
            let _ = future.await;
        });
    }
}

use rquickjs::{Class, Ctx, Result};

pub mod blob;
pub mod file;

pub use blob::Blob;
pub use file::File;

pub fn init(ctx: &Ctx) -> Result<()> {
    Class::<Blob>::define(&ctx.globals())?;
    Class::<File>::define(&ctx.globals())?;

    // Convert Blob prototype methods to getters
    ctx.eval::<(), _>(
        r#"
        (function() {
            const wrapGetter = (proto, name) => {
                const fn = proto[name];
                Object.defineProperty(proto, name, {
                    get: function() { return fn.call(this); },
                    enumerable: true,
                    configurable: true
                });
            };
            wrapGetter(Blob.prototype, 'size');
            wrapGetter(Blob.prototype, 'type');
            wrapGetter(File.prototype, 'name');
            wrapGetter(File.prototype, 'lastModified');
            wrapGetter(File.prototype, 'size');
            wrapGetter(File.prototype, 'type');
            Object.defineProperty(File.prototype, 'webkitRelativePath', {
                get: function() { return ''; },
                enumerable: true,
                configurable: true
            });

            Blob.prototype.stream = function() {
                var self = this;
                // ReadableStream is defined later in the init sequence; deferring
                // the lookup until .stream() is called avoids ordering issues.
                return new ReadableStream({
                    start: function(controller) {
                        var buf = self.arrayBuffer();
                        controller.enqueue(new Uint8Array(buf));
                        controller.close();
                    }
                });
            };

        })();
    "#,
    )?;

    Ok(())
}

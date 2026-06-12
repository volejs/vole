use rquickjs::{
    function::Opt,
    module::{Declarations, Exports, ModuleDef},
    Ctx, Object, Promise, Result, TypedArray,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex as TokioMutex;

static FILES: std::sync::OnceLock<Mutex<HashMap<u64, Arc<TokioMutex<tokio::fs::File>>>>> =
    std::sync::OnceLock::new();
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn files() -> &'static Mutex<HashMap<u64, Arc<TokioMutex<tokio::fs::File>>>> {
    FILES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn file_open<'js>(
    ctx: Ctx<'js>,
    path: String,
    options: Opt<Object<'js>>,
) -> Result<Promise<'js>> {
    let mut opts = tokio::fs::OpenOptions::new();
    opts.read(true);

    if let Some(obj) = options.0 {
        if let Ok(v) = obj.get::<&str, bool>("write") {
            if v {
                opts.write(true);
            }
        }
        if let Ok(v) = obj.get::<&str, bool>("append") {
            if v {
                opts.append(true);
            }
        }
        if let Ok(v) = obj.get::<&str, bool>("create") {
            if v {
                opts.create(true);
            }
        }
        if let Ok(v) = obj.get::<&str, bool>("truncate") {
            if v {
                opts.truncate(true);
            }
        }
    }

    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match opts.open(&path).await {
            Ok(file) => {
                let id = next_id();
                files()
                    .lock()
                    .unwrap()
                    .insert(id, Arc::new(TokioMutex::new(file)));
                let obj = Object::new(ctx_clone).unwrap();
                let _ = obj.set("_id", id);
                let _ = resolve.call::<_, ()>((obj,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn file_read<'js>(
    ctx: Ctx<'js>,
    id: u64,
    size: Opt<u32>,
) -> Result<Promise<'js>> {
    let file = files()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        let mut buf = vec![0u8; size.0.unwrap_or(4096) as usize];
        match file.lock().await.read(&mut buf).await {
            Ok(n) => {
                buf.truncate(n);
                match TypedArray::new_copy(ctx_clone, buf) {
                    Ok(ta) => {
                        let _ = resolve.call::<_, ()>((ta,));
                    }
                    Err(_) => {
                        let _ = reject.call::<_, ()>(("read failed",));
                    }
                }
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn file_write<'js>(
    ctx: Ctx<'js>,
    id: u64,
    data: TypedArray<'js, u8>,
) -> Result<Promise<'js>> {
    let file = files()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match file.lock().await.write_all(&bytes).await {
            Ok(()) => {
                let _ = resolve.call::<_, ()>((bytes.len() as f64,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn file_seek<'js>(
    ctx: Ctx<'js>,
    id: u64,
    offset: i64,
    whence: String,
) -> Result<Promise<'js>> {
    let file = files()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let from = match whence.as_str() {
        "start" => std::io::SeekFrom::Start(offset as u64),
        "current" => std::io::SeekFrom::Current(offset),
        "end" => std::io::SeekFrom::End(offset),
        _ => std::io::SeekFrom::Start(offset as u64),
    };
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match file.lock().await.seek(from).await {
            Ok(pos) => {
                let _ = resolve.call::<_, ()>((pos as f64,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn file_close<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let _ = files().lock().unwrap().remove(&id);
    let (promise, resolve, _reject) = ctx.promise()?;
    let _ = resolve.call::<_, ()>(((),));
    Ok(promise)
}

/// Native module exposing the low-level file operations used by the `std:file` JS shim.
pub struct FileRawModule;

impl ModuleDef for FileRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_file_open")?;
        decl.declare("__std_file_read")?;
        decl.declare("__std_file_write")?;
        decl.declare("__std_file_seek")?;
        decl.declare("__std_file_close")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__std_file_open", rquickjs::Function::new(ctx.clone(), file_open))?;
        exports.export("__std_file_read", rquickjs::Function::new(ctx.clone(), file_read))?;
        exports.export("__std_file_write", rquickjs::Function::new(ctx.clone(), file_write))?;
        exports.export("__std_file_seek", rquickjs::Function::new(ctx.clone(), file_seek))?;
        exports.export("__std_file_close", rquickjs::Function::new(ctx.clone(), file_close))?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `std:file`.
/// All file helpers are provided through the native module `std:raw:file`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

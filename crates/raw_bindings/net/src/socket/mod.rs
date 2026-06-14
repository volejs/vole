use rquickjs::{
    function::Opt,
    module::{Declarations, Exports, ModuleDef},
    Ctx, Function, Object, Promise, Result, TypedArray,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex as TokioMutex;

#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

static SOCKETS: std::sync::OnceLock<Mutex<HashMap<u64, Arc<TokioMutex<UnixStream>>>>> =
    std::sync::OnceLock::new();
static SERVERS: std::sync::OnceLock<Mutex<HashMap<u64, Arc<TokioMutex<UnixListener>>>>> =
    std::sync::OnceLock::new();
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn sockets() -> &'static Mutex<HashMap<u64, Arc<TokioMutex<UnixStream>>>> {
    SOCKETS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn servers() -> &'static Mutex<HashMap<u64, Arc<TokioMutex<UnixListener>>>> {
    SERVERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

#[cfg(unix)]
fn socket_connect<'js>(ctx: Ctx<'js>, path: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match UnixStream::connect(&path).await {
            Ok(stream) => {
                let id = next_id();
                sockets()
                    .lock()
                    .unwrap()
                    .insert(id, Arc::new(TokioMutex::new(stream)));
                let obj = Object::new(ctx_clone).unwrap();
                let _ = obj.set("_id", id);
                let _ = obj.set("path", path);
                let _ = resolve.call::<_, ()>((obj,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

#[cfg(not(unix))]
fn socket_connect<'js>(ctx: Ctx<'js>, _path: String) -> Result<Promise<'js>> {
    let (promise, _resolve, reject) = ctx.promise()?;
    let _ = reject.call::<_, ()>(("Unix sockets are not supported on this platform",));
    Ok(promise)
}

fn socket_read<'js>(ctx: Ctx<'js>, id: u64, size: Opt<u32>) -> Result<Promise<'js>> {
    let stream = sockets()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        let mut buf = vec![0u8; size.0.unwrap_or(4096) as usize];
        match stream.lock().await.read(&mut buf).await {
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

fn socket_write<'js>(ctx: Ctx<'js>, id: u64, data: TypedArray<'js, u8>) -> Result<Promise<'js>> {
    let stream = sockets()
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
        match stream.lock().await.write_all(&bytes).await {
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

fn socket_close<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let _ = sockets().lock().unwrap().remove(&id);
    let (promise, resolve, _reject) = ctx.promise()?;
    let _ = resolve.call::<_, ()>(((),));
    Ok(promise)
}

#[cfg(unix)]
fn socket_listen<'js>(ctx: Ctx<'js>, path: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        let _ = std::fs::remove_file(&path);
        match UnixListener::bind(&path) {
            Ok(listener) => {
                let id = next_id();
                servers()
                    .lock()
                    .unwrap()
                    .insert(id, Arc::new(TokioMutex::new(listener)));
                let obj = Object::new(ctx_clone).unwrap();
                let _ = obj.set("_id", id);
                let _ = obj.set("path", path);
                let _ = resolve.call::<_, ()>((obj,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

#[cfg(not(unix))]
fn socket_listen<'js>(ctx: Ctx<'js>, _path: String) -> Result<Promise<'js>> {
    let (promise, _resolve, reject) = ctx.promise()?;
    let _ = reject.call::<_, ()>(("Unix sockets are not supported on this platform",));
    Ok(promise)
}

fn socket_server_accept<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let server = servers()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match server.lock().await.accept().await {
            Ok((stream, _addr)) => {
                let socket_id = next_id();
                sockets()
                    .lock()
                    .unwrap()
                    .insert(socket_id, Arc::new(TokioMutex::new(stream)));
                let obj = Object::new(ctx_clone).unwrap();
                let _ = obj.set("_id", socket_id);
                let _ = resolve.call::<_, ()>((obj,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn socket_server_close<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let _ = servers().lock().unwrap().remove(&id);
    let (promise, resolve, _reject) = ctx.promise()?;
    let _ = resolve.call::<_, ()>(((),));
    Ok(promise)
}

/// Native module exposing the low-level Unix socket operations used by the `vole:net/socket` JS shim.
pub struct SocketRawModule;

impl ModuleDef for SocketRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_socket_connect")?;
        decl.declare("__std_socket_read")?;
        decl.declare("__std_socket_write")?;
        decl.declare("__std_socket_close")?;
        decl.declare("__std_socket_listen")?;
        decl.declare("__std_socket_server_accept")?;
        decl.declare("__std_socket_server_close")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export(
            "__std_socket_connect",
            Function::new(ctx.clone(), socket_connect),
        )?;
        exports.export("__std_socket_read", Function::new(ctx.clone(), socket_read))?;
        exports.export(
            "__std_socket_write",
            Function::new(ctx.clone(), socket_write),
        )?;
        exports.export(
            "__std_socket_close",
            Function::new(ctx.clone(), socket_close),
        )?;
        exports.export(
            "__std_socket_listen",
            Function::new(ctx.clone(), socket_listen),
        )?;
        exports.export(
            "__std_socket_server_accept",
            Function::new(ctx.clone(), socket_server_accept),
        )?;
        exports.export(
            "__std_socket_server_close",
            Function::new(ctx.clone(), socket_server_close),
        )?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `vole:net/socket`.
/// All Unix socket helpers are provided through the native module `vole:raw:net/socket`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

use rquickjs::{
    function::Opt,
    module::{Declarations, Exports, ModuleDef},
    Ctx, Function, Object, Promise, Result, TypedArray,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex as TokioMutex;

static SOCKETS: std::sync::OnceLock<Mutex<HashMap<u64, Arc<TokioMutex<TcpStream>>>>> =
    std::sync::OnceLock::new();
static SERVERS: std::sync::OnceLock<Mutex<HashMap<u64, Arc<TokioMutex<TcpListener>>>>> =
    std::sync::OnceLock::new();
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn sockets() -> &'static Mutex<HashMap<u64, Arc<TokioMutex<TcpStream>>>> {
    SOCKETS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn servers() -> &'static Mutex<HashMap<u64, Arc<TokioMutex<TcpListener>>>> {
    SERVERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn tcp_connect<'js>(ctx: Ctx<'js>, host: String, port: u16) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match TcpStream::connect((host, port)).await {
            Ok(stream) => {
                let id = next_id();
                sockets()
                    .lock()
                    .unwrap()
                    .insert(id, Arc::new(TokioMutex::new(stream)));
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

fn tcp_socket_read<'js>(ctx: Ctx<'js>, id: u64, size: Opt<u32>) -> Result<Promise<'js>> {
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

fn tcp_socket_write<'js>(
    ctx: Ctx<'js>,
    id: u64,
    data: TypedArray<'js, u8>,
) -> Result<Promise<'js>> {
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

fn tcp_socket_close<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let stream = sockets().lock().unwrap().remove(&id);
    let (promise, resolve, _reject) = ctx.promise()?;
    if let Some(stream) = stream {
        ctx.spawn(async move {
            let _ = stream.lock().await.shutdown().await;
            let _ = resolve.call::<_, ()>(((),));
        });
    } else {
        let _ = resolve.call::<_, ()>(((),));
    }
    Ok(promise)
}

fn tcp_listen<'js>(ctx: Ctx<'js>, port: u16, host: Opt<String>) -> Result<Promise<'js>> {
    let host = host.0.unwrap_or_else(|| "0.0.0.0".to_string());
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match TcpListener::bind((host, port)).await {
            Ok(listener) => {
                let id = next_id();
                let addr = listener.local_addr().unwrap();
                servers()
                    .lock()
                    .unwrap()
                    .insert(id, Arc::new(TokioMutex::new(listener)));
                let obj = Object::new(ctx_clone.clone()).unwrap();
                let _ = obj.set("_id", id);
                let addr_obj = Object::new(ctx_clone).unwrap();
                let _ = addr_obj.set("host", addr.ip().to_string());
                let _ = addr_obj.set("port", addr.port() as f64);
                let _ = obj.set("address", addr_obj);
                let _ = resolve.call::<_, ()>((obj,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn tcp_server_accept<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
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

fn tcp_server_close<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let _ = servers().lock().unwrap().remove(&id);
    let (promise, resolve, _reject) = ctx.promise()?;
    let _ = resolve.call::<_, ()>(((),));
    Ok(promise)
}

/// Native module exposing the low-level TCP operations used by the `vole:net/tcp` JS shim.
pub struct TcpRawModule;

impl ModuleDef for TcpRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_tcp_connect")?;
        decl.declare("__std_tcp_socket_read")?;
        decl.declare("__std_tcp_socket_write")?;
        decl.declare("__std_tcp_socket_close")?;
        decl.declare("__std_tcp_listen")?;
        decl.declare("__std_tcp_server_accept")?;
        decl.declare("__std_tcp_server_close")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__std_tcp_connect", Function::new(ctx.clone(), tcp_connect))?;
        exports.export(
            "__std_tcp_socket_read",
            Function::new(ctx.clone(), tcp_socket_read),
        )?;
        exports.export(
            "__std_tcp_socket_write",
            Function::new(ctx.clone(), tcp_socket_write),
        )?;
        exports.export(
            "__std_tcp_socket_close",
            Function::new(ctx.clone(), tcp_socket_close),
        )?;
        exports.export("__std_tcp_listen", Function::new(ctx.clone(), tcp_listen))?;
        exports.export(
            "__std_tcp_server_accept",
            Function::new(ctx.clone(), tcp_server_accept),
        )?;
        exports.export(
            "__std_tcp_server_close",
            Function::new(ctx.clone(), tcp_server_close),
        )?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `vole:net/tcp`.
/// All TCP helpers are provided through the native module `vole:raw:net/tcp`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

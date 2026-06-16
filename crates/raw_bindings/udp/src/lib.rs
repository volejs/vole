use rquickjs::{
    Ctx, Function, Object, Promise, Result, TypedArray,
    function::Opt,
    module::{Declarations, Exports, ModuleDef},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::Mutex as TokioMutex;

static SOCKETS: std::sync::OnceLock<Mutex<HashMap<u64, Arc<TokioMutex<UdpSocket>>>>> =
    std::sync::OnceLock::new();
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn sockets() -> &'static Mutex<HashMap<u64, Arc<TokioMutex<UdpSocket>>>> {
    SOCKETS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn udp_create_socket<'js>(
    ctx: Ctx<'js>,
    port: Opt<u16>,
    host: Opt<String>,
) -> Result<Promise<'js>> {
    let host = host.0.unwrap_or_else(|| "0.0.0.0".to_string());
    let port = port.0.unwrap_or(0);
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match UdpSocket::bind((host, port)).await {
            Ok(socket) => {
                let id = next_id();
                let addr = socket.local_addr().unwrap();
                sockets()
                    .lock()
                    .unwrap()
                    .insert(id, Arc::new(TokioMutex::new(socket)));
                let obj = Object::new(ctx_clone.clone()).unwrap();
                let _ = obj.set("_id", id);
                let addr_obj = Object::new(ctx_clone).unwrap();
                let _ = addr_obj.set("host", addr.ip().to_string());
                let _ = addr_obj.set("port", addr.port() as f64);
                let _ = addr_obj.set("family", if addr.is_ipv4() { "IPv4" } else { "IPv6" });
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

fn udp_socket_send<'js>(
    ctx: Ctx<'js>,
    id: u64,
    data: TypedArray<'js, u8>,
    remote_host: String,
    remote_port: u16,
) -> Result<Promise<'js>> {
    let socket = sockets()
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
        match socket
            .lock()
            .await
            .send_to(&bytes, (remote_host, remote_port))
            .await
        {
            Ok(n) => {
                let _ = resolve.call::<_, ()>((n as f64,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn udp_socket_recv<'js>(ctx: Ctx<'js>, id: u64, size: Opt<u32>) -> Result<Promise<'js>> {
    let socket = sockets()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        let mut buf = vec![0u8; size.0.unwrap_or(65535) as usize];
        match socket.lock().await.recv_from(&mut buf).await {
            Ok((n, addr)) => {
                buf.truncate(n);
                let data = match TypedArray::new_copy(ctx_clone.clone(), buf) {
                    Ok(ta) => ta,
                    Err(_) => {
                        let _ = reject.call::<_, ()>(("recv failed",));
                        return;
                    }
                };
                let addr_obj = Object::new(ctx_clone.clone()).unwrap();
                let _ = addr_obj.set("host", addr.ip().to_string());
                let _ = addr_obj.set("port", addr.port() as f64);
                let _ = addr_obj.set("family", if addr.is_ipv4() { "IPv4" } else { "IPv6" });
                let result = Object::new(ctx_clone).unwrap();
                let _ = result.set("data", data);
                let _ = result.set("address", addr_obj);
                let _ = resolve.call::<_, ()>((result,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn udp_socket_close<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let _ = sockets().lock().unwrap().remove(&id);
    let (promise, resolve, _reject) = ctx.promise()?;
    let _ = resolve.call::<_, ()>(((),));
    Ok(promise)
}

fn udp_socket_local_address<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let socket = sockets()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match socket.lock().await.local_addr() {
            Ok(addr) => {
                let obj = Object::new(ctx_clone).unwrap();
                let _ = obj.set("host", addr.ip().to_string());
                let _ = obj.set("port", addr.port() as f64);
                let _ = obj.set("family", if addr.is_ipv4() { "IPv4" } else { "IPv6" });
                let _ = resolve.call::<_, ()>((obj,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

/// Native module exposing the low-level UDP operations used by the `vole:udp` JS shim.
pub struct UdpRawModule;

impl ModuleDef for UdpRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_udp_create_socket")?;
        decl.declare("__std_udp_socket_send")?;
        decl.declare("__std_udp_socket_recv")?;
        decl.declare("__std_udp_socket_close")?;
        decl.declare("__std_udp_socket_local_address")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export(
            "__std_udp_create_socket",
            Function::new(ctx.clone(), udp_create_socket),
        )?;
        exports.export(
            "__std_udp_socket_send",
            Function::new(ctx.clone(), udp_socket_send),
        )?;
        exports.export(
            "__std_udp_socket_recv",
            Function::new(ctx.clone(), udp_socket_recv),
        )?;
        exports.export(
            "__std_udp_socket_close",
            Function::new(ctx.clone(), udp_socket_close),
        )?;
        exports.export(
            "__std_udp_socket_local_address",
            Function::new(ctx.clone(), udp_socket_local_address),
        )?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `vole:udp`.
/// All UDP helpers are provided through the native module `vole:raw:udp`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

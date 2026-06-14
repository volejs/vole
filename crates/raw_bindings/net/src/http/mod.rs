use axum::{
    body::Bytes,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response as AxumResponse},
    routing::any,
    Router,
};
use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    Array, Ctx, Function, Object, Promise, Result, TypedArray, Value,
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, OnceLock,
};
use tokio::sync::{mpsc, oneshot};

static SERVERS: OnceLock<Mutex<HashMap<u64, ServerHandle>>> = OnceLock::new();
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
struct ServerHandle {
    shutdown: mpsc::Sender<()>,
}

fn servers() -> &'static Mutex<HashMap<u64, ServerHandle>> {
    SERVERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    uri: String,
    headers: Vec<Vec<String>>,
    body: Vec<u8>,
    response_tx: oneshot::Sender<HttpResponse>,
}

#[derive(Debug, Clone)]
struct HttpResponse {
    status: u16,
    headers: Vec<Vec<String>>,
    body: Vec<u8>,
}

fn http_serve<'js>(
    ctx: Ctx<'js>,
    hostname: String,
    port: u16,
    handler_id: String,
) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();

    let addr: std::net::SocketAddr = match format!("{}:{}", hostname, port).parse() {
        Ok(a) => a,
        Err(_) => {
            return Err(rquickjs::Error::Exception);
        }
    };

    ctx.spawn(async move {
        let (req_tx, mut req_rx) = mpsc::channel::<HttpRequest>(32);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        let server_id = next_id();
        servers().lock().unwrap().insert(
            server_id,
            ServerHandle {
                shutdown: shutdown_tx,
            },
        );

        // Start the axum server in a dedicated tokio task.
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("http server bind error: {}", e);
                let _ = reject.call::<_, ()>((e.to_string(),));
                return;
            }
        };

        let axum_handle = tokio::spawn(run_axum_server(listener, req_tx));

        // Resolve the promise immediately with the server id.
        let result = Object::new(ctx_clone.clone()).unwrap();
        let _ = result.set("_id", server_id);
        let _ = resolve.call::<_, ()>((result,));

        // Process incoming HTTP requests by delegating to the JS handler.
        loop {
            tokio::select! {
                Some(req) = req_rx.recv() => {
                    let response_tx = req.response_tx;
                    let resp = handle_request(
                        ctx_clone.clone(),
                        handler_id.clone(),
                        req.method,
                        req.uri,
                        req.headers,
                        req.body,
                    ).await;
                    let _ = response_tx.send(resp);
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
                else => break,
            }
        }

        axum_handle.abort();
        servers().lock().unwrap().remove(&server_id);
    });

    Ok(promise)
}

fn http_server_shutdown<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let (promise, resolve, _reject) = ctx.promise()?;

    let handle = servers().lock().unwrap().get(&id).cloned();
    ctx.spawn(async move {
        if let Some(handle) = handle {
            let _ = handle.shutdown.send(()).await;
        }
        let _ = resolve.call::<_, ()>(((),));
    });

    Ok(promise)
}

async fn run_axum_server(listener: tokio::net::TcpListener, req_tx: mpsc::Sender<HttpRequest>) {
    let app =
        Router::new().route(
            "/{*path}",
            any(
                move |method: Method,
                      uri: Uri,
                      headers: HeaderMap,
                      body: Bytes|
                      -> std::pin::Pin<
                    Box<dyn std::future::Future<Output = AxumResponse> + Send>,
                > {
                    let req_tx = req_tx.clone();
                    Box::pin(async move {
                        let headers_vec: Vec<Vec<String>> = headers
                            .iter()
                            .map(|(k, v)| {
                                vec![k.as_str().to_string(), v.to_str().unwrap_or("").to_string()]
                            })
                            .collect();

                        let (response_tx, response_rx) = oneshot::channel::<HttpResponse>();
                        let req = HttpRequest {
                            method: method.as_str().to_string(),
                            uri: uri.to_string(),
                            headers: headers_vec,
                            body: body.to_vec(),
                            response_tx,
                        };

                        if req_tx.send(req).await.is_err() {
                            return (StatusCode::SERVICE_UNAVAILABLE, "server shutting down")
                                .into_response();
                        }

                        match response_rx.await {
                            Ok(resp) => {
                                let mut axum_resp = AxumResponse::builder().status(resp.status);
                                for pair in resp.headers {
                                    if pair.len() >= 2 {
                                        axum_resp = axum_resp.header(&pair[0], &pair[1]);
                                    }
                                }
                                axum_resp
                                    .body(axum::body::Body::from(resp.body))
                                    .unwrap_or_else(|_| {
                                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                                    })
                            }
                            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                        }
                    })
                },
            ),
        );

    let _ = axum::serve(listener, app).await;
}

async fn handle_request<'js>(
    ctx: Ctx<'js>,
    handler_id: String,
    method: String,
    uri: String,
    headers: Vec<Vec<String>>,
    body: Vec<u8>,
) -> HttpResponse {
    let result = (async || -> Result<HttpResponse> {
        let globals = ctx.globals();
        // The JS shim registers the dispatcher under a private global name so that
        // the Rust backend can invoke it without exposing user handlers on globalThis.
        let dispatcher: Function = globals
            .get("__vole_http_dispatcher")
            .map_err(|_| rquickjs::Error::Exception)?;

        let headers_arr = Array::new(ctx.clone())?;
        for (i, pair) in headers.iter().enumerate() {
            let arr = Array::new(ctx.clone())?;
            for (j, s) in pair.iter().enumerate() {
                arr.set(j, s.as_str())?;
            }
            headers_arr.set(i, arr)?;
        }

        let body_ta = TypedArray::new_copy(ctx.clone(), &body)?;

        let result: Value = dispatcher.call((
            handler_id.as_str(),
            method.as_str(),
            uri.as_str(),
            headers_arr,
            body_ta,
        ))?;

        let result = if result.is_promise() {
            let promise: Promise = result.into_promise().ok_or(rquickjs::Error::Exception)?;
            promise.into_future::<Value>().await?
        } else {
            result
        };

        let obj = result.into_object().ok_or(rquickjs::Error::Exception)?;
        let status: u16 = obj.get("status")?;
        let headers: Vec<Vec<String>> = obj.get("headers")?;
        let body_ta: TypedArray<u8> = obj.get("body")?;
        let body = if let Some(raw) = body_ta.as_raw() {
            unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
        } else {
            Vec::new()
        };

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    })()
    .await;

    match result {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("http handler error: {:?}", e);
            HttpResponse {
                status: 500,
                headers: Vec::new(),
                body: b"Internal Server Error".to_vec(),
            }
        }
    }
}

/// Native module exposing the low-level HTTP server operations used by the `vole:net/http` JS shim.
pub struct HttpRawModule;

impl ModuleDef for HttpRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_http_serve")?;
        decl.declare("__std_http_server_shutdown")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__std_http_serve", Function::new(ctx.clone(), http_serve))?;
        exports.export(
            "__std_http_server_shutdown",
            Function::new(ctx.clone(), http_server_shutdown),
        )?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for the HTTP server backend.
/// Server lifecycle helpers are provided through `vole:raw:net/http`; the request dispatcher
/// remains a private JS global (`__vole_http_dispatcher`) because the Rust backend
/// needs a stable callback entry point across async request handling.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

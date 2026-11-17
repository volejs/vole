use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use hickory_resolver::{
    config::{ResolverConfig, GOOGLE},
    net::runtime::TokioRuntimeProvider,
    proto::rr::RecordType,
    TokioResolver,
};
use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::{Constructor, Opt, This},
    Array, Class, Ctx, Exception, Function, IntoJs, Object, Promise, Result, Value,
};

use crate::record::DnsIpRecord;

#[derive(Clone, Debug)]
pub struct DnsResolver {
    resolver: Arc<TokioResolver>,
    cache_enabled: bool,
    cache: Arc<Mutex<HashMap<String, Vec<DnsIpRecord>>>>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for DnsResolver {
    type Changed<'to> = DnsResolver;
}

impl<'js> Trace<'js> for DnsResolver {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for DnsResolver {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

impl DnsResolver {
    fn new(cache_enabled: bool) -> std::result::Result<Self, String> {
        Ok(Self {
            resolver: Arc::new(create_hickory_resolver()?),
            cache_enabled,
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn cached(&self, hostname: &str) -> Option<Vec<DnsIpRecord>> {
        if !self.cache_enabled {
            return None;
        }
        self.cache
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .get(hostname)
            .cloned()
    }

    fn store_cache(&self, hostname: String, records: Vec<DnsIpRecord>) {
        if !self.cache_enabled {
            return;
        }
        self.cache
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .insert(hostname, records);
    }
}

static DEFAULT_RESOLVER: OnceLock<std::result::Result<TokioResolver, String>> = OnceLock::new();

fn create_hickory_resolver() -> std::result::Result<TokioResolver, String> {
    match TokioResolver::builder_tokio() {
        Ok(builder) => builder.build().map_err(|err| err.to_string()),
        Err(_) => TokioResolver::builder_with_config(
            ResolverConfig::udp_and_tcp(&GOOGLE),
            TokioRuntimeProvider::default(),
        )
        .build()
        .map_err(|err| err.to_string()),
    }
}

fn default_resolver() -> std::result::Result<TokioResolver, String> {
    DEFAULT_RESOLVER.get_or_init(create_hickory_resolver).clone()
}

async fn lookup_records(
    resolver: TokioResolver,
    hostname: String,
) -> std::result::Result<Vec<DnsIpRecord>, String> {
    let lookup = resolver
        .lookup_ip(hostname)
        .await
        .map_err(|err| err.to_string())?;
    let mut seen = HashSet::new();
    let mut records = Vec::new();
    for ip in lookup.iter() {
        if seen.insert(ip) {
            records.push(DnsIpRecord::from_ip(ip));
        }
    }
    Ok(records)
}

fn validate_hostname<'js>(ctx: &Ctx<'js>, hostname: &str) -> Result<()> {
    if hostname.is_empty() {
        return Err(Exception::throw_type(ctx, "DNS hostname must not be empty"));
    }
    Ok(())
}

fn records_to_ip_array<'js>(ctx: &Ctx<'js>, records: Vec<DnsIpRecord>) -> Result<Array<'js>> {
    let array = Array::new(ctx.clone())?;
    for (idx, record) in records.into_iter().enumerate() {
        array.set(idx, record)?;
    }
    Ok(array)
}

fn records_to_host_array<'js>(ctx: &Ctx<'js>, records: Vec<DnsIpRecord>) -> Result<Array<'js>> {
    let array = Array::new(ctx.clone())?;
    for (idx, record) in records.into_iter().enumerate() {
        array.set(idx, record.address)?;
    }
    Ok(array)
}

fn resolver_options_cache<'js>(ctx: &Ctx<'js>, options: Option<Value<'js>>) -> Result<bool> {
    let Some(options) = options else {
        return Ok(true);
    };
    if options.is_undefined() {
        return Ok(true);
    }
    let object = options
        .as_object()
        .ok_or_else(|| Exception::throw_type(ctx, "DnsResolver options must be an object"))?;
    object.get("cache").or(Ok(true))
}

fn resolver_ctor<'js>(ctx: Ctx<'js>, options: Opt<Value<'js>>) -> Result<DnsResolver> {
    let cache_enabled = resolver_options_cache(&ctx, options.0)?;
    DnsResolver::new(cache_enabled).map_err(|err| Exception::throw_message(&ctx, &err))
}

fn resolver_lookup_records<'js>(
    ctx: Ctx<'js>,
    resolver: DnsResolver,
    hostname: String,
) -> Result<Promise<'js>> {
    validate_hostname(&ctx, &hostname)?;

    if let Some(records) = resolver.cached(&hostname) {
        let (promise, resolve, _reject) = Promise::new(&ctx)?;
        let array = records_to_ip_array(&ctx, records)?;
        resolve.call::<_, ()>((array,))?;
        return Ok(promise);
    }

    let (promise, resolve, reject) = Promise::new(&ctx)?;
    let ctx_for_resolve = ctx.clone();
    ctx.spawn(async move {
        match lookup_records((*resolver.resolver).clone(), hostname.clone()).await {
            Ok(records) => {
                resolver.store_cache(hostname, records.clone());
                match records_to_ip_array(&ctx_for_resolve, records) {
                    Ok(array) => {
                        let _ = resolve.call::<_, ()>((array,));
                    }
                    Err(err) => {
                        let _ = reject.call::<_, ()>((err.to_string(),));
                    }
                }
            }
            Err(err) => {
                let _ = reject.call::<_, ()>((err.to_string(),));
            }
        }
    });

    Ok(promise)
}

fn resolver_lookup_host<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, DnsResolver>>,
    hostname: String,
) -> Result<Promise<'js>> {
    let resolver = this.0.try_borrow()?.clone();
    validate_hostname(&ctx, &hostname)?;

    if let Some(records) = resolver.cached(&hostname) {
        let (promise, resolve, _reject) = Promise::new(&ctx)?;
        let array = records_to_host_array(&ctx, records)?;
        resolve.call::<_, ()>((array,))?;
        return Ok(promise);
    }

    let (promise, resolve, reject) = Promise::new(&ctx)?;
    let ctx_for_resolve = ctx.clone();
    ctx.spawn(async move {
        match lookup_records((*resolver.resolver).clone(), hostname.clone()).await {
            Ok(records) => {
                resolver.store_cache(hostname, records.clone());
                match records_to_host_array(&ctx_for_resolve, records) {
                    Ok(array) => {
                        let _ = resolve.call::<_, ()>((array,));
                    }
                    Err(err) => {
                        let _ = reject.call::<_, ()>((err.to_string(),));
                    }
                }
            }
            Err(err) => {
                let _ = reject.call::<_, ()>((err.to_string(),));
            }
        }
    });

    Ok(promise)
}

fn resolver_lookup_ip<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, DnsResolver>>,
    hostname: String,
) -> Result<Promise<'js>> {
    let resolver = this.0.try_borrow()?.clone();
    resolver_lookup_records(ctx, resolver, hostname)
}

fn resolver_clear_cache<'js>(
    this: This<Class<'js, DnsResolver>>,
    hostname: Opt<String>,
) -> Result<bool> {
    let resolver = this.0.try_borrow()?;
    let mut cache = resolver.cache.lock().unwrap_or_else(|err| err.into_inner());
    match hostname.0 {
        Some(hostname) => {
            resolver
                .resolver
                .clear_lookup_cache(hostname.clone(), RecordType::A);
            resolver
                .resolver
                .clear_lookup_cache(hostname.clone(), RecordType::AAAA);
            Ok(cache.remove(&hostname).is_some())
        }
        None => {
            cache.clear();
            resolver.resolver.clear_cache();
            Ok(true)
        }
    }
}

fn resolver_cache_enabled<'js>(this: This<Class<'js, DnsResolver>>) -> Result<bool> {
    Ok(this.0.try_borrow()?.cache_enabled)
}

impl<'js> JsClass<'js> for DnsResolver {
    const NAME: &'static str = "DnsResolver";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set(
            "lookupHost",
            Function::new(ctx.clone(), resolver_lookup_host)?,
        )?;
        proto.set("lookupIp", Function::new(ctx.clone(), resolver_lookup_ip)?)?;
        proto.set(
            "clearCache",
            Function::new(ctx.clone(), resolver_clear_cache)?,
        )?;
        proto.set(
            "cacheEnabled",
            Function::new(ctx.clone(), resolver_cache_enabled)?,
        )?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<DnsResolver, _, _>(ctx.clone(), resolver_ctor)?;
        Ok(Some(constr))
    }
}

pub(crate) fn lookup_host<'js>(ctx: Ctx<'js>, hostname: String) -> Result<Promise<'js>> {
    validate_hostname(&ctx, &hostname)?;
    let resolver = default_resolver().map_err(|err| Exception::throw_message(&ctx, &err))?;
    let (promise, resolve, reject) = Promise::new(&ctx)?;
    let ctx_for_resolve = ctx.clone();
    ctx.spawn(async move {
        match lookup_records(resolver, hostname).await {
            Ok(records) => match records_to_host_array(&ctx_for_resolve, records) {
                Ok(array) => {
                    let _ = resolve.call::<_, ()>((array,));
                }
                Err(err) => {
                    let _ = reject.call::<_, ()>((err.to_string(),));
                }
            },
            Err(err) => {
                let _ = reject.call::<_, ()>((err.to_string(),));
            }
        }
    });
    Ok(promise)
}

pub(crate) fn lookup_ip<'js>(ctx: Ctx<'js>, hostname: String) -> Result<Promise<'js>> {
    let resolver = DnsResolver::new(false).map_err(|err| Exception::throw_message(&ctx, &err))?;
    resolver_lookup_records(ctx, resolver, hostname)
}

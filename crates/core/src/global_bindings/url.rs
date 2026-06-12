use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::This,
    Class, Constructor, Ctx, FromJs, IntoJs, Object, Result, Value,
};
use url::Url;

// --- URL ---

#[derive(Clone, Debug)]
pub struct UrlWrapper {
    url: Url,
}

impl UrlWrapper {
    fn new(url: &str, base: Option<&str>) -> Result<Self> {
        let parsed = match base {
            Some(b) => Url::parse(b)
                .and_then(|base| base.join(url))
                .map_err(|_| rquickjs::Error::Exception)?,
            None => Url::parse(url).map_err(|_| rquickjs::Error::Exception)?,
        };
        Ok(Self { url: parsed })
    }
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for UrlWrapper {
    type Changed<'to> = UrlWrapper;
}

impl<'js> Trace<'js> for UrlWrapper {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for UrlWrapper {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> FromJs<'js> for UrlWrapper {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Ok(Class::<UrlWrapper>::from_js(ctx, value)?
            .try_borrow()?
            .clone())
    }
}

fn url_get_href<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(this.try_borrow()?.url.as_str().to_string())
}

fn url_get_protocol<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(format!("{}:", this.try_borrow()?.url.scheme()))
}

fn url_get_host<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    let u = this.try_borrow()?;
    Ok(u.url
        .host_str()
        .map(|h| match u.url.port() {
            Some(p) => format!("{}:{}", h, p),
            None => h.to_string(),
        })
        .unwrap_or_default())
}

fn url_get_hostname<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(this.try_borrow()?.url.host_str().unwrap_or("").to_string())
}

fn url_get_port<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(this
        .try_borrow()?
        .url
        .port()
        .map(|p| p.to_string())
        .unwrap_or_default())
}

fn url_get_pathname<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(this.try_borrow()?.url.path().to_string())
}

fn url_get_search<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    let inner = this.try_borrow()?;
    let s = inner.url.query().unwrap_or("");
    Ok(if s.is_empty() {
        "".to_string()
    } else {
        format!("?{}", s)
    })
}

fn url_get_hash<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    let inner = this.try_borrow()?;
    let f = inner.url.fragment().unwrap_or("");
    Ok(if f.is_empty() {
        "".to_string()
    } else {
        format!("#{}", f)
    })
}

fn url_get_username<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(this.try_borrow()?.url.username().to_string())
}

fn url_get_password<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(this.try_borrow()?.url.password().unwrap_or("").to_string())
}

fn url_get_origin<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(this.try_borrow()?.url.origin().ascii_serialization())
}

fn url_to_string<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(this.try_borrow()?.url.as_str().to_string())
}

fn url_to_json<'js>(this: This<Class<'js, UrlWrapper>>) -> Result<String> {
    Ok(this.try_borrow()?.url.as_str().to_string())
}

fn url_get_search_params<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, UrlWrapper>>,
) -> Result<Value<'js>> {
    let query = this.try_borrow()?.url.query().unwrap_or("").to_string();
    let params = UrlSearchParams::new(Some(query));
    Class::instance(ctx.clone(), params).into_js(&ctx)
}

impl<'js> JsClass<'js> for UrlWrapper {
    const NAME: &'static str = "URL";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;

        proto.set("href", rquickjs::Function::new(ctx.clone(), url_get_href)?)?;
        proto.set(
            "protocol",
            rquickjs::Function::new(ctx.clone(), url_get_protocol)?,
        )?;
        proto.set("host", rquickjs::Function::new(ctx.clone(), url_get_host)?)?;
        proto.set(
            "hostname",
            rquickjs::Function::new(ctx.clone(), url_get_hostname)?,
        )?;
        proto.set("port", rquickjs::Function::new(ctx.clone(), url_get_port)?)?;
        proto.set(
            "pathname",
            rquickjs::Function::new(ctx.clone(), url_get_pathname)?,
        )?;
        proto.set(
            "search",
            rquickjs::Function::new(ctx.clone(), url_get_search)?,
        )?;
        proto.set("hash", rquickjs::Function::new(ctx.clone(), url_get_hash)?)?;
        proto.set(
            "username",
            rquickjs::Function::new(ctx.clone(), url_get_username)?,
        )?;
        proto.set(
            "password",
            rquickjs::Function::new(ctx.clone(), url_get_password)?,
        )?;
        proto.set(
            "origin",
            rquickjs::Function::new(ctx.clone(), url_get_origin)?,
        )?;
        proto.set(
            "toString",
            rquickjs::Function::new(ctx.clone(), url_to_string)?,
        )?;
        proto.set("toJSON", rquickjs::Function::new(ctx.clone(), url_to_json)?)?;
        proto.set(
            "searchParams",
            rquickjs::Function::new(ctx.clone(), url_get_search_params)?,
        )?;

        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<UrlWrapper, _, _>(
            ctx.clone(),
            |url: String, base: rquickjs::function::Opt<String>| {
                UrlWrapper::new(&url, base.0.as_deref())
            },
        )?;
        Ok(Some(constr))
    }
}

// --- URLSearchParams ---

#[derive(Clone, Debug)]
pub struct UrlSearchParams {
    pairs: Vec<(String, String)>,
}

impl UrlSearchParams {
    fn new(init: Option<String>) -> Self {
        let mut pairs = Vec::new();
        if let Some(s) = init {
            for part in s.trim_start_matches('?').split('&') {
                if let Some((k, v)) = part.split_once('=') {
                    pairs.push((
                        percent_decode_str(k).decode_utf8_lossy().to_string(),
                        percent_decode_str(v).decode_utf8_lossy().to_string(),
                    ));
                } else if !part.is_empty() {
                    pairs.push((
                        percent_decode_str(part).decode_utf8_lossy().to_string(),
                        String::new(),
                    ));
                }
            }
        }
        Self { pairs }
    }

    fn to_query_string(&self) -> String {
        self.pairs
            .iter()
            .map(|(k, v)| {
                let enc_k = utf8_percent_encode(k, NON_ALPHANUMERIC).to_string();
                if v.is_empty() {
                    enc_k
                } else {
                    format!("{}={}", enc_k, utf8_percent_encode(v, NON_ALPHANUMERIC))
                }
            })
            .collect::<Vec<_>>()
            .join("&")
    }
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for UrlSearchParams {
    type Changed<'to> = UrlSearchParams;
}

impl<'js> Trace<'js> for UrlSearchParams {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for UrlSearchParams {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> FromJs<'js> for UrlSearchParams {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Ok(Class::<UrlSearchParams>::from_js(ctx, value)?
            .try_borrow()?
            .clone())
    }
}

fn usp_append<'js>(
    this: This<Class<'js, UrlSearchParams>>,
    key: String,
    value: String,
) -> Result<()> {
    this.try_borrow_mut()?.pairs.push((key, value));
    Ok(())
}

fn usp_delete<'js>(this: This<Class<'js, UrlSearchParams>>, key: String) -> Result<()> {
    this.try_borrow_mut()?.pairs.retain(|(k, _)| k != &key);
    Ok(())
}

fn usp_get<'js>(this: This<Class<'js, UrlSearchParams>>, key: String) -> Result<Option<String>> {
    let inner = this.try_borrow()?;
    Ok(inner
        .pairs
        .iter()
        .find(|(k, _)| k == &key)
        .map(|(_, v)| v.clone()))
}

fn usp_get_all<'js>(this: This<Class<'js, UrlSearchParams>>, key: String) -> Result<Vec<String>> {
    let inner = this.try_borrow()?;
    Ok(inner
        .pairs
        .iter()
        .filter(|(k, _)| k == &key)
        .map(|(_, v)| v.clone())
        .collect())
}

fn usp_has<'js>(this: This<Class<'js, UrlSearchParams>>, key: String) -> Result<bool> {
    let inner = this.try_borrow()?;
    Ok(inner.pairs.iter().any(|(k, _)| k == &key))
}

fn usp_set<'js>(this: This<Class<'js, UrlSearchParams>>, key: String, value: String) -> Result<()> {
    let mut inner = this.try_borrow_mut()?;
    inner.pairs.retain(|(k, _)| k != &key);
    inner.pairs.push((key, value));
    Ok(())
}

fn usp_to_string<'js>(this: This<Class<'js, UrlSearchParams>>) -> Result<String> {
    Ok(this.try_borrow()?.to_query_string())
}

fn usp_size<'js>(this: This<Class<'js, UrlSearchParams>>) -> Result<usize> {
    Ok(this.try_borrow()?.pairs.len())
}

fn usp_sort<'js>(this: This<Class<'js, UrlSearchParams>>) -> Result<()> {
    this.try_borrow_mut()?.pairs.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(())
}

fn usp_constructor(init: rquickjs::function::Opt<String>) -> Result<UrlSearchParams> {
    Ok(UrlSearchParams::new(init.0))
}

impl<'js> JsClass<'js> for UrlSearchParams {
    const NAME: &'static str = "URLSearchParams";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;

        proto.set("append", rquickjs::Function::new(ctx.clone(), usp_append)?)?;
        proto.set("delete", rquickjs::Function::new(ctx.clone(), usp_delete)?)?;
        proto.set("get", rquickjs::Function::new(ctx.clone(), usp_get)?)?;
        proto.set("getAll", rquickjs::Function::new(ctx.clone(), usp_get_all)?)?;
        proto.set("has", rquickjs::Function::new(ctx.clone(), usp_has)?)?;
        proto.set("set", rquickjs::Function::new(ctx.clone(), usp_set)?)?;
        proto.set(
            "toString",
            rquickjs::Function::new(ctx.clone(), usp_to_string)?,
        )?;
        proto.set("size", rquickjs::Function::new(ctx.clone(), usp_size)?)?;
        proto.set("sort", rquickjs::Function::new(ctx.clone(), usp_sort)?)?;

        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<UrlSearchParams, _, _>(ctx.clone(), usp_constructor)?;
        Ok(Some(constr))
    }
}

pub fn init(ctx: &Ctx) -> Result<()> {
    Class::<UrlWrapper>::define(&ctx.globals())?;
    Class::<UrlSearchParams>::define(&ctx.globals())?;

    // Convert URL prototype methods to getters
    ctx.eval::<(), _>(r#"
        (function() {
            const wrapGetter = (proto, name) => {
                const fn = proto[name];
                Object.defineProperty(proto, name, {
                    get: function() { return fn.call(this); },
                    enumerable: true,
                    configurable: true
                });
            };
            ['href','protocol','host','hostname','port','pathname','search','hash','username','password','origin','searchParams']
                .forEach(p => wrapGetter(URL.prototype, p));
            wrapGetter(URLSearchParams.prototype, 'size');

            function uspEntries(self) {
                var str = self.toString();
                if (!str) return [];
                return str.split('&').map(function(pair) {
                    var eq = pair.indexOf('=');
                    return eq >= 0
                        ? [decodeURIComponent(pair.slice(0, eq)), decodeURIComponent(pair.slice(eq + 1))]
                        : [decodeURIComponent(pair), ''];
                });
            }
            URLSearchParams.prototype.forEach = function(callback, thisArg) {
                uspEntries(this).forEach(function(entry) {
                    callback.call(thisArg, entry[1], entry[0], this);
                }, this);
            };
            URLSearchParams.prototype.entries = function() {
                return uspEntries(this)[Symbol.iterator]();
            };
            URLSearchParams.prototype.keys = function() {
                return uspEntries(this).map(function(p) { return p[0]; })[Symbol.iterator]();
            };
            URLSearchParams.prototype.values = function() {
                return uspEntries(this).map(function(p) { return p[1]; })[Symbol.iterator]();
            };
            URLSearchParams.prototype[Symbol.iterator] = function() {
                return uspEntries(this)[Symbol.iterator]();
            };
        })();
    "#)?;

    ctx.eval::<(), _>(
        r#"
        (function() {
            class URLPattern {
                constructor(input, baseURL) {
                    let pattern = {};
                    if (typeof input === "string") {
                        if (baseURL) {
                            const base = new URL(baseURL);
                            pattern.pathname = input;
                            pattern.protocol = base.protocol;
                            pattern.hostname = base.hostname;
                            pattern.port = base.port;
                        } else {
                            pattern.pathname = input;
                        }
                    } else if (input && typeof input === "object") {
                        pattern = { ...input };
                    }
                    this.protocol = pattern.protocol || "*";
                    this.username = pattern.username || "*";
                    this.password = pattern.password || "*";
                    this.hostname = pattern.hostname || "*";
                    this.port = pattern.port || "*";
                    this.pathname = pattern.pathname || "/*";
                    this.search = pattern.search || "*";
                    this.hash = pattern.hash || "*";
                }

                _makeRegex(pattern) {
                    let re = pattern
                        .replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
                        .replace(/\\\*/g, ".*")
                        .replace(/:([a-zA-Z0-9_]+)/g, "(?<$1>[^/]+)");
                    return new RegExp("^" + re + "$");
                }

                test(input) {
                    const url = typeof input === "string" ? new URL(input) : input;
                    return this._makeRegex(this.protocol).test(url.protocol) &&
                           this._makeRegex(this.hostname).test(url.hostname) &&
                           this._makeRegex(this.port).test(url.port) &&
                           this._makeRegex(this.pathname).test(url.pathname) &&
                           this._makeRegex(this.search).test(url.search) &&
                           this._makeRegex(this.hash).test(url.hash);
                }

                exec(input) {
                    const url = typeof input === "string" ? new URL(input) : input;
                    const execPart = (pattern, value) => {
                        const match = this._makeRegex(pattern).exec(value);
                        return {
                            input: value,
                            groups: match ? (match.groups || {}) : {}
                        };
                    };
                    return {
                        input: String(input),
                        protocol: execPart(this.protocol, url.protocol),
                        username: { input: url.username, groups: {} },
                        password: { input: url.password, groups: {} },
                        hostname: execPart(this.hostname, url.hostname),
                        port: execPart(this.port, url.port),
                        pathname: execPart(this.pathname, url.pathname),
                        search: execPart(this.search, url.search),
                        hash: execPart(this.hash, url.hash),
                    };
                }
            }
            globalThis.URLPattern = URLPattern;

            URL.canParse = function(url, base) {
                try {
                    if (base === undefined) {
                        new URL(url);
                    } else {
                        new URL(url, base);
                    }
                    return true;
                } catch {
                    return false;
                }
            };
            URL.parse = function(url, base) {
                try {
                    if (base === undefined) {
                        return new URL(url);
                    } else {
                        return new URL(url, base);
                    }
                } catch {
                    return null;
                }
            };
        })();
    "#,
    )?;

    Ok(())
}

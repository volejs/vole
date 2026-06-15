use std::{cell::RefCell, rc::Rc};

use rquickjs::{
    Class, Coerced, Ctx, Exception, FromJs, IntoJs, Null, Object, Result, Value,
    atom::PredefinedAtom, class::Trace, function::Opt,
};
use url::{Url, quirks};
use vole_core::string::get_lossy_string;

use super::url_search_params::URLSearchParams;

#[derive(Clone, Trace, rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct URL<'js> {
    #[qjs(skip_trace)]
    url: Rc<RefCell<Url>>,
    search_params: Class<'js, URLSearchParams>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> URL<'js> {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, input: Value<'js>, base: Opt<Value<'js>>) -> Result<Self> {
        let input: Result<String> = if input.is_string() {
            get_lossy_string(input.clone())
        } else {
            Coerced::<String>::from_js(&ctx, input.clone()).map(|c| c.0)
        };

        if let Some(base) = base.into_inner() {
            if let Some(base) = base.as_string() {
                if let Ok(base) = base.to_string() {
                    let base_url: Url = base
                        .parse()
                        .map_err(|_| Exception::throw_type(&ctx, "Invalid base URL"))?;
                    let base_url = super::preserve_file_url_host(&base, base_url);
                    if let Ok(input) = input {
                        let mut joined = base_url
                            .join(input.as_str())
                            .map_err(|_| Exception::throw_type(&ctx, "Invalid URL"))?;
                        super::restore_file_url_host(&base_url, &mut joined);
                        return Self::from_url(ctx, joined);
                    }
                    return Self::from_str(ctx, &base);
                }
            }
        }

        if let Ok(input) = input {
            Self::from_str(ctx, input.as_str())
        } else {
            Err(Exception::throw_message(&ctx, "Invalid URL"))
        }
    }

    #[qjs(get)]
    pub fn hash(&self) -> String {
        quirks::hash(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "hash")]
    pub fn set_hash(&mut self, hash: String) -> String {
        self.before_mutation();
        quirks::set_hash(&mut self.url.borrow_mut(), &hash);
        hash
    }

    #[qjs(get)]
    pub fn host(&self) -> String {
        quirks::host(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "host")]
    pub fn set_host(&mut self, host: Coerced<String>) -> String {
        self.before_mutation();
        let _ = quirks::set_host(&mut self.url.borrow_mut(), &host);
        host.0
    }

    #[qjs(get)]
    pub fn hostname(&self) -> String {
        quirks::hostname(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "hostname")]
    pub fn set_hostname(&mut self, hostname: Coerced<String>) -> String {
        self.before_mutation();
        let _ = quirks::set_hostname(&mut self.url.borrow_mut(), hostname.as_str());
        super::strip_path_sentinel(&mut self.url.borrow_mut());
        hostname.0
    }

    #[qjs(get)]
    pub fn href(&self) -> String {
        quirks::href(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "href")]
    pub fn set_href(&mut self, href: String) -> String {
        self.before_mutation();
        let _ = quirks::set_href(&mut self.url.borrow_mut(), &href);
        href
    }

    #[qjs(get)]
    pub fn origin(&self) -> String {
        let url = self.url.borrow();
        if url.scheme() == "blob" {
            return match Url::parse(url.path()) {
                Ok(inner) if matches!(inner.scheme(), "http" | "https") => quirks::origin(&inner),
                _ => "null".into(),
            };
        }
        quirks::origin(&url)
    }

    #[qjs(get)]
    pub fn password(&self) -> String {
        quirks::password(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "password")]
    pub fn set_password(&mut self, password: Coerced<String>) -> String {
        self.before_mutation();
        let _ = quirks::set_password(&mut self.url.borrow_mut(), &password);
        password.0
    }

    #[qjs(get)]
    pub fn pathname(&self) -> String {
        quirks::pathname(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "pathname")]
    pub fn set_pathname(&mut self, pathname: Coerced<String>) -> String {
        self.before_mutation();
        quirks::set_pathname(&mut self.url.borrow_mut(), pathname.as_str());
        if pathname.0.is_empty() {
            super::erase_empty_host_path(&mut self.url.borrow_mut());
        }
        pathname.0
    }

    #[qjs(get)]
    pub fn port(&self) -> String {
        quirks::port(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "port")]
    pub fn set_port(&mut self, ctx: Ctx<'js>, port: Value<'js>) -> Value<'js> {
        if port.is_null()
            || port.is_undefined()
            || (port.is_int() && unsafe { port.as_int().unwrap_unchecked() } < 0)
        {
            return port;
        }
        if let Ok(port_string) = Coerced::<String>::from_js(&ctx, port.clone()) {
            self.before_mutation();
            if port_string.is_empty() {
                let _ = quirks::set_port(&mut self.url.borrow_mut(), "");
            } else {
                let stripped: String = port_string
                    .chars()
                    .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
                    .collect();
                if !stripped.is_empty() {
                    let _ = quirks::set_port(&mut self.url.borrow_mut(), &stripped);
                }
            }
        }
        port
    }

    #[qjs(get)]
    pub fn protocol(&self) -> String {
        quirks::protocol(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "protocol")]
    pub fn set_protocol(&mut self, protocol: Coerced<String>) -> String {
        self.before_mutation();
        let _ = quirks::set_protocol(&mut self.url.borrow_mut(), &protocol);
        protocol.0
    }

    #[qjs(get)]
    pub fn search(&self) -> String {
        quirks::search(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "search")]
    pub fn set_search(&mut self, search: Coerced<String>) -> String {
        self.before_mutation();
        quirks::set_search(&mut self.url.borrow_mut(), &search);
        search.0
    }

    #[qjs(get, rename = "searchParams")]
    pub fn search_params(&self) -> &Value<'js> {
        self.search_params.as_value()
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        stringify!(URL)
    }

    #[qjs(get)]
    pub fn username(&self) -> String {
        quirks::username(&self.url.borrow()).to_string()
    }

    #[qjs(set, rename = "username")]
    pub fn set_username(&mut self, username: Coerced<String>) -> String {
        self.before_mutation();
        let _ = quirks::set_username(&mut self.url.borrow_mut(), &username);
        username.0
    }

    #[qjs(static, rename = "canParse")]
    pub fn can_parse(ctx: Ctx<'js>, input: Value<'js>, base: Opt<Value<'js>>) -> bool {
        Self::new(ctx, input, base).is_ok()
    }

    #[qjs(static)]
    pub fn parse(ctx: Ctx<'js>, input: Value<'js>, base: Opt<Value<'js>>) -> Result<Value<'js>> {
        Self::new(ctx.clone(), input, base)
            .map_or_else(|_| Null.into_js(&ctx), |instance| instance.into_js(&ctx))
    }

    #[qjs(rename = PredefinedAtom::ToJSON)]
    pub fn to_json(&self) -> String {
        self.to_string()
    }

    pub fn to_string(&self) -> String {
        self.href()
    }
}

impl<'js> URL<'js> {
    pub fn from_str(ctx: Ctx<'js>, input: &str) -> Result<Self> {
        let mut url: Url = input
            .parse()
            .map_err(|_| Exception::throw_type(&ctx, "Invalid URL"))?;
        super::normalize_windows_drive_letter(&mut url);
        super::convert_trailing_space(&mut url);
        Self::build(ctx, url)
    }

    pub fn from_url(ctx: Ctx<'js>, mut url: Url) -> Result<Self> {
        super::normalize_windows_drive_letter(&mut url);
        super::convert_trailing_space(&mut url);
        Self::build(ctx, url)
    }

    pub fn is_valid(input: &str) -> bool {
        input.parse::<Url>().is_ok()
    }

    fn build(ctx: Ctx<'js>, url: Url) -> Result<Self> {
        let shared = Rc::new(RefCell::new(url));
        let search_params = Class::instance(ctx, URLSearchParams::from_url(&shared))?;
        Ok(Self {
            url: shared,
            search_params,
        })
    }

    fn before_mutation(&mut self) {
        super::convert_trailing_space(&mut self.url.borrow_mut());
    }

    pub(crate) fn inner_url(&self) -> std::cell::Ref<'_, Url> {
        self.url.borrow()
    }
}

pub fn url_to_http_options<'js>(ctx: Ctx<'js>, url: Class<'js, URL<'js>>) -> Result<Object<'js>> {
    let obj = Object::new(ctx)?;
    let url = url.borrow();

    let port = url.port();
    let username = url.username();
    let search = url.search();
    let hash = url.inner_url().fragment().unwrap_or("").to_string();

    obj.set("protocol", url.protocol())?;
    obj.set("hostname", url.hostname())?;

    if !hash.is_empty() {
        obj.set("hash", hash)?;
    }

    let pathname = url.pathname();
    let path = [pathname.as_str(), search.as_str()].concat();
    if !search.is_empty() {
        obj.set("search", search)?;
    }
    obj.set("pathname", pathname)?;
    obj.set("path", path)?;
    obj.set("href", url.href())?;

    if !username.is_empty() {
        obj.set("auth", [username, url.password()].join(":"))?;
    }

    if !port.is_empty() {
        obj.set("port", port)?;
    }

    Ok(obj)
}

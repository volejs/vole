use rquickjs::{
    atom::PredefinedAtom,
    class::Trace,
    function::Opt,
    Array, Coerced, Ctx, FromJs, IntoJs, Null, Object, Result, Value,
};
use url::Url;
use urlpattern::{
    UrlPattern as NativeUrlPattern, UrlPatternComponentResult, UrlPatternInit,
    UrlPatternMatchInput, UrlPatternOptions,
};
use vole_core::webidl::to_dom_string;

#[derive(Trace, rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct URLPattern {
    #[qjs(skip_trace)]
    inner: NativeUrlPattern,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> URLPattern {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'js>,
        input: Value<'js>,
        base_url_or_options: Opt<Value<'js>>,
        options: Opt<Value<'js>>,
    ) -> Result<Self> {
        let (init, options) = constructor_args_to_init_and_options(
            &ctx,
            input,
            base_url_or_options.0,
            options.0,
        )?;
        let inner = NativeUrlPattern::parse(init, options)
            .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))?;
        Ok(Self { inner })
    }

    #[qjs(get)]
    pub fn protocol(&self) -> String {
        self.inner.protocol().to_string()
    }

    #[qjs(get)]
    pub fn username(&self) -> String {
        self.inner.username().to_string()
    }

    #[qjs(get)]
    pub fn password(&self) -> String {
        self.inner.password().to_string()
    }

    #[qjs(get)]
    pub fn hostname(&self) -> String {
        self.inner.hostname().to_string()
    }

    #[qjs(get)]
    pub fn port(&self) -> String {
        self.inner.port().to_string()
    }

    #[qjs(get)]
    pub fn pathname(&self) -> String {
        self.inner.pathname().to_string()
    }

    #[qjs(get)]
    pub fn search(&self) -> String {
        self.inner.search().to_string()
    }

    #[qjs(get)]
    pub fn hash(&self) -> String {
        self.inner.hash().to_string()
    }

    #[qjs(get, rename = "hasRegExpGroups")]
    pub fn has_regexp_groups(&self) -> bool {
        self.inner.has_regexp_groups()
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        "URLPattern"
    }

    pub fn test(
        &self,
        ctx: Ctx<'js>,
        input: Value<'js>,
        base_url: Opt<Value<'js>>,
    ) -> Result<bool> {
        let parsed = match_args_to_input(&ctx, input, base_url.0)?;
        self.inner
            .test(parsed.input)
            .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))
    }

    pub fn exec(
        &self,
        ctx: Ctx<'js>,
        input: Value<'js>,
        base_url: Opt<Value<'js>>,
    ) -> Result<Value<'js>> {
        let parsed = match_args_to_input(&ctx, input, base_url.0)?;
        let Some(result) = self
            .inner
            .exec(parsed.input)
            .map_err(|err| rquickjs::Exception::throw_type(&ctx, &err.to_string()))?
        else {
            return Null.into_js(&ctx);
        };

        let object = Object::new(ctx.clone())?;
        let inputs = Array::new(ctx.clone())?;
        for (idx, input) in parsed.inputs.into_iter().enumerate() {
            inputs.set(idx, input)?;
        }
        object.set("inputs", inputs)?;
        object.set("protocol", component_result(&ctx, result.protocol)?)?;
        object.set("username", component_result(&ctx, result.username)?)?;
        object.set("password", component_result(&ctx, result.password)?)?;
        object.set("hostname", component_result(&ctx, result.hostname)?)?;
        object.set("port", component_result(&ctx, result.port)?)?;
        object.set("pathname", component_result(&ctx, result.pathname)?)?;
        object.set("search", component_result(&ctx, result.search)?)?;
        object.set("hash", component_result(&ctx, result.hash)?)?;
        Ok(object.into_value())
    }
}

struct ParsedMatchInput {
    input: UrlPatternMatchInput,
    inputs: Vec<ValueInput>,
}

#[derive(Clone)]
enum ValueInput {
    String(String),
    Object,
}

impl<'js> IntoJs<'js> for ValueInput {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        match self {
            Self::String(value) => value.into_js(ctx),
            Self::Object => "[object Object]".into_js(ctx),
        }
    }
}

fn constructor_args_to_init_and_options<'js>(
    ctx: &Ctx<'js>,
    input: Value<'js>,
    base_url_or_options: Option<Value<'js>>,
    options: Option<Value<'js>>,
) -> Result<(UrlPatternInit, UrlPatternOptions)> {
    if input.is_string() {
        let pattern = to_dom_string(&input)?;
        let base_url = match base_url_or_options.as_ref() {
            Some(value) if !value.is_undefined() && !value.is_null() && !value.as_object().map(is_options_object).unwrap_or(false) => {
                Some(parse_url(ctx, &to_dom_string(value)?)?)
            },
            _ => None,
        };
        let options_value = options.or_else(|| {
            base_url_or_options.filter(|value| value.as_object().map(is_options_object).unwrap_or(false))
        });
        let init = UrlPatternInit::parse_constructor_string::<regex::Regex>(&pattern, base_url)
            .map_err(|err| rquickjs::Exception::throw_type(ctx, &err.to_string()))?;
        return Ok((init, parse_options(ctx, options_value.as_ref())?));
    }

    let Some(object) = input.as_object() else {
        return Err(rquickjs::Exception::throw_type(
            ctx,
            "URLPattern input must be a string or an object",
        ));
    };

    let init = init_from_object(ctx, object)?;
    let options_value = base_url_or_options.or(options);
    Ok((init, parse_options(ctx, options_value.as_ref())?))
}

fn match_args_to_input<'js>(
    ctx: &Ctx<'js>,
    input: Value<'js>,
    base_url: Option<Value<'js>>,
) -> Result<ParsedMatchInput> {
    if input.is_string() {
        let input_string = to_dom_string(&input)?;
        let url = match base_url.as_ref() {
            Some(base_url) if !base_url.is_undefined() => {
                let base = parse_url(ctx, &to_dom_string(&base_url)?)?;
                base.join(&input_string)
                    .map_err(|err| rquickjs::Exception::throw_type(ctx, &err.to_string()))?
            },
            _ => parse_url(ctx, &input_string)?,
        };
        let mut inputs = vec![ValueInput::String(input_string)];
        if let Some(base_url) = base_url.as_ref() {
            if !base_url.is_undefined() {
                inputs.push(ValueInput::String(to_dom_string(&base_url)?));
            }
        }
        return Ok(ParsedMatchInput {
            input: UrlPatternMatchInput::Url(url),
            inputs,
        });
    }

    let Some(object) = input.as_object() else {
        return Err(rquickjs::Exception::throw_type(
            ctx,
            "URLPattern input must be a string or an object",
        ));
    };

    Ok(ParsedMatchInput {
        input: UrlPatternMatchInput::Init(init_from_object(ctx, object)?),
        inputs: vec![ValueInput::Object],
    })
}

fn init_from_object<'js>(ctx: &Ctx<'js>, object: &Object<'js>) -> Result<UrlPatternInit> {
    Ok(UrlPatternInit {
        protocol: optional_component(ctx, object, "protocol")?,
        username: optional_component(ctx, object, "username")?,
        password: optional_component(ctx, object, "password")?,
        hostname: optional_component(ctx, object, "hostname")?,
        port: optional_component(ctx, object, "port")?,
        pathname: optional_component(ctx, object, "pathname")?,
        search: optional_component(ctx, object, "search")?,
        hash: optional_component(ctx, object, "hash")?,
        base_url: optional_component(ctx, object, "baseURL")?
            .map(|base_url| parse_url(ctx, &base_url))
            .transpose()?,
    })
}

fn optional_component<'js>(
    ctx: &Ctx<'js>,
    object: &Object<'js>,
    name: &str,
) -> Result<Option<String>> {
    match object.get::<_, Value>(name) {
        Ok(value) if !value.is_undefined() => Ok(Some(Coerced::<String>::from_js(ctx, value)?.0)),
        _ => Ok(None),
    }
}

fn parse_options<'js>(ctx: &Ctx<'js>, value: Option<&Value<'js>>) -> Result<UrlPatternOptions> {
    let Some(value) = value else {
        return Ok(UrlPatternOptions::default());
    };
    if value.is_undefined() || value.is_null() {
        return Ok(UrlPatternOptions::default());
    }
    let Some(object) = value.as_object() else {
        return Err(rquickjs::Exception::throw_type(
            ctx,
            "URLPattern options must be an object",
        ));
    };
    Ok(UrlPatternOptions {
        ignore_case: object.get("ignoreCase").unwrap_or(false),
        regex_syntax: Default::default(),
    })
}

fn is_options_object(object: &Object<'_>) -> bool {
    object.contains_key("ignoreCase").unwrap_or(false)
}

fn parse_url<'js>(ctx: &Ctx<'js>, input: &str) -> Result<Url> {
    Url::parse(input).map_err(|err| rquickjs::Exception::throw_type(ctx, &err.to_string()))
}

fn component_result<'js>(
    ctx: &Ctx<'js>,
    result: UrlPatternComponentResult,
) -> Result<Object<'js>> {
    let object = Object::new(ctx.clone())?;
    object.set("input", result.input)?;
    let groups = Object::new(ctx.clone())?;
    for (key, value) in result.groups {
        match value {
            Some(value) => groups.set(key, value)?,
            None => groups.set(key, Value::new_undefined(ctx.clone()))?,
        }
    }
    object.set("groups", groups)?;
    Ok(object)
}

use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::This,
    Class, Constructor, Ctx, Function, Object, Result, Value,
};

#[derive(Clone, Debug)]
pub struct DomException {
    message: String,
    name: String,
    code: u16,
}

impl DomException {
    fn new(message: Option<String>, name: Option<String>) -> Self {
        let name = name.unwrap_or_else(|| "Error".to_string());
        let code = code_for_name(&name);
        Self {
            message: message.unwrap_or_default(),
            name,
            code,
        }
    }
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for DomException {
    type Changed<'to> = DomException;
}

impl<'js> Trace<'js> for DomException {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> rquickjs::IntoJs<'js> for DomException {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn code_for_name(name: &str) -> u16 {
    match name {
        "IndexSizeError" => 1,
        "HierarchyRequestError" => 3,
        "WrongDocumentError" => 4,
        "InvalidCharacterError" => 5,
        "NoModificationAllowedError" => 7,
        "NotFoundError" => 8,
        "NotSupportedError" => 9,
        "InUseAttributeError" => 10,
        "InvalidStateError" => 11,
        "SyntaxError" => 12,
        "InvalidModificationError" => 13,
        "NamespaceError" => 14,
        "InvalidAccessError" => 15,
        "TypeMismatchError" => 17,
        "SecurityError" => 18,
        "NetworkError" => 19,
        "AbortError" => 20,
        "URLMismatchError" => 21,
        "QuotaExceededError" => 22,
        "TimeoutError" => 23,
        "InvalidNodeTypeError" => 24,
        "DataCloneError" => 25,
        _ => 0,
    }
}

fn dom_exception_message<'js>(this: This<Class<'js, DomException>>) -> Result<String> {
    Ok(this.try_borrow()?.message.clone())
}

fn dom_exception_name<'js>(this: This<Class<'js, DomException>>) -> Result<String> {
    Ok(this.try_borrow()?.name.clone())
}

fn dom_exception_code<'js>(this: This<Class<'js, DomException>>) -> Result<u16> {
    Ok(this.try_borrow()?.code)
}

impl<'js> JsClass<'js> for DomException {
    const NAME: &'static str = "DOMException";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;

        // Make DOMException instances instanceof Error.
        let error_ctor: Function = ctx.globals().get("Error")?;
        let error_proto: Object = error_ctor.get("prototype")?;
        proto.set_prototype(Some(&error_proto))?;

        proto.set(
            "message",
            Function::new(ctx.clone(), dom_exception_message)?,
        )?;
        proto.set("name", Function::new(ctx.clone(), dom_exception_name)?)?;
        proto.set("code", Function::new(ctx.clone(), dom_exception_code)?)?;

        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor(
            _ctx: Ctx,
            message: rquickjs::function::Opt<String>,
            name: rquickjs::function::Opt<String>,
        ) -> Result<DomException> {
            Ok(DomException::new(message.0, name.0))
        }
        let constr = Constructor::new_class::<DomException, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}

pub fn init(ctx: &Ctx) -> Result<()> {
    Class::<DomException>::define(&ctx.globals())?;

    // Convert prototype methods to getters to match the spec.
    ctx.eval::<(), _>(
        r#"
        (function() {
            const wrapGetter = (proto, name) => {
                const fn = proto[name];
                Object.defineProperty(proto, name, {
                    get: fn,
                    enumerable: true,
                    configurable: true,
                });
            };
            wrapGetter(DOMException.prototype, "message");
            wrapGetter(DOMException.prototype, "name");
            wrapGetter(DOMException.prototype, "code");
        })();
    "#,
    )?;

    Ok(())
}

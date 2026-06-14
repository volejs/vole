use rquickjs::{
    atom::PredefinedAtom,
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
    object::{Accessor, Property},
    Class, Coerced, Constructor, Ctx, Exception, Function, IntoJs, Object, Result, Value,
};
use vole_core::option::Undefined;

#[derive(Clone, Debug)]
pub struct DomException {
    message: String,
    name: String,
    stack: String,
    code: u16,
}

impl DomException {
    fn new_with_name(ctx: &Ctx<'_>, name: DomExceptionName, message: String) -> Result<Self> {
        let error_ctor: Function = ctx.globals().get("Error")?;
        let error: Object = error_ctor.call((message.clone(),))?;
        let stack = error.get::<_, String>("stack").unwrap_or_default();

        Ok(Self {
            name: name.as_str().to_string(),
            code: name.code(),
            message,
            stack,
        })
    }
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for DomException {
    type Changed<'to> = DomException;
}

impl<'js> Trace<'js> for DomException {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for DomException {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
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

fn dom_exception_stack<'js>(this: This<Class<'js, DomException>>) -> Result<String> {
    Ok(this.try_borrow()?.stack.clone())
}

fn dom_exception_constructor<'js>(
    ctx: Ctx<'js>,
    this: This<Value<'js>>,
    message: Opt<Undefined<Coerced<String>>>,
    name: Opt<Undefined<Coerced<String>>>,
) -> Result<DomException> {
    if this.0.as_function().is_none() {
        return Err(Exception::throw_type(
            &ctx,
            "Cannot call the DOMException constructor without 'new'",
        ));
    }

    let message = match message.0 {
        Some(Undefined(Some(message))) => message.0,
        _ => String::new(),
    };
    let name = match name.0 {
        Some(Undefined(Some(name))) => DomExceptionName::from(name.0),
        _ => DomExceptionName::Error,
    };

    DomException::new_with_name(&ctx, name, message)
}

impl<'js> JsClass<'js> for DomException {
    const NAME: &'static str = "DOMException";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;

        let error_ctor: Function = ctx.globals().get("Error")?;
        let error_proto: Object = error_ctor.get("prototype")?;
        proto.set_prototype(Some(&error_proto))?;

        proto.prop(
            "message",
            Accessor::from(dom_exception_message).enumerable().configurable(),
        )?;
        proto.prop(
            "name",
            Accessor::from(dom_exception_name).enumerable().configurable(),
        )?;
        proto.prop(
            "code",
            Accessor::from(dom_exception_code).enumerable().configurable(),
        )?;
        proto.prop(
            "stack",
            Accessor::from(dom_exception_stack).configurable(),
        )?;
        proto.prop(
            PredefinedAtom::SymbolToStringTag,
            Property::from("DOMException").configurable(),
        )?;

        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr =
            Constructor::new_class::<DomException, _, _>(ctx.clone(), dom_exception_constructor)?;
        Ok(Some(constr))
    }
}

#[derive(Debug)]
enum DomExceptionName {
    IndexSizeError,
    HierarchyRequestError,
    WrongDocumentError,
    InvalidCharacterError,
    NoModificationAllowedError,
    NotFoundError,
    NotSupportedError,
    InUseAttributeError,
    InvalidStateError,
    SyntaxError,
    InvalidModificationError,
    NamespaceError,
    InvalidAccessError,
    TypeMismatchError,
    SecurityError,
    NetworkError,
    AbortError,
    URLMismatchError,
    QuotaExceededError,
    TimeoutError,
    InvalidNodeTypeError,
    DataCloneError,
    EncodingError,
    NotReadableError,
    UnknownError,
    ConstraintError,
    DataError,
    TransactionInactiveError,
    ReadOnlyError,
    VersionError,
    OperationError,
    NotAllowedError,
    Error,
    Other(String),
}

impl DomExceptionName {
    fn as_str(&self) -> &str {
        match self {
            Self::IndexSizeError => "IndexSizeError",
            Self::HierarchyRequestError => "HierarchyRequestError",
            Self::WrongDocumentError => "WrongDocumentError",
            Self::InvalidCharacterError => "InvalidCharacterError",
            Self::NoModificationAllowedError => "NoModificationAllowedError",
            Self::NotFoundError => "NotFoundError",
            Self::NotSupportedError => "NotSupportedError",
            Self::InUseAttributeError => "InUseAttributeError",
            Self::InvalidStateError => "InvalidStateError",
            Self::SyntaxError => "SyntaxError",
            Self::InvalidModificationError => "InvalidModificationError",
            Self::NamespaceError => "NamespaceError",
            Self::InvalidAccessError => "InvalidAccessError",
            Self::TypeMismatchError => "TypeMismatchError",
            Self::SecurityError => "SecurityError",
            Self::NetworkError => "NetworkError",
            Self::AbortError => "AbortError",
            Self::URLMismatchError => "URLMismatchError",
            Self::QuotaExceededError => "QuotaExceededError",
            Self::TimeoutError => "TimeoutError",
            Self::InvalidNodeTypeError => "InvalidNodeTypeError",
            Self::DataCloneError => "DataCloneError",
            Self::EncodingError => "EncodingError",
            Self::NotReadableError => "NotReadableError",
            Self::UnknownError => "UnknownError",
            Self::ConstraintError => "ConstraintError",
            Self::DataError => "DataError",
            Self::TransactionInactiveError => "TransactionInactiveError",
            Self::ReadOnlyError => "ReadOnlyError",
            Self::VersionError => "VersionError",
            Self::OperationError => "OperationError",
            Self::NotAllowedError => "NotAllowedError",
            Self::Error => "Error",
            Self::Other(value) => value,
        }
    }

    fn code(&self) -> u16 {
        match self {
            Self::IndexSizeError => 1,
            Self::HierarchyRequestError => 3,
            Self::WrongDocumentError => 4,
            Self::InvalidCharacterError => 5,
            Self::NoModificationAllowedError => 7,
            Self::NotFoundError => 8,
            Self::NotSupportedError => 9,
            Self::InUseAttributeError => 10,
            Self::InvalidStateError => 11,
            Self::SyntaxError => 12,
            Self::InvalidModificationError => 13,
            Self::NamespaceError => 14,
            Self::InvalidAccessError => 15,
            Self::TypeMismatchError => 17,
            Self::SecurityError => 18,
            Self::NetworkError => 19,
            Self::AbortError => 20,
            Self::URLMismatchError => 21,
            Self::QuotaExceededError => 22,
            Self::TimeoutError => 23,
            Self::InvalidNodeTypeError => 24,
            Self::DataCloneError => 25,
            _ => 0,
        }
    }
}

impl From<String> for DomExceptionName {
    fn from(value: String) -> Self {
        match value.as_str() {
            "IndexSizeError" => Self::IndexSizeError,
            "HierarchyRequestError" => Self::HierarchyRequestError,
            "WrongDocumentError" => Self::WrongDocumentError,
            "InvalidCharacterError" => Self::InvalidCharacterError,
            "NoModificationAllowedError" => Self::NoModificationAllowedError,
            "NotFoundError" => Self::NotFoundError,
            "NotSupportedError" => Self::NotSupportedError,
            "InUseAttributeError" => Self::InUseAttributeError,
            "InvalidStateError" => Self::InvalidStateError,
            "SyntaxError" => Self::SyntaxError,
            "InvalidModificationError" => Self::InvalidModificationError,
            "NamespaceError" => Self::NamespaceError,
            "InvalidAccessError" => Self::InvalidAccessError,
            "TypeMismatchError" => Self::TypeMismatchError,
            "SecurityError" => Self::SecurityError,
            "NetworkError" => Self::NetworkError,
            "AbortError" => Self::AbortError,
            "URLMismatchError" => Self::URLMismatchError,
            "QuotaExceededError" => Self::QuotaExceededError,
            "TimeoutError" => Self::TimeoutError,
            "InvalidNodeTypeError" => Self::InvalidNodeTypeError,
            "DataCloneError" => Self::DataCloneError,
            "EncodingError" => Self::EncodingError,
            "NotReadableError" => Self::NotReadableError,
            "UnknownError" => Self::UnknownError,
            "ConstraintError" => Self::ConstraintError,
            "DataError" => Self::DataError,
            "TransactionInactiveError" => Self::TransactionInactiveError,
            "ReadOnlyError" => Self::ReadOnlyError,
            "VersionError" => Self::VersionError,
            "OperationError" => Self::OperationError,
            "NotAllowedError" => Self::NotAllowedError,
            "Error" => Self::Error,
            _ => Self::Other(value),
        }
    }
}

fn is_error<'js>(ctx: Ctx<'js>, value: Value<'js>) -> Result<bool> {
    if value.is_error() {
        return Ok(true);
    }

    let Some(obj) = value.as_object() else {
        return Ok(false);
    };

    let dom_exception: Value = ctx.globals().get(DomException::NAME)?;
    Ok(obj.is_instance_of(&dom_exception))
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();
    Class::<DomException>::define(&globals)?;

    let ctor: Object = globals.get(DomException::NAME)?;
    let proto: Object = ctor.get("prototype")?;
    define_dom_exception_constants(&ctor, &proto)?;

    let error_ctor: Function = ctx.globals().get("Error")?;
    error_ctor.set("isError", Function::new(ctx.clone(), is_error)?)?;

    Ok(())
}

fn define_dom_exception_constants<'js>(ctor: &Object<'js>, proto: &Object<'js>) -> Result<()> {
    const CONSTANTS: &[(&str, u16)] = &[
        ("INDEX_SIZE_ERR", 1),
        ("DOMSTRING_SIZE_ERR", 2),
        ("HIERARCHY_REQUEST_ERR", 3),
        ("WRONG_DOCUMENT_ERR", 4),
        ("INVALID_CHARACTER_ERR", 5),
        ("NO_DATA_ALLOWED_ERR", 6),
        ("NO_MODIFICATION_ALLOWED_ERR", 7),
        ("NOT_FOUND_ERR", 8),
        ("NOT_SUPPORTED_ERR", 9),
        ("INUSE_ATTRIBUTE_ERR", 10),
        ("INVALID_STATE_ERR", 11),
        ("SYNTAX_ERR", 12),
        ("INVALID_MODIFICATION_ERR", 13),
        ("NAMESPACE_ERR", 14),
        ("INVALID_ACCESS_ERR", 15),
        ("VALIDATION_ERR", 16),
        ("TYPE_MISMATCH_ERR", 17),
        ("SECURITY_ERR", 18),
        ("NETWORK_ERR", 19),
        ("ABORT_ERR", 20),
        ("URL_MISMATCH_ERR", 21),
        ("QUOTA_EXCEEDED_ERR", 22),
        ("TIMEOUT_ERR", 23),
        ("INVALID_NODE_TYPE_ERR", 24),
        ("DATA_CLONE_ERR", 25),
    ];

    for (name, value) in CONSTANTS {
        let property = Property::from(*value).enumerable();
        ctor.prop(*name, property)?;
        let property = Property::from(*value).enumerable();
        proto.prop(*name, property)?;
    }

    Ok(())
}

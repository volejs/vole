use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use rquickjs::{
    atom::PredefinedAtom,
    function::{Opt, This},
    Array, ArrayBuffer, Ctx, Exception, Function, Object, Result, Type, Value,
};
use vole_core::{
    error::{dom_exception_name, throw_dom_exception},
    primordials::{BasePrimordials, Primordial},
};

pub fn init(ctx: &Ctx) -> Result<()> {
    BasePrimordials::init(ctx)?;
    let globals = ctx.globals();
    globals.set(
        "structuredClone",
        Function::new(ctx.clone(), structured_clone)?,
    )?;
    Ok(())
}

fn structured_clone<'js>(
    ctx: Ctx<'js>,
    value: Value<'js>,
    options: Opt<Value<'js>>,
) -> Result<Value<'js>> {
    reject_unsupported_transfer(&ctx, options.0.as_ref())?;
    let mut state = CloneState::default();
    clone_value(&ctx, value, &mut state)
}

#[derive(Default)]
struct CloneState<'js> {
    seen: HashMap<usize, Value<'js>>,
}

fn clone_value<'js>(
    ctx: &Ctx<'js>,
    value: Value<'js>,
    state: &mut CloneState<'js>,
) -> Result<Value<'js>> {
    match value.type_of() {
        Type::Symbol | Type::Function | Type::Constructor | Type::Promise | Type::Proxy | Type::Module => {
            throw_dom_exception(
                ctx,
                dom_exception_name::DATA_CLONE,
                "Value cannot be cloned",
            )
        },
        Type::Array => clone_array(ctx, value, state),
        Type::Object | Type::Exception => clone_object_like(ctx, value, state),
        _ => Ok(value),
    }
}

fn clone_array<'js>(
    ctx: &Ctx<'js>,
    value: Value<'js>,
    state: &mut CloneState<'js>,
) -> Result<Value<'js>> {
    if let Some(existing) = seen_value(&value, state) {
        return Ok(existing);
    }

    let source = value.as_array().expect("value checked as array");
    let target = Array::new(ctx.clone())?;
    state.seen.insert(value_hash(&value), target.clone().into_value());

    for index in 0..source.len() {
        let item: Value<'js> = source.get(index)?;
        target.set(index, clone_value(ctx, item, state)?)?;
    }

    clone_string_properties(ctx, &source.clone().into_object(), &target.clone().into_object(), state)?;
    Ok(target.into_value())
}

fn clone_object_like<'js>(
    ctx: &Ctx<'js>,
    value: Value<'js>,
    state: &mut CloneState<'js>,
) -> Result<Value<'js>> {
    if let Some(existing) = seen_value(&value, state) {
        return Ok(existing);
    }

    if let Some(array_buffer) = ArrayBuffer::from_value(value.clone()) {
        let bytes = array_buffer.as_bytes().ok_or_else(|| {
            Exception::throw_type(ctx, "Cannot clone a detached ArrayBuffer")
        })?;
        return Ok(ArrayBuffer::new_copy(ctx.clone(), bytes)?.into_value());
    }

    let object = value.as_object().expect("value checked as object");
    let primordials = BasePrimordials::get(ctx)?;

    if object.is_instance_of(&primordials.constructor_date) {
        return object_constructor_clone(ctx, object, &value, state, true);
    }
    if object.is_instance_of(&primordials.constructor_regexp) {
        return object_constructor_clone(ctx, object, &value, state, true);
    }
    if object.is_instance_of(&primordials.constructor_set) {
        return clone_set(ctx, object, &value, state);
    }
    if object.is_instance_of(&primordials.constructor_map) {
        return clone_map(ctx, object, &value, state);
    }
    if primordials
        .function_array_buffer_is_view
        .call::<_, bool>((value.clone(),))?
    {
        return clone_array_buffer_view(ctx, object, &value, state);
    }

    if value.is_error() || is_dom_exception(ctx, object)? {
        return clone_error_like(ctx, object, &value, state);
    }

    clone_plain_object(ctx, object, &value, state)
}

fn clone_plain_object<'js>(
    ctx: &Ctx<'js>,
    source: &Object<'js>,
    original: &Value<'js>,
    state: &mut CloneState<'js>,
) -> Result<Value<'js>> {
    let target = Object::new(ctx.clone())?;
    state.seen.insert(value_hash(original), target.clone().into_value());
    clone_string_properties(ctx, source, &target, state)?;
    Ok(target.into_value())
}

fn clone_error_like<'js>(
    ctx: &Ctx<'js>,
    source: &Object<'js>,
    original: &Value<'js>,
    state: &mut CloneState<'js>,
) -> Result<Value<'js>> {
    let name = source.get::<_, String>("name").unwrap_or_else(|_| "Error".to_string());
    let message = source.get::<_, String>("message").unwrap_or_default();

    let target: Value<'js> = if name.ends_with("Error") && name != "Error" {
        match ctx.globals().get::<_, rquickjs::function::Constructor>(name.as_str()) {
            Ok(ctor) => ctor.construct((message.clone(),))?,
            Err(_) => BasePrimordials::get(ctx)?.constructor_error.construct((message.clone(),))?,
        }
    } else if is_dom_exception(ctx, source)? {
        match ctx.globals().get::<_, rquickjs::function::Constructor>("DOMException") {
            Ok(ctor) => ctor.construct((message.clone(), name.clone()))?,
            Err(_) => BasePrimordials::get(ctx)?.constructor_error.construct((message.clone(),))?,
        }
    } else {
        BasePrimordials::get(ctx)?.constructor_error.construct((message.clone(),))?
    };

    state.seen.insert(value_hash(original), target.clone());
    if let Some(target_object) = target.as_object() {
        clone_string_properties(ctx, source, target_object, state)?;
    }
    Ok(target)
}

fn object_constructor_clone<'js>(
    ctx: &Ctx<'js>,
    object: &Object<'js>,
    original: &Value<'js>,
    state: &mut CloneState<'js>,
    clone_props: bool,
) -> Result<Value<'js>> {
    let ctor: rquickjs::function::Constructor = object.get(PredefinedAtom::Constructor)?;
    let target: Value<'js> = ctor.construct((object.clone(),))?;
    state.seen.insert(value_hash(original), target.clone());
    if clone_props {
        if let Some(target_object) = target.as_object() {
            clone_string_properties(ctx, object, target_object, state)?;
        }
    }
    Ok(target)
}

fn clone_array_buffer_view<'js>(
    ctx: &Ctx<'js>,
    object: &Object<'js>,
    original: &Value<'js>,
    state: &mut CloneState<'js>,
) -> Result<Value<'js>> {
    if let Ok(slice) = object.get::<_, Function>("slice") {
        let ctor: rquickjs::function::Constructor = object.get(PredefinedAtom::Constructor)?;
        let bytes: Value<'js> = slice.call((This(object.clone()),))?;
        let target: Value<'js> = ctor.construct((bytes,))?;
        state.seen.insert(value_hash(original), target.clone());
        return Ok(target);
    }

    let buffer: ArrayBuffer<'js> = object.get("buffer")?;
    let offset: usize = object.get("byteOffset").unwrap_or(0);
    let length: usize = object.get("byteLength").unwrap_or(0);
    let bytes = buffer.as_bytes().ok_or_else(|| {
        Exception::throw_type(ctx, "Cannot clone a detached ArrayBuffer")
    })?;
    let end = offset.saturating_add(length).min(bytes.len());
    let cloned_buffer = ArrayBuffer::new_copy(ctx.clone(), &bytes[offset..end])?;
    let ctor: rquickjs::function::Constructor = object.get(PredefinedAtom::Constructor)?;
    let target: Value<'js> = ctor.construct((cloned_buffer,))?;
    state.seen.insert(value_hash(original), target.clone());
    Ok(target)
}

fn clone_set<'js>(
    ctx: &Ctx<'js>,
    object: &Object<'js>,
    original: &Value<'js>,
    state: &mut CloneState<'js>,
) -> Result<Value<'js>> {
    let ctor = BasePrimordials::get(ctx)?.constructor_set.clone();
    let target: Object<'js> = ctor.construct(())?;
    state.seen.insert(value_hash(original), target.clone().into_value());

    let values: Array<'js> = BasePrimordials::get(ctx)?
        .function_array_from
        .call((object.clone(),))?;
    let add: Function<'js> = target.get("add")?;
    for value in values.iter::<Value<'js>>() {
        add.call::<_, Value>((This(target.clone()), clone_value(ctx, value?, state)?))?;
    }

    Ok(target.into_value())
}

fn clone_map<'js>(
    ctx: &Ctx<'js>,
    object: &Object<'js>,
    original: &Value<'js>,
    state: &mut CloneState<'js>,
) -> Result<Value<'js>> {
    let ctor = BasePrimordials::get(ctx)?.constructor_map.clone();
    let target: Object<'js> = ctor.construct(())?;
    state.seen.insert(value_hash(original), target.clone().into_value());

    let entries: Array<'js> = BasePrimordials::get(ctx)?
        .function_array_from
        .call((object.clone(),))?;
    let set: Function<'js> = target.get("set")?;
    for entry in entries.iter::<Array<'js>>() {
        let entry = entry?;
        let key: Value<'js> = entry.get(0)?;
        let value: Value<'js> = entry.get(1)?;
        set.call::<_, Value>((
            This(target.clone()),
            clone_value(ctx, key, state)?,
            clone_value(ctx, value, state)?,
        ))?;
    }

    Ok(target.into_value())
}

fn clone_string_properties<'js>(
    ctx: &Ctx<'js>,
    source: &Object<'js>,
    target: &Object<'js>,
    state: &mut CloneState<'js>,
) -> Result<()> {
    for key in source.keys::<String>() {
        let key = key?;
        if should_skip_builtin_property(&key) {
            continue;
        }
        let property_value: Value<'js> = source.get(key.as_str())?;
        target.set(key, clone_value(ctx, property_value, state)?)?;
    }
    Ok(())
}

fn should_skip_builtin_property(key: &str) -> bool {
    matches!(key, "length" | "byteLength" | "byteOffset" | "buffer")
}

fn seen_value<'js>(value: &Value<'js>, state: &CloneState<'js>) -> Option<Value<'js>> {
    state.seen.get(&value_hash(value)).cloned()
}

fn value_hash(value: &Value<'_>) -> usize {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish() as usize
}

fn is_dom_exception<'js>(ctx: &Ctx<'js>, object: &Object<'js>) -> Result<bool> {
    let Ok(dom_exception) = ctx.globals().get::<_, Value>("DOMException") else {
        return Ok(false);
    };
    Ok(object.is_instance_of(&dom_exception))
}

fn reject_unsupported_transfer<'js>(
    ctx: &Ctx<'js>,
    options: Option<&Value<'js>>,
) -> Result<()> {
    let Some(options) = options else {
        return Ok(());
    };
    if options.is_undefined() || options.is_null() {
        return Ok(());
    }

    let Some(object) = options.as_object() else {
        return Err(Exception::throw_type(
            ctx,
            "structuredClone options must be an object",
        ));
    };

    let transfer = object
        .get::<_, Value>("transfer")
        .unwrap_or_else(|_| Value::new_undefined(ctx.clone()));
    if transfer.is_undefined() || transfer.is_null() {
        return Ok(());
    }

    if transfer.is_array() {
        let array = transfer.as_array().expect("Value checked as array");
        if array.len() == 0 {
            return Ok(());
        }
    }

    throw_dom_exception(
        ctx,
        dom_exception_name::DATA_CLONE,
        "structuredClone transfer list is not supported yet",
    )
}

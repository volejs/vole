use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
    Array, Class, Constructor, Ctx, Function, Object, Result, Value,
};

#[derive(Clone, Debug)]
pub struct EventTarget;

unsafe impl<'js> rquickjs::JsLifetime<'js> for EventTarget {
    type Changed<'to> = EventTarget;
}

impl<'js> Trace<'js> for EventTarget {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> rquickjs::IntoJs<'js> for EventTarget {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn ensure_listeners<'js>(ctx: &Ctx<'js>, this: &Object<'js>) -> Result<Object<'js>> {
    if !this.contains_key("_listeners")? {
        this.set("_listeners", Object::new(ctx.clone())?)?;
    }
    this.get::<&str, Object>("_listeners")
}

fn parse_listener_options(options: Option<Value<'_>>) -> (bool, bool) {
    let mut capture = false;
    let mut once = false;
    if let Some(options) = options {
        if let Some(c) = options.as_bool() {
            capture = c;
        } else if let Some(obj) = options.as_object() {
            if let Ok(v) = obj.get::<&str, bool>("capture") {
                capture = v;
            }
            if let Ok(v) = obj.get::<&str, bool>("once") {
                once = v;
            }
        }
    }
    (capture, once)
}

fn listener_key(type_: &str, capture: bool) -> String {
    if capture {
        format!("{}~capture", type_)
    } else {
        type_.to_string()
    }
}

fn add_event_listener<'js>(
    ctx: Ctx<'js>,
    this: This<Object<'js>>,
    type_: String,
    callback: Function<'js>,
    options: Opt<Value<'js>>,
) -> Result<()> {
    let (capture, once) = parse_listener_options(options.0);
    let listeners = ensure_listeners(&ctx, &this.0)?;
    let key = listener_key(&type_, capture);
    let list: Array<'js> = if listeners.contains_key(&key)? {
        listeners.get(&key)?
    } else {
        let arr = Array::new(ctx.clone())?;
        listeners.set(key.as_str(), arr.clone())?;
        arr
    };

    let len = list.len();
    for i in 0..len {
        let entry: Object<'js> = match list.get(i)? {
            Some(entry) => entry,
            None => continue,
        };
        let existing: Function<'js> = entry.get("callback")?;
        let existing_capture: bool = entry.get("capture")?;
        if existing == callback && existing_capture == capture {
            return Ok(());
        }
    }

    let entry = Object::new(ctx.clone())?;
    entry.set("callback", callback)?;
    entry.set("capture", capture)?;
    entry.set("once", once)?;
    list.set(len, entry)?;
    Ok(())
}

fn remove_event_listener<'js>(
    ctx: Ctx<'js>,
    this: This<Object<'js>>,
    type_: String,
    callback: Function<'js>,
    options: Opt<Value<'js>>,
) -> Result<()> {
    let (capture, _) = parse_listener_options(options.0);
    let listeners = ensure_listeners(&ctx, &this.0)?;
    let key = listener_key(&type_, capture);
    if !listeners.contains_key(&key)? {
        return Ok(());
    }
    let list: Array<'js> = listeners.get(&key)?;
    let len = list.len();
    for i in (0..len).rev() {
        let entry: Object<'js> = match list.get(i)? {
            Some(entry) => entry,
            None => continue,
        };
        let existing: Function<'js> = entry.get("callback")?;
        let existing_capture: bool = entry.get("capture")?;
        if existing == callback && existing_capture == capture {
            list.set(i, Value::new_undefined(ctx.clone()))?;
        }
    }
    Ok(())
}

fn set_event_target<'js>(_ctx: &Ctx<'js>, event: &Object<'js>, target: &Object<'js>) -> Result<()> {
    let target_value = target.clone().into_value();
    if let Some(cls) = Class::<crate::event::Event>::from_object(event) {
        let mut ev = cls.try_borrow_mut()?;
        ev.target = Some(target_value.clone());
        ev.current_target = Some(target_value);
    } else if let Some(cls) = Class::<crate::custom_event::CustomEvent>::from_object(event) {
        let mut ev = cls.try_borrow_mut()?;
        ev.target = Some(target_value.clone());
        ev.current_target = Some(target_value);
    } else if let Some(cls) = Class::<crate::message_event::MessageEvent>::from_object(event) {
        let mut ev = cls.try_borrow_mut()?;
        ev.target = Some(target_value.clone());
        ev.current_target = Some(target_value);
    }
    Ok(())
}

fn is_stop_immediate_propagation<'js>(_ctx: &Ctx<'js>, event: &Object<'js>) -> bool {
    if let Some(cls) = Class::<crate::event::Event>::from_object(event) {
        return cls
            .try_borrow()
            .map(|e| e.stop_immediate_propagation)
            .unwrap_or(false);
    }
    if let Some(cls) = Class::<crate::custom_event::CustomEvent>::from_object(event) {
        return cls
            .try_borrow()
            .map(|e| e.stop_immediate_propagation)
            .unwrap_or(false);
    }
    if let Some(cls) = Class::<crate::message_event::MessageEvent>::from_object(event) {
        return cls
            .try_borrow()
            .map(|e| e.stop_immediate_propagation)
            .unwrap_or(false);
    }
    false
}

pub fn dispatch_event_on_object<'js>(
    ctx: &Ctx<'js>,
    target_obj: &Object<'js>,
    event: &Object<'js>,
) -> Result<bool> {
    let type_: String = event.get::<&str, String>("type")?;
    set_event_target(ctx, event, target_obj)?;

    let listeners: Object<'js> = ensure_listeners(ctx, target_obj)?;

    let key = listener_key(&type_, false);
    if listeners.contains_key(&key)? {
        let list: Array<'js> = listeners.get(&key)?;
        let len = list.len();
        let mut to_remove = Vec::new();
        for i in 0..len {
            let entry: Object<'js> = match list.get(i)? {
                Some(entry) => entry,
                None => continue,
            };
            let callback: Function<'js> = entry.get("callback")?;
            let once: bool = entry.get("once")?;
            callback.call::<_, ()>((event.clone(),))?;
            if once {
                to_remove.push(i);
            }
            if is_stop_immediate_propagation(ctx, event) {
                break;
            }
        }
        for i in to_remove.into_iter().rev() {
            list.set(i, Value::new_undefined(ctx.clone()))?;
        }
    }

    let default_prevented: bool = event.get::<&str, bool>("defaultPrevented")?;
    Ok(!default_prevented)
}

fn dispatch_event<'js>(ctx: Ctx<'js>, this: This<Object<'js>>, event: Object<'js>) -> Result<bool> {
    dispatch_event_on_object(&ctx, &this.0, &event)
}

fn event_target_ctor<'js>(_ctx: Ctx<'js>) -> Result<EventTarget> {
    Ok(EventTarget)
}

impl<'js> JsClass<'js> for EventTarget {
    const NAME: &'static str = "EventTarget";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set(
            "addEventListener",
            Function::new(ctx.clone(), add_event_listener)?,
        )?;
        proto.set(
            "removeEventListener",
            Function::new(ctx.clone(), remove_event_listener)?,
        )?;
        proto.set("dispatchEvent", Function::new(ctx.clone(), dispatch_event)?)?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<EventTarget, _, _>(ctx.clone(), event_target_ctor)?;
        Ok(Some(constr))
    }
}

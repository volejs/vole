use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
    Array, Class, Constructor, Ctx, Function, Object, Result, Value,
};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static START: OnceLock<Instant> = OnceLock::new();
static START_TIME_ORIGIN: OnceLock<f64> = OnceLock::new();
static MARKS: OnceLock<Mutex<HashMap<String, (Instant, f64)>>> = OnceLock::new();
static MEASURES: OnceLock<Mutex<HashMap<String, f64>>> = OnceLock::new();
static ENTRIES: OnceLock<Mutex<Vec<PerfEntry>>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct Performance;

#[derive(Clone)]
struct PerfEntry {
    name: String,
    entry_type: String,
    start_time: f64,
    duration: f64,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for Performance {
    type Changed<'to> = Performance;
}

impl<'js> Trace<'js> for Performance {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> rquickjs::IntoJs<'js> for Performance {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

fn marks() -> &'static Mutex<HashMap<String, (Instant, f64)>> {
    MARKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn measures() -> &'static Mutex<HashMap<String, f64>> {
    MEASURES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn entries() -> &'static Mutex<Vec<PerfEntry>> {
    ENTRIES.get_or_init(|| Mutex::new(Vec::new()))
}

fn now_ms() -> f64 {
    let start = START.get_or_init(Instant::now);
    start.elapsed().as_secs_f64() * 1000.0
}

fn time_origin_ms() -> f64 {
    *START_TIME_ORIGIN.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0
    })
}

fn performance_ctor() -> Result<Performance> {
    Ok(Performance)
}

fn performance_now(_this: This<Class<'_, Performance>>) -> f64 {
    now_ms()
}

fn performance_time_origin(_this: This<Class<'_, Performance>>) -> f64 {
    time_origin_ms()
}

fn performance_mark<'js>(
    ctx: Ctx<'js>,
    _this: This<Class<'js, Performance>>,
    name: String,
) -> Result<Object<'js>> {
    let now = now_ms();
    let instant = Instant::now();
    marks().lock().unwrap().insert(name.clone(), (instant, now));
    let entry = PerfEntry {
        name: name.clone(),
        entry_type: "mark".to_string(),
        start_time: now,
        duration: 0.0,
    };
    entries().lock().unwrap().push(entry.clone());
    build_entry_obj(&ctx, &entry)
}

fn performance_measure<'js>(
    ctx: Ctx<'js>,
    _this: This<Class<'js, Performance>>,
    name: String,
    start_mark: Opt<String>,
    end_mark: Opt<String>,
) -> Result<Object<'js>> {
    let marks = marks().lock().unwrap();
    let (start_instant, start_time) = start_mark
        .0
        .and_then(|m| marks.get(&m).cloned())
        .unwrap_or_else(|| (*START.get_or_init(Instant::now), 0.0));
    let end = end_mark
        .0
        .and_then(|m| marks.get(&m).cloned())
        .map(|(inst, _)| inst)
        .unwrap_or_else(Instant::now);
    let elapsed = end.duration_since(start_instant).as_secs_f64() * 1000.0;
    measures().lock().unwrap().insert(name.clone(), elapsed);
    let entry = PerfEntry {
        name: name.clone(),
        entry_type: "measure".to_string(),
        start_time,
        duration: elapsed,
    };
    entries().lock().unwrap().push(entry.clone());
    println!("{}: {:.3}ms", name, elapsed);
    build_entry_obj(&ctx, &entry)
}

fn performance_clear_marks(_this: This<Class<'_, Performance>>, name: Opt<String>) {
    let mut marks = marks().lock().unwrap();
    if let Some(n) = name.0 {
        marks.remove(&n);
    } else {
        marks.clear();
    }
}

fn performance_clear_measures(_this: This<Class<'_, Performance>>, name: Opt<String>) {
    let mut measures = measures().lock().unwrap();
    if let Some(n) = name.0 {
        measures.remove(&n);
    } else {
        measures.clear();
    }
}

fn performance_to_json<'js>(
    _this: This<Class<'js, Performance>>,
    ctx: Ctx<'js>,
) -> Result<Object<'js>> {
    let obj = Object::new(ctx)?;
    obj.set("timeOrigin", time_origin_ms())?;
    Ok(obj)
}

fn build_entry_obj<'js>(ctx: &Ctx<'js>, entry: &PerfEntry) -> Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;
    obj.set("name", entry.name.clone())?;
    obj.set("entryType", entry.entry_type.clone())?;
    obj.set("startTime", entry.start_time)?;
    obj.set("duration", entry.duration)?;
    Ok(obj)
}

fn performance_get_entries<'js>(
    _this: This<Class<'js, Performance>>,
    ctx: Ctx<'js>,
) -> Result<Array<'js>> {
    let entries = entries().lock().unwrap();
    let arr = Array::new(ctx.clone())?;
    for (i, entry) in entries.iter().enumerate() {
        arr.set(i, build_entry_obj(&ctx, entry)?)?;
    }
    Ok(arr)
}

fn performance_get_entries_by_type<'js>(
    _this: This<Class<'js, Performance>>,
    ctx: Ctx<'js>,
    entry_type: String,
) -> Result<Array<'js>> {
    let entries = entries().lock().unwrap();
    let arr = Array::new(ctx.clone())?;
    let mut idx = 0usize;
    for entry in entries.iter().filter(|e| e.entry_type == entry_type) {
        arr.set(idx, build_entry_obj(&ctx, entry)?)?;
        idx += 1;
    }
    Ok(arr)
}

fn performance_get_entries_by_name<'js>(
    _this: This<Class<'js, Performance>>,
    ctx: Ctx<'js>,
    name: String,
    entry_type: Opt<String>,
) -> Result<Array<'js>> {
    let entries = entries().lock().unwrap();
    let arr = Array::new(ctx.clone())?;
    let mut idx = 0usize;
    for entry in entries
        .iter()
        .filter(|e| e.name == name && entry_type.0.as_ref().map_or(true, |t| e.entry_type == *t))
    {
        arr.set(idx, build_entry_obj(&ctx, entry)?)?;
        idx += 1;
    }
    Ok(arr)
}

impl<'js> JsClass<'js> for Performance {
    const NAME: &'static str = "Performance";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("now", Function::new(ctx.clone(), performance_now)?)?;
        proto.set("mark", Function::new(ctx.clone(), performance_mark)?)?;
        proto.set("measure", Function::new(ctx.clone(), performance_measure)?)?;
        proto.set(
            "clearMarks",
            Function::new(ctx.clone(), performance_clear_marks),
        )?;
        proto.set(
            "clearMeasures",
            Function::new(ctx.clone(), performance_clear_measures),
        )?;
        proto.set("toJSON", Function::new(ctx.clone(), performance_to_json)?)?;
        proto.set(
            "getEntries",
            Function::new(ctx.clone(), performance_get_entries),
        )?;
        proto.set(
            "getEntriesByType",
            Function::new(ctx.clone(), performance_get_entries_by_type),
        )?;
        proto.set(
            "getEntriesByName",
            Function::new(ctx.clone(), performance_get_entries_by_name),
        )?;
        // timeOrigin is exposed as a data property on the global performance object
        // and as a getter on the prototype.
        proto.set(
            "timeOrigin",
            Function::new(ctx.clone(), performance_time_origin),
        )?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<Performance, _, _>(ctx.clone(), performance_ctor)?;
        Ok(Some(constr))
    }
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let _ = START.get_or_init(Instant::now);

    Class::<Performance>::define(&ctx.globals())?;

    // Create the global `performance` object as an instance of Performance.
    let perf = Class::<Performance>::instance(ctx.clone(), Performance)?;
    ctx.globals().set("performance", perf)?;

    // Turn prototype methods into getters where the spec requires getters.
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
            wrapGetter(Performance.prototype, "timeOrigin");
        })();
    "#,
    )?;

    Ok(())
}

use rquickjs::function::Opt;
use rquickjs::{Array, Ctx, Function, Object, Result};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static START: OnceLock<Instant> = OnceLock::new();
static START_TIME_ORIGIN: OnceLock<f64> = OnceLock::new();
static MARKS: OnceLock<Mutex<HashMap<String, (Instant, f64)>>> = OnceLock::new();
static MEASURES: OnceLock<Mutex<HashMap<String, f64>>> = OnceLock::new();
static ENTRIES: OnceLock<Mutex<Vec<PerfEntry>>> = OnceLock::new();

#[derive(Clone)]
struct PerfEntry {
    name: String,
    entry_type: String,
    start_time: f64,
    duration: f64,
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

pub fn init(ctx: &Ctx) -> Result<()> {
    let _ = START.get_or_init(Instant::now);
    let perf = Object::new(ctx.clone())?;
    perf.set("now", Function::new(ctx.clone(), || now_ms()))?;
    perf.set("timeOrigin", time_origin_ms())?;
    perf.set("mark", Function::new(ctx.clone(), perf_mark))?;
    perf.set("measure", Function::new(ctx.clone(), perf_measure))?;
    perf.set("clearMarks", Function::new(ctx.clone(), perf_clear_marks))?;
    perf.set(
        "clearMeasures",
        Function::new(ctx.clone(), perf_clear_measures),
    )?;
    perf.set("toJSON", Function::new(ctx.clone(), perf_to_json))?;
    perf.set("getEntries", Function::new(ctx.clone(), perf_get_entries))?;
    perf.set(
        "getEntriesByType",
        Function::new(ctx.clone(), perf_get_entries_by_type),
    )?;
    perf.set(
        "getEntriesByName",
        Function::new(ctx.clone(), perf_get_entries_by_name),
    )?;
    ctx.globals().set("performance", perf)?;

    // Expose the Performance constructor so `new Performance()` and
    // `performance instanceof Performance` work as expected.
    ctx.eval::<(), _>(r#"
        (function() {
            class Performance {
                constructor() {}
                now() { return performance.now(); }
                mark(name) { return performance.mark(name); }
                measure(name, startMark, endMark) { return performance.measure(name, startMark, endMark); }
                clearMarks(name) { performance.clearMarks(name); }
                clearMeasures(name) { performance.clearMeasures(name); }
                getEntries() { return performance.getEntries(); }
                getEntriesByType(type) { return performance.getEntriesByType(type); }
                getEntriesByName(name, type) { return performance.getEntriesByName(name, type); }
                toJSON() { return performance.toJSON(); }
            }
            Object.defineProperty(Performance.prototype, "timeOrigin", {
                get: function() { return performance.timeOrigin; },
                enumerable: true,
                configurable: true,
            });
            globalThis.Performance = Performance;
            Object.setPrototypeOf(performance, Performance.prototype);
        })();
    "#)?;

    Ok(())
}

fn perf_mark<'js>(ctx: Ctx<'js>, name: String) -> Result<Object<'js>> {
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

fn perf_measure<'js>(
    ctx: Ctx<'js>,
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

fn perf_clear_marks(name: Option<String>) {
    let mut marks = marks().lock().unwrap();
    if let Some(n) = name {
        marks.remove(&n);
    } else {
        marks.clear();
    }
}

fn perf_clear_measures(name: Option<String>) {
    let mut measures = measures().lock().unwrap();
    if let Some(n) = name {
        measures.remove(&n);
    } else {
        measures.clear();
    }
}

fn perf_to_json<'js>(ctx: Ctx<'js>) -> Result<Object<'js>> {
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

fn perf_get_entries<'js>(ctx: Ctx<'js>) -> Result<Array<'js>> {
    let entries = entries().lock().unwrap();
    let arr = Array::new(ctx.clone())?;
    for (i, entry) in entries.iter().enumerate() {
        arr.set(i, build_entry_obj(&ctx, entry)?)?;
    }
    Ok(arr)
}

fn perf_get_entries_by_type<'js>(ctx: Ctx<'js>, entry_type: String) -> Result<Array<'js>> {
    let entries = entries().lock().unwrap();
    let arr = Array::new(ctx.clone())?;
    let mut idx = 0usize;
    for entry in entries.iter().filter(|e| e.entry_type == entry_type) {
        arr.set(idx, build_entry_obj(&ctx, entry)?)?;
        idx += 1;
    }
    Ok(arr)
}

fn perf_get_entries_by_name<'js>(
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

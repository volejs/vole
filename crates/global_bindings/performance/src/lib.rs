use std::{cell::RefCell, rc::Rc};

use rquickjs::{
    atom::PredefinedAtom,
    class::Trace,
    function::{Opt, This},
    Array, Class, Coerced, Ctx, Exception, FromJs, Function, Object, Result, Value,
};
use vole_core::time::TimeOrigin;

#[derive(Clone)]
struct PerformanceEntryRecord {
    name: String,
    entry_type: &'static str,
    start_time: f64,
    duration: f64,
}

#[derive(Default)]
struct PerformanceState {
    marks: Vec<PerformanceEntryRecord>,
    measures: Vec<PerformanceEntryRecord>,
    entries: Vec<PerformanceEntryRecord>,
}

const PERFORMANCE_ENTRY_MARK: &str = "mark";
const PERFORMANCE_ENTRY_MEASURE: &str = "measure";

/// Native `Performance` implementation.
///
/// 时间源来自 `vole_core::time::TimeOrigin`，按 JS context 隔离；
/// mark/measure/entries 是 `Performance` 实例自己的状态，不再使用进程级 static。
#[derive(Clone, Trace, rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct Performance {
    #[qjs(skip_trace)]
    state: Rc<RefCell<PerformanceState>>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> Performance {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> Result<Self> {
        TimeOrigin::init(&ctx)?;
        Ok(Self {
            state: Rc::new(RefCell::new(PerformanceState::default())),
        })
    }

    #[qjs(get, rename = "timeOrigin")]
    pub fn time_origin(&self, ctx: Ctx<'js>) -> Result<f64> {
        Ok(TimeOrigin::get_or_init(&ctx)?.time_origin())
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        "Performance"
    }

    pub fn now(&self, ctx: Ctx<'js>) -> Result<f64> {
        Ok(TimeOrigin::get_or_init(&ctx)?.now())
    }

    pub fn mark(
        &self,
        ctx: Ctx<'js>,
        name: Coerced<String>,
        options: Opt<Value<'js>>,
    ) -> Result<Object<'js>> {
        let now = TimeOrigin::get_or_init(&ctx)?.now();
        let start_time = match options.0.as_ref() {
            Some(value) if !value.is_undefined() => parse_mark_start_time(&ctx, value, now)?,
            _ => now,
        };

        if start_time < 0.0 {
            return Err(Exception::throw_range(
                &ctx,
                "Performance mark startTime must not be negative",
            ));
        }

        let entry = PerformanceEntryRecord {
            name: name.0,
            entry_type: PERFORMANCE_ENTRY_MARK,
            start_time,
            duration: 0.0,
        };
        self.push_entry(entry.clone());
        entry_to_object(&ctx, &entry)
    }

    pub fn measure(
        &self,
        ctx: Ctx<'js>,
        name: Coerced<String>,
        start: Opt<Value<'js>>,
        end: Opt<Value<'js>>,
    ) -> Result<Object<'js>> {
        let now = TimeOrigin::get_or_init(&ctx)?.now();
        let (start_time, end_time) =
            self.resolve_measure_inputs(&ctx, start.0, end.0, now)?;

        let duration = end_time - start_time;
        let entry = PerformanceEntryRecord {
            name: name.0,
            entry_type: PERFORMANCE_ENTRY_MEASURE,
            start_time,
            duration,
        };
        self.push_entry(entry.clone());
        entry_to_object(&ctx, &entry)
    }

    pub fn clear_marks(&self, name: Opt<Coerced<String>>) {
        clear_entries(&mut self.state.borrow_mut(), PERFORMANCE_ENTRY_MARK, name.0.map(|name| name.0));
    }

    pub fn clear_measures(&self, name: Opt<Coerced<String>>) {
        clear_entries(
            &mut self.state.borrow_mut(),
            PERFORMANCE_ENTRY_MEASURE,
            name.0.map(|name| name.0),
        );
    }

    pub fn get_entries(&self, ctx: Ctx<'js>) -> Result<Array<'js>> {
        entries_to_array(&ctx, self.state.borrow().entries.iter())
    }

    pub fn get_entries_by_type(
        &self,
        ctx: Ctx<'js>,
        entry_type: Coerced<String>,
    ) -> Result<Array<'js>> {
        entries_to_array(
            &ctx,
            self.state
                .borrow()
                .entries
                .iter()
                .filter(|entry| entry.entry_type == entry_type.0),
        )
    }

    pub fn get_entries_by_name(
        &self,
        ctx: Ctx<'js>,
        name: Coerced<String>,
        entry_type: Opt<Coerced<String>>,
    ) -> Result<Array<'js>> {
        let entry_type = entry_type.0.map(|entry_type| entry_type.0);
        entries_to_array(
            &ctx,
            self.state.borrow().entries.iter().filter(|entry| {
                entry.name == name.0
                    && entry_type
                        .as_ref()
                        .map(|entry_type| entry.entry_type == entry_type)
                        .unwrap_or(true)
            }),
        )
    }

    pub fn to_json(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let object = Object::new(ctx.clone())?;
        object.set("timeOrigin", TimeOrigin::get_or_init(&ctx)?.time_origin())?;
        Ok(object)
    }
}

impl Performance {
    fn push_entry(&self, entry: PerformanceEntryRecord) {
        let mut state = self.state.borrow_mut();
        match entry.entry_type {
            PERFORMANCE_ENTRY_MARK => state.marks.push(entry.clone()),
            PERFORMANCE_ENTRY_MEASURE => state.measures.push(entry.clone()),
            _ => {}
        }
        state.entries.push(entry);
    }

    fn resolve_mark_or_time<'js>(&self, ctx: &Ctx<'js>, value: Value<'js>) -> Result<f64> {
        if let Some(number) = value.as_number() {
            return Ok(number);
        }

        let name = Coerced::<String>::from_js(ctx, value)?.0;
        self.state
            .borrow()
            .marks
            .iter()
            .rev()
            .find(|entry| entry.name == name)
            .map(|entry| entry.start_time)
            .ok_or_else(|| Exception::throw_syntax(ctx, "Performance mark does not exist"))
    }

    fn resolve_measure_inputs<'js>(
        &self,
        ctx: &Ctx<'js>,
        start: Option<Value<'js>>,
        end: Option<Value<'js>>,
        now: f64,
    ) -> Result<(f64, f64)> {
        let (default_start_time, default_end_time) = (0.0, now);

        match (start, end) {
            (None, None) => Ok((default_start_time, default_end_time)),
            (Some(start_value), None) if is_measure_options(ctx, &start_value)? => {
                let object = start_value.as_object().ok_or_else(|| {
                    Exception::throw_type(ctx, "Performance.measure options must be an object")
                })?;
                let (start_time, end_time) = parse_measure_options(self, ctx, object)?;
                Ok((start_time.unwrap_or(default_start_time), end_time.unwrap_or(default_end_time)))
            }
            (Some(start_value), None) => {
                let start_time = self.resolve_mark_or_time(ctx, start_value)?;
                Ok((start_time, default_end_time))
            }
            (Some(start_value), Some(end_value)) => {
                let start_time = self.resolve_mark_or_time(ctx, start_value)?;
                let end_time = if end_value.is_undefined() {
                    default_end_time
                } else {
                    self.resolve_mark_or_time(ctx, end_value)?
                };
                Ok((start_time, end_time))
            }
            (None, Some(end_value)) if is_measure_options(ctx, &end_value)? => {
                let object = end_value.as_object().ok_or_else(|| {
                    Exception::throw_type(ctx, "Performance.measure options must be an object")
                })?;
                let (start_time, end_time) = parse_measure_options(self, ctx, object)?;
                Ok((start_time.unwrap_or(default_start_time), end_time.unwrap_or(default_end_time)))
            }
            (None, Some(end_value)) => {
                let end_time = if end_value.is_undefined() {
                    default_end_time
                } else {
                    self.resolve_mark_or_time(ctx, end_value)?
                };
                Ok((default_start_time, end_time))
            }
        }
    }
}

fn clear_entries(state: &mut PerformanceState, entry_type: &'static str, name: Option<String>) {
    state
        .entries
        .retain(|entry| should_keep_entry(entry, entry_type, name.as_deref()));
    match entry_type {
        PERFORMANCE_ENTRY_MARK => state
            .marks
            .retain(|entry| should_keep_entry(entry, entry_type, name.as_deref())),
        PERFORMANCE_ENTRY_MEASURE => state
            .measures
            .retain(|entry| should_keep_entry(entry, entry_type, name.as_deref())),
        _ => {}
    }
}

fn should_keep_entry(
    entry: &PerformanceEntryRecord,
    entry_type: &'static str,
    name: Option<&str>,
) -> bool {
    if entry.entry_type != entry_type {
        return true;
    }
    name.map(|name| entry.name != name).unwrap_or(false)
}

fn is_measure_options<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<bool> {
    let object = match value.as_object() {
        Some(object) => object,
        None => return Ok(false),
    };

    let start = get_optional_property(ctx, &object, "start")?;
    let end = get_optional_property(ctx, &object, "end")?;
    let start_time = get_optional_property(ctx, &object, "startTime")?;
    let end_time = get_optional_property(ctx, &object, "endTime")?;

    !start.is_undefined()
        || !end.is_undefined()
        || !start_time.is_undefined()
        || !end_time.is_undefined()
}

fn parse_mark_start_time<'js>(
    ctx: &Ctx<'js>,
    value: &Value<'js>,
    fallback: f64,
) -> Result<f64> {
    let object = value
        .as_object()
        .ok_or_else(|| Exception::throw_type(ctx, "PerformanceMarkOptions must be an object"))?;

    let raw = get_optional_property(ctx, &object, "startTime")?;
    if raw.is_undefined() {
        return Ok(fallback);
    }

    let start_time = Coerced::<f64>::from_js(ctx, raw)?.0;
    if start_time < 0.0 {
        return Err(Exception::throw_range(
            ctx,
            "Performance mark startTime must not be negative",
        ));
    }
    Ok(start_time)
}

fn parse_measure_options<'js>(
    performance: &Performance,
    ctx: &Ctx<'js>,
    object: Object<'js>,
) -> Result<(Option<f64>, Option<f64>)> {
    let parse_time = |value: Value<'js>| -> Result<f64> {
        if value.is_undefined() {
            return Ok(0.0);
        }
        performance.resolve_mark_or_time(ctx, value)
    };

    let mut start = None;
    let mut end = None;

    let start_value = get_optional_property(ctx, &object, "start")?;
    if !start_value.is_undefined() {
        start = Some(parse_time(start_value)?);
    }

    let end_value = get_optional_property(ctx, &object, "end")?;
    if !end_value.is_undefined() {
        end = Some(parse_time(end_value)?);
    }

    if start.is_none() {
        let start_time = get_optional_property(ctx, &object, "startTime")?;
        if !start_time.is_undefined() {
            start = Some(parse_time(start_time)?);
        }
    }

    if end.is_none() {
        let end_time = get_optional_property(ctx, &object, "endTime")?;
        if !end_time.is_undefined() {
            end = Some(parse_time(end_time)?);
        }
    }

    Ok((start, end))
}

fn get_optional_property<'js>(ctx: &Ctx<'js>, object: &Object<'js>, key: &str) -> Result<Value<'js>> {
    object.get::<_, Value>(key).or_else(|_| Ok(Value::new_undefined(ctx.clone())))
}

fn entries_to_array<'js, I>(ctx: &Ctx<'js>, entries: I) -> Result<Array<'js>>
where
    I: IntoIterator,
    I::Item: std::borrow::Borrow<PerformanceEntryRecord>,
{
    let array = Array::new(ctx.clone())?;
    for (idx, entry) in entries.into_iter().enumerate() {
        array.set(idx, entry_to_object(ctx, std::borrow::Borrow::borrow(&entry))?)?;
    }
    Ok(array)
}

fn entry_to_object<'js>(
    ctx: &Ctx<'js>,
    entry: &PerformanceEntryRecord,
) -> Result<Object<'js>> {
    let object = Object::new(ctx.clone())?;
    object.set("name", entry.name.clone())?;
    object.set("entryType", entry.entry_type)?;
    object.set("startTime", entry.start_time)?;
    object.set("duration", entry.duration)?;
    object.set("detail", Value::new_undefined(ctx.clone()))?;

    let to_json = Function::new(ctx.clone(), |This(this): This<Object>| -> Result<Object> {
        let clone = Object::new(this.ctx())?;
        clone.set("name", this.get::<_, String>("name")?)?;
        clone.set("entryType", this.get::<_, String>("entryType")?)?;
        clone.set("startTime", this.get::<_, f64>("startTime")?)?;
        clone.set("duration", this.get::<_, f64>("duration")?)?;
        let detail: Value = this
            .get::<_, Value>("detail")
            .or_else(|_| Ok(Value::new_undefined(this.ctx())))?;
        clone.set("detail", detail)?;
        Ok(clone)
    })?;
    object.set("toJSON", to_json)?;

    Ok(object)
}

pub fn init(ctx: &Ctx) -> Result<()> {
    TimeOrigin::init(ctx)?;
    let globals = ctx.globals();
    Class::<Performance>::define(&globals)?;
    let performance = Class::instance(ctx.clone(), Performance::new(ctx.clone())?)?;
    globals.set("performance", performance)?;
    Ok(())
}

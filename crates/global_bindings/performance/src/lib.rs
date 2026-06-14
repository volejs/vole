use std::{cell::RefCell, rc::Rc};

use rquickjs::{
    atom::PredefinedAtom,
    class::Trace,
    function::Opt,
    Array, Class, Coerced, Ctx, Exception, FromJs, Object, Result, Value,
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
        let start_time = match options.0 {
            Some(value) if !value.is_undefined() && !value.is_null() => {
                let object = value.as_object().ok_or_else(|| {
                    Exception::throw_type(&ctx, "PerformanceMarkOptions must be an object")
                })?;
                object
                    .get::<_, Coerced<f64>>("startTime")
                    .map(|value| value.0)
                    .unwrap_or(TimeOrigin::get_or_init(&ctx)?.now())
            },
            _ => TimeOrigin::get_or_init(&ctx)?.now(),
        };

        if start_time < 0.0 {
            return Err(Exception::throw_range(
                &ctx,
                "Performance mark startTime must not be negative",
            ));
        }

        let entry = PerformanceEntryRecord {
            name: name.0,
            entry_type: "mark",
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
        let end_time = match end.0 {
            Some(value) if !value.is_undefined() => self.resolve_mark_or_time(&ctx, value)?,
            _ => TimeOrigin::get_or_init(&ctx)?.now(),
        };
        let start_time = match start.0 {
            Some(value) if !value.is_undefined() => self.resolve_mark_or_time(&ctx, value)?,
            _ => 0.0,
        };

        let duration = end_time - start_time;
        let entry = PerformanceEntryRecord {
            name: name.0,
            entry_type: "measure",
            start_time,
            duration,
        };
        self.push_entry(entry.clone());
        entry_to_object(&ctx, &entry)
    }

    pub fn clear_marks(&self, name: Opt<Coerced<String>>) {
        clear_entries(&mut self.state.borrow_mut(), "mark", name.0.map(|name| name.0));
    }

    pub fn clear_measures(&self, name: Opt<Coerced<String>>) {
        clear_entries(
            &mut self.state.borrow_mut(),
            "measure",
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
            "mark" => state.marks.push(entry.clone()),
            "measure" => state.measures.push(entry.clone()),
            _ => {},
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
}

fn clear_entries(state: &mut PerformanceState, entry_type: &'static str, name: Option<String>) {
    state
        .entries
        .retain(|entry| should_keep_entry(entry, entry_type, name.as_deref()));
    match entry_type {
        "mark" => state
            .marks
            .retain(|entry| should_keep_entry(entry, entry_type, name.as_deref())),
        "measure" => state
            .measures
            .retain(|entry| should_keep_entry(entry, entry_type, name.as_deref())),
        _ => {},
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

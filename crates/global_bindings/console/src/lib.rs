use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stderr, stdout, IsTerminal, Write},
    time::Instant,
};

use rquickjs::{
    atom::PredefinedAtom,
    function::Rest,
    object::Property,
    prelude::{Coerced, Opt},
    Ctx, Function, Object, Result, Type, Value,
};
use vole_core::{
    json::stringify::json_stringify,
    primordials::{BasePrimordials, Primordial},
};

const DEFAULT_LABEL: &str = "default";
const GROUP_INDENT: &str = "  ";
const MAX_ARRAY_ITEMS: usize = 20;
const MAX_OBJECT_KEYS: usize = 20;

#[derive(Default)]
struct ConsoleState {
    counters: RefCell<HashMap<String, u64>>,
    timers: RefCell<HashMap<String, Instant>>,
    group_indent: RefCell<String>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for ConsoleState {
    type Changed<'to> = ConsoleState;
}

pub fn init(ctx: &Ctx) -> Result<()> {
    BasePrimordials::init(ctx)?;
    if ctx.userdata::<ConsoleState>().is_none() {
        let _ = ctx.store_userdata(ConsoleState::default());
    }

    let globals = ctx.globals();
    let proto = Object::new(ctx.clone())?;
    let console = Object::new_proto(ctx.clone(), Some(&proto))?;

    console.set("log", Function::new(ctx.clone(), console_log))?;
    console.set("info", Function::new(ctx.clone(), console_log))?;
    console.set("debug", Function::new(ctx.clone(), console_debug))?;
    console.set("warn", Function::new(ctx.clone(), console_warn))?;
    console.set("error", Function::new(ctx.clone(), console_error))?;
    console.set("assert", Function::new(ctx.clone(), console_assert))?;
    console.set("clear", Function::new(ctx.clone(), console_clear))?;
    console.set("count", Function::new(ctx.clone(), console_count))?;
    console.set("countReset", Function::new(ctx.clone(), console_count_reset))?;
    console.set("time", Function::new(ctx.clone(), console_time))?;
    console.set("timeLog", Function::new(ctx.clone(), console_time_log))?;
    console.set("timeEnd", Function::new(ctx.clone(), console_time_end))?;
    console.set("group", Function::new(ctx.clone(), console_group))?;
    console.set(
        "groupCollapsed",
        Function::new(ctx.clone(), console_group_collapsed),
    )?;
    console.set("groupEnd", Function::new(ctx.clone(), console_group_end))?;
    console.set("trace", Function::new(ctx.clone(), console_trace))?;
    console.set("dir", Function::new(ctx.clone(), console_dir))?;
    console.set("dirxml", Function::new(ctx.clone(), console_dirxml))?;
    console.set("table", Function::new(ctx.clone(), console_table))?;
    console.set("profile", Function::new(ctx.clone(), console_noop))?;
    console.set("profileEnd", Function::new(ctx.clone(), console_noop))?;
    console.set("timeStamp", Function::new(ctx.clone(), console_noop))?;

    console.prop(
        PredefinedAtom::SymbolToStringTag,
        Property::from("console").configurable(),
    )?;
    globals.prop("console", Property::from(console).writable().configurable())?;

    Ok(())
}

fn console_log<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<()> {
    write_console(stdout(), &ctx, args)
}

fn console_debug<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<()> {
    write_console(stdout(), &ctx, args)
}

fn console_warn<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<()> {
    write_console(stderr(), &ctx, args)
}

fn console_error<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<()> {
    write_console(stderr(), &ctx, args)
}

fn console_assert<'js>(ctx: Ctx<'js>, condition: Value<'js>, args: Rest<Value<'js>>) -> Result<()> {
    if is_truthy(&condition) {
        return Ok(());
    }

    let args = if args.0.is_empty() {
        vec![string_value(ctx.clone(), "Assertion failed")?]
    } else {
        let mut values = Vec::with_capacity(args.0.len() + 1);
        values.push(string_value(ctx.clone(), "Assertion failed:")?);
        values.extend(args.0);
        values
    };
    write_console_values(stderr(), &ctx, args)
}

fn console_clear() {
    let mut out = stdout();
    let _ = out.write_all(b"\x1b[1;1H\x1b[0J");
    let _ = out.flush();
}

fn console_count(ctx: Ctx<'_>, label: Opt<Coerced<String>>) -> Result<()> {
    let label = get_label(label);
    let Some(state) = ctx.userdata::<ConsoleState>() else {
        return Ok(());
    };
    let mut counters = state.counters.borrow_mut();
    let count = counters.entry(label.clone()).or_insert(0);
    *count += 1;
    writeln_ignore(stdout(), &format!("{}: {}", label, count));
    Ok(())
}

fn console_count_reset(ctx: Ctx<'_>, label: Opt<Coerced<String>>) -> Result<()> {
    let label = get_label(label);
    let Some(state) = ctx.userdata::<ConsoleState>() else {
        return Ok(());
    };
    let existed = state.counters.borrow_mut().remove(&label).is_some();
    if !existed {
        writeln_ignore(
            stderr(),
            &format!(
                "Count for '{}' does not exist",
                label.replace('\'', "\\'")
            ),
        );
    }
    Ok(())
}

fn console_time(ctx: Ctx<'_>, label: Opt<Coerced<String>>) -> Result<()> {
    let label = get_label(label);
    let Some(state) = ctx.userdata::<ConsoleState>() else {
        return Ok(());
    };
    let mut timers = state.timers.borrow_mut();
    if timers.contains_key(&label) {
        writeln_ignore(stderr(), &format!("Timer '{}' already exists", label));
    } else {
        timers.insert(label, Instant::now());
    }
    Ok(())
}

fn console_time_log<'js>(
    ctx: Ctx<'js>,
    label: Opt<Coerced<String>>,
    args: Rest<Value<'js>>,
) -> Result<()> {
    let label = get_label(label);
    let Some(state) = ctx.userdata::<ConsoleState>() else {
        return Ok(());
    };
    let Some(start) = state.timers.borrow().get(&label).copied() else {
        writeln_ignore(stderr(), &format!("Timer '{}' does not exist", label));
        return Ok(());
    };

    let mut values = Vec::with_capacity(args.0.len() + 1);
    values.push(string_value(
        ctx.clone(),
        format!("{}: {:.3}ms", label, elapsed_ms(start)).as_str(),
    )?);
    values.extend(args.0);
    write_console_values(stdout(), &ctx, values)
}

fn console_time_end(ctx: Ctx<'_>, label: Opt<Coerced<String>>) -> Result<()> {
    let label = get_label(label);
    let Some(state) = ctx.userdata::<ConsoleState>() else {
        return Ok(());
    };
    let Some(start) = state.timers.borrow_mut().remove(&label) else {
        writeln_ignore(stderr(), &format!("Timer '{}' does not exist", label));
        return Ok(());
    };
    writeln_ignore(stdout(), &format!("{}: {:.3}ms", label, elapsed_ms(start)));
    Ok(())
}

fn console_group<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<()> {
    if !args.0.is_empty() {
        write_console(stdout(), &ctx, args)?;
    }
    increase_group_indent(&ctx);
    Ok(())
}

fn console_group_collapsed<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<()> {
    console_group(ctx, args)
}

fn console_group_end(ctx: Ctx<'_>) {
    let Some(state) = ctx.userdata::<ConsoleState>() else {
        return;
    };
    let mut indent = state.group_indent.borrow_mut();
    let len = indent.len();
    if len >= GROUP_INDENT.len() {
        indent.truncate(len - GROUP_INDENT.len());
    }
}

fn console_trace<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<()> {
    let mut values = Vec::with_capacity(args.0.len() + 1);
    values.push(string_value(ctx.clone(), "Trace:")?);
    values.extend(args.0);
    write_console_values(stderr(), &ctx, values)?;

    if let Ok(stack) = ctx.eval::<String, _>("new Error().stack || ''") {
        for line in stack.lines().skip(1) {
            writeln_with_indent(stderr(), &ctx, line);
        }
    }
    Ok(())
}

fn console_dir<'js>(ctx: Ctx<'js>, value: Value<'js>, _options: Rest<Value<'js>>) -> Result<()> {
    write_console_values(stdout(), &ctx, vec![value])
}

fn console_dirxml<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<()> {
    write_console(stdout(), &ctx, args)
}

fn console_table<'js>(ctx: Ctx<'js>, data: Value<'js>, _columns: Rest<Value<'js>>) -> Result<()> {
    let mut lines = vec!["(index) | Value".to_string(), "--------|--------".to_string()];

    if let Some(array) = data.as_array() {
        let len = array.len();
        for index in 0..len.min(MAX_ARRAY_ITEMS) {
            let value = array
                .get::<Option<Value>>(index)?
                .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
            lines.push(format!(
                "{} | {}",
                index,
                inspect_value(&ctx, &value, false)?
            ));
        }
        if len > MAX_ARRAY_ITEMS {
            lines.push(format!("... | {} more items", len - MAX_ARRAY_ITEMS));
        }
    } else if let Some(object) = data.as_object() {
        let mut count = 0usize;
        for key in object.keys::<String>().flatten() {
            if count >= MAX_OBJECT_KEYS {
                lines.push("... | more properties".to_string());
                break;
            }
            let value: Value = object.get(key.as_str())?;
            lines.push(format!(
                "{} | {}",
                key,
                inspect_value(&ctx, &value, false)?
            ));
            count += 1;
        }
    } else {
        lines.truncate(0);
        lines.push(inspect_value(&ctx, &data, false)?);
    }

    for line in lines {
        writeln_with_indent(stdout(), &ctx, &line);
    }
    Ok(())
}

fn console_noop() {}

fn write_console<'js, W>(output: W, ctx: &Ctx<'js>, args: Rest<Value<'js>>) -> Result<()>
where
    W: Write + IsTerminal,
{
    write_console_values(output, ctx, args.0)
}

fn write_console_values<'js, W>(mut output: W, ctx: &Ctx<'js>, args: Vec<Value<'js>>) -> Result<()>
where
    W: Write + IsTerminal,
{
    let text = format_console_values(ctx, args, output.is_terminal())?;
    let indent = current_group_indent(ctx);
    for line in text.lines() {
        let _ = writeln!(output, "{}{}", indent, line);
    }
    if text.is_empty() {
        let _ = writeln!(output);
    }
    let _ = output.flush();
    Ok(())
}

fn format_console_values<'js>(
    ctx: &Ctx<'js>,
    args: Vec<Value<'js>>,
    color: bool,
) -> Result<String> {
    if args.is_empty() {
        return Ok(String::new());
    }

    let mut result = String::new();
    let mut next_index = 0usize;

    if args.len() > 1 {
        if let Some(first) = args[0].as_string() {
            let first = first.to_string()?;
            if first.contains('%') {
                result.push_str(&format_with_substitutions(ctx, &first, &args, &mut next_index)?);
            }
        }
    }

    if next_index == 0 {
        result.push_str(&inspect_value(ctx, &args[0], true)?);
        next_index = 1;
    }

    while next_index < args.len() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&inspect_value(ctx, &args[next_index], true)?);
        next_index += 1;
    }

    if color {
        Ok(result)
    } else {
        Ok(result)
    }
}

fn format_with_substitutions<'js>(
    ctx: &Ctx<'js>,
    format: &str,
    args: &[Value<'js>],
    next_index: &mut usize,
) -> Result<String> {
    let mut result = String::new();
    let mut chars = format.chars().peekable();
    *next_index = 1;

    while let Some(ch) = chars.next() {
        if ch != '%' {
            result.push(ch);
            continue;
        }

        let Some(spec) = chars.next() else {
            result.push('%');
            break;
        };

        if spec == '%' {
            result.push('%');
            continue;
        }

        if *next_index >= args.len() {
            result.push('%');
            result.push(spec);
            continue;
        }

        let value = &args[*next_index];
        *next_index += 1;

        match spec {
            's' => result.push_str(&to_js_string(value)),
            'd' | 'f' => result.push_str(&to_number_string(value, false)),
            'i' => result.push_str(&to_number_string(value, true)),
            'j' => result.push_str(
                json_stringify(ctx, value.clone())?
                    .unwrap_or_else(|| "undefined".to_string())
                    .as_str(),
            ),
            'o' | 'O' => result.push_str(&inspect_value(ctx, value, false)?),
            'c' => {},
            other => {
                result.push('%');
                result.push(other);
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(&inspect_value(ctx, value, true)?);
            },
        }
    }

    Ok(result)
}

fn inspect_value<'js>(ctx: &Ctx<'js>, value: &Value<'js>, root: bool) -> Result<String> {
    match value.type_of() {
        Type::Uninitialized | Type::Null => Ok("null".to_string()),
        Type::Undefined => Ok("undefined".to_string()),
        Type::Bool => Ok(value.as_bool().unwrap_or(false).to_string()),
        Type::Int | Type::Float => Ok(to_number_string(value, false)),
        Type::BigInt => Ok(format!("{}n", to_js_string(value))),
        Type::String => {
            let text = value
                .as_string()
                .and_then(|s| s.to_string().ok())
                .unwrap_or_default();
            if root {
                Ok(text)
            } else {
                Ok(format!("{:?}", text))
            }
        },
        Type::Symbol => format_symbol(value),
        Type::Function | Type::Constructor => format_function(value),
        Type::Array => format_array(ctx, value),
        Type::Object | Type::Exception | Type::Proxy | Type::Promise => format_object(ctx, value),
        _ => Ok(to_js_string(value)),
    }
}

fn format_symbol(value: &Value<'_>) -> Result<String> {
    if let Some(symbol) = value.as_symbol() {
        if let Ok(description) = symbol.description() {
            if let Ok(description) = description.get::<String>() {
                return Ok(format!("Symbol({})", description));
            }
        }
    }
    Ok("Symbol()".to_string())
}

fn format_function(value: &Value<'_>) -> Result<String> {
    let Some(object) = value.as_object() else {
        return Ok("[function]".to_string());
    };
    let name = object
        .get::<_, String>(PredefinedAtom::Name)
        .unwrap_or_default();
    if name.is_empty() {
        Ok("[function: (anonymous)]".to_string())
    } else {
        Ok(format!("[function: {}]", name))
    }
}

fn format_array<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<String> {
    let Some(array) = value.as_array() else {
        return Ok("[]".to_string());
    };
    let len = array.len();
    let mut parts = Vec::new();
    for index in 0..len.min(MAX_ARRAY_ITEMS) {
        let value = array
            .get::<Option<Value>>(index)?
            .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
        parts.push(inspect_value(ctx, &value, false)?);
    }
    if len > MAX_ARRAY_ITEMS {
        parts.push(format!("... {} more items", len - MAX_ARRAY_ITEMS));
    }
    Ok(format!("[{}]", parts.join(", ")))
}

fn format_object<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<String> {
    let Some(object) = value.as_object() else {
        return Ok("[object Object]".to_string());
    };

    if let Ok(stack) = object.get::<_, String>("stack") {
        if !stack.is_empty() {
            return Ok(stack);
        }
    }

    if value.type_of() == Type::Promise {
        return Ok("Promise { <pending> }".to_string());
    }

    let mut parts = Vec::new();
    for key in object.keys::<String>().flatten().take(MAX_OBJECT_KEYS) {
        let item = object
            .get::<_, Value>(key.as_str())
            .map(|v| inspect_value(ctx, &v, false))
            .unwrap_or_else(|_| Ok("[Thrown]".to_string()))?;
        parts.push(format!("{}: {}", key, item));
    }

    if parts.is_empty() {
        Ok("{}".to_string())
    } else {
        Ok(format!("{{ {} }}", parts.join(", ")))
    }
}

fn to_js_string(value: &Value<'_>) -> String {
    match value.type_of() {
        Type::Uninitialized | Type::Null => "null".to_string(),
        Type::Undefined => "undefined".to_string(),
        Type::String => value
            .as_string()
            .and_then(|s| s.to_string().ok())
            .unwrap_or_default(),
        Type::Symbol => format_symbol(value).unwrap_or_else(|_| "Symbol()".to_string()),
        _ => value
            .get::<Coerced<String>>()
            .map(|s| s.0)
            .unwrap_or_else(|_| "[object Object]".to_string()),
    }
}

fn to_number_string(value: &Value<'_>, integer: bool) -> String {
    let Ok(number) = value.get::<Coerced<f64>>() else {
        return "NaN".to_string();
    };
    let number = number.0;
    if number.is_nan() {
        return "NaN".to_string();
    }
    if number.is_infinite() {
        return if number.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    if integer {
        return (number as i64).to_string();
    }
    if number.fract() == 0.0 {
        (number as i64).to_string()
    } else {
        number.to_string()
    }
}

fn is_truthy(value: &Value<'_>) -> bool {
    match value.type_of() {
        Type::Uninitialized | Type::Null | Type::Undefined => false,
        Type::Bool => value.as_bool().unwrap_or(false),
        Type::Int | Type::Float => value.as_number().is_some_and(|n| n != 0.0 && !n.is_nan()),
        Type::String => value
            .as_string()
            .and_then(|s| s.to_string().ok())
            .is_some_and(|s| !s.is_empty()),
        _ => true,
    }
}

fn get_label(label: Opt<Coerced<String>>) -> String {
    label
        .0
        .map(|label| label.0)
        .unwrap_or_else(|| DEFAULT_LABEL.to_string())
}

fn current_group_indent(ctx: &Ctx<'_>) -> String {
    ctx.userdata::<ConsoleState>()
        .map(|state| state.group_indent.borrow().clone())
        .unwrap_or_default()
}

fn increase_group_indent(ctx: &Ctx<'_>) {
    let Some(state) = ctx.userdata::<ConsoleState>() else {
        return;
    };
    state.group_indent.borrow_mut().push_str(GROUP_INDENT);
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn string_value<'js>(ctx: Ctx<'js>, value: &str) -> Result<Value<'js>> {
    Ok(rquickjs::String::from_str(ctx, value)?.into_value())
}

fn writeln_ignore<W: Write>(mut output: W, line: &str) {
    let _ = writeln!(output, "{}", line);
    let _ = output.flush();
}

fn writeln_with_indent<W>(mut output: W, ctx: &Ctx<'_>, line: &str)
where
    W: Write + IsTerminal,
{
    let indent = current_group_indent(ctx);
    let _ = writeln!(output, "{}{}", indent, line);
    let _ = output.flush();
}

use rquickjs::function::Rest;
use rquickjs::module::{Declarations, Exports, ModuleDef};
use rquickjs::{Ctx, Function, Object, Value};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

static COUNTERS: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();

fn timers() -> &'static Mutex<HashMap<String, Instant>> {
    static TIMERS: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    TIMERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn init(ctx: &Ctx) -> Result<(), rquickjs::Error> {
    let console = Object::new(ctx.clone())?;

    console.set("log", Function::new(ctx.clone(), console_log))?;
    console.set("info", Function::new(ctx.clone(), console_log))?;
    console.set("debug", Function::new(ctx.clone(), console_log))?;
    console.set("warn", Function::new(ctx.clone(), console_error))?;
    console.set("error", Function::new(ctx.clone(), console_error))?;
    console.set("time", Function::new(ctx.clone(), console_time))?;
    console.set("timeLog", Function::new(ctx.clone(), console_time_log))?;
    console.set("timeEnd", Function::new(ctx.clone(), console_time_end))?;
    console.set(
        "table",
        Function::new(ctx.clone(), |data: Value| {
            println!("{}", format_js_value(&data));
        }),
    )?;
    console.set("count", Function::new(ctx.clone(), console_count))?;
    console.set(
        "countReset",
        Function::new(ctx.clone(), console_count_reset),
    )?;
    console.set("assert", Function::new(ctx.clone(), console_assert))?;
    console.set(
        "clear",
        Function::new(ctx.clone(), || print!("\x1B[2J\x1B[1;1H")),
    )?;

    ctx.globals().set("console", console)?;

    // globalThis is the only global scope identifier in a pure ECMAScript runtime.
    // Browser-specific globals like navigator/self/window are intentionally not exposed.

    // Location stub
    let location = Object::new(ctx.clone())?;
    location.set("href", "file://")?;
    location.set("protocol", "file:")?;
    location.set("host", "")?;
    location.set("hostname", "")?;
    location.set("port", "")?;
    location.set("pathname", "")?;
    location.set("search", "")?;
    location.set("hash", "")?;
    location.set("origin", "file://")?;
    ctx.globals().set("location", location)?;

    ctx.eval::<(), _>(
        r#"
        (function() {
            console.table = function(data, columns) {
                if (Array.isArray(data)) {
                    console.log("(index) | Value");
                    console.log("--------|--------");
                    data.forEach(function(item, index) {
                        if (item !== null && typeof item === "object") {
                            console.log(index + " | " + JSON.stringify(item));
                        } else {
                            console.log(index + " | " + item);
                        }
                    });
                } else if (data !== null && typeof data === "object") {
                    console.log("(index) | Value");
                    console.log("--------|--------");
                    Object.entries(data).forEach(function(entry) {
                        var key = entry[0];
                        var value = entry[1];
                        if (value !== null && typeof value === "object") {
                            console.log(key + " | " + JSON.stringify(value));
                        } else {
                            console.log(key + " | " + value);
                        }
                    });
                } else {
                    console.log(data);
                }
            };

            var groupIndent = "";
            var originalLog = console.log;
            var originalInfo = console.info;
            var originalWarn = console.warn;
            var originalError = console.error;

            function indentLog(fn) {
                return function() {
                    var args = Array.prototype.slice.call(arguments);
                    if (args.length > 0 && typeof args[0] === "string") {
                        args[0] = groupIndent + args[0];
                    } else if (groupIndent) {
                        args.unshift(groupIndent);
                    }
                    fn.apply(this, args);
                };
            }

            console.log = indentLog(originalLog);
            console.info = indentLog(originalInfo);
            console.warn = indentLog(originalWarn);
            console.error = indentLog(originalError);

            console.group = function() {
                var args = Array.prototype.slice.call(arguments);
                console.log.apply(console, args);
                groupIndent += "  ";
            };
            console.groupEnd = function() {
                groupIndent = groupIndent.slice(0, -2);
            };
            console.groupCollapsed = console.group;

            console.trace = function() {
                var args = Array.prototype.slice.call(arguments);
                console.log("Trace: " + (args.length > 0 ? args.join(" ") : ""));
                var err = new Error();
                if (err.stack) {
                    console.log(err.stack);
                }
            };

            console.dir = function(obj) {
                if (obj !== null && typeof obj === "object") {
                    console.log(JSON.stringify(obj, null, 2));
                } else {
                    console.log(obj);
                }
            };
            console.dirxml = console.log;

            console.profile = function() {};
            console.profileEnd = function() {};
            console.timeStamp = function(label) {};

            console.memory = {
                jsHeapSizeLimit: 2190000000,
                totalJSHeapSize: 0,
                usedJSHeapSize: 0
            };
        })();
    "#,
    )?;

    Ok(())
}

fn console_log(_ctx: Ctx, args: Rest<Value>) {
    let parts: Vec<String> = args.0.iter().map(format_js_value).collect();
    println!("{}", parts.join(" "));
}

fn console_error(_ctx: Ctx, args: Rest<Value>) {
    let parts: Vec<String> = args.0.iter().map(format_js_value).collect();
    eprintln!("{}", parts.join(" "));
}

fn console_time(_ctx: Ctx, label: Option<String>) {
    let label = label.unwrap_or_else(|| "default".to_string());
    timers().lock().unwrap().insert(label, Instant::now());
}

fn console_time_log(_ctx: Ctx, label: Option<String>) {
    let label = label.unwrap_or_else(|| "default".to_string());
    let timers = timers().lock().unwrap();
    if let Some(start) = timers.get(&label) {
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        println!("{}: {:.3}ms", label, elapsed);
    } else {
        eprintln!(
            "warn: console.timeLog('{}') called without preceding console.time('{}')",
            label, label
        );
    }
}

fn console_time_end(_ctx: Ctx, label: Option<String>) {
    let label = label.unwrap_or_else(|| "default".to_string());
    let mut timers = timers().lock().unwrap();
    if let Some(start) = timers.remove(&label) {
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        println!("{}: {:.3}ms", label, elapsed);
    } else {
        eprintln!(
            "warn: console.timeEnd('{}') called without preceding console.time('{}')",
            label, label
        );
    }
}

fn console_count(label: Option<String>) {
    let label = label.unwrap_or_else(|| "default".to_string());
    let mut counters = COUNTERS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();
    let count = counters.entry(label.clone()).or_insert(0);
    *count += 1;
    println!("{}: {}", label, *count);
}

fn console_count_reset(label: Option<String>) {
    let label = label.unwrap_or_else(|| "default".to_string());
    let mut counters = COUNTERS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();
    counters.remove(&label);
}

fn console_assert(_ctx: Ctx, condition: bool, args: Rest<Value>) {
    if !condition {
        let mut parts = vec!["Assertion failed:".to_string()];
        parts.extend(args.0.iter().map(format_js_value));
        eprintln!("{}", parts.join(" "));
    }
}

fn format_js_value(value: &Value) -> String {
    if value.is_undefined() {
        "undefined".to_string()
    } else if value.is_null() {
        "null".to_string()
    } else if value.is_bool() {
        value.as_bool().unwrap().to_string()
    } else if value.is_number() {
        value.as_number().unwrap().to_string()
    } else if value.is_string() {
        value
            .as_string()
            .and_then(|s| s.to_string().ok())
            .unwrap_or_else(|| "string".to_string())
    } else if value.is_array() {
        "[array]".to_string()
    } else if value.is_object() {
        "[object Object]".to_string()
    } else {
        "unknown".to_string()
    }
}

pub struct ConsoleRawModule;

impl ModuleDef for ConsoleRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("__console_log")?;
        decl.declare("__console_error")?;
        decl.declare("__console_warn")?;
        decl.declare("__console_info")?;
        decl.declare("__console_debug")?;
        decl.declare("__console_time")?;
        decl.declare("__console_time_log")?;
        decl.declare("__console_time_end")?;
        decl.declare("__console_count")?;
        decl.declare("__console_count_reset")?;
        decl.declare("__console_assert")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> rquickjs::Result<()> {
        exports.export("__console_log", Function::new(ctx.clone(), console_log))?;
        exports.export("__console_error", Function::new(ctx.clone(), console_error))?;
        exports.export("__console_warn", Function::new(ctx.clone(), console_error))?;
        exports.export("__console_info", Function::new(ctx.clone(), console_log))?;
        exports.export("__console_debug", Function::new(ctx.clone(), console_log))?;
        exports.export("__console_time", Function::new(ctx.clone(), console_time))?;
        exports.export(
            "__console_time_log",
            Function::new(ctx.clone(), console_time_log),
        )?;
        exports.export(
            "__console_time_end",
            Function::new(ctx.clone(), console_time_end),
        )?;
        exports.export("__console_count", Function::new(ctx.clone(), console_count))?;
        exports.export(
            "__console_count_reset",
            Function::new(ctx.clone(), console_count_reset),
        )?;
        exports.export(
            "__console_assert",
            Function::new(ctx.clone(), console_assert),
        )?;
        Ok(())
    }
}

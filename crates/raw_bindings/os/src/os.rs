use rquickjs::{
    Ctx, Exception, Function, Result,
    module::{Declarations, Exports, ModuleDef},
};
use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

static START_TIME: OnceLock<Instant> = OnceLock::new();
static EXIT_CODE: AtomicI32 = AtomicI32::new(0);

fn normalize_exit_code(code: i32) -> i32 {
    code.rem_euclid(256)
}

fn process_exit_code() -> i32 {
    EXIT_CODE.load(Ordering::Relaxed)
}

fn process_set_exit_code(code: i32) {
    EXIT_CODE.store(normalize_exit_code(code), Ordering::Relaxed);
}

fn process_exit(code: Option<i32>) -> rquickjs::Result<()> {
    let exit_code = match code {
        Some(code) => normalize_exit_code(code),
        None => process_exit_code(),
    };
    std::process::exit(exit_code)
}

fn process_abort() -> rquickjs::Result<()> {
    std::process::abort()
}

fn process_cwd() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string())
}

fn process_chdir<'js>(ctx: Ctx<'js>, path: String) -> Result<()> {
    std::env::set_current_dir(&path).map_err(|err| {
        Exception::throw_type(&ctx, &format!("chdir failed for path '{path}': {err}"))
    })
}

fn process_uptime() -> f64 {
    START_TIME.get_or_init(Instant::now).elapsed().as_secs_f64()
}

fn process_memory_usage() -> HashMap<String, u64> {
    let mut map = HashMap::new();
    let pid = std::process::id();

    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("ps")
            .args(["-o", "rss=,vsize=", "-p", &pid.to_string()])
            .output()
        {
            if output.status.success() {
                let output = String::from_utf8_lossy(&output.stdout);
                let mut parts = output
                    .split_whitespace()
                    .filter_map(|part| part.parse::<u64>().ok());
                let rss_kib = parts.next().unwrap_or(0);
                let vsize_kib = parts.next().unwrap_or(0);
                map.insert("rss".to_string(), rss_kib.saturating_mul(1024));
                map.insert("heapTotal".to_string(), vsize_kib.saturating_mul(1024));
                map.insert("heapUsed".to_string(), rss_kib.saturating_mul(1024));
            }
        }
    }

    if !map.contains_key("rss") {
        map.insert("rss".to_string(), 0);
    }
    if !map.contains_key("heapTotal") {
        map.insert("heapTotal".to_string(), 0);
    }
    if !map.contains_key("heapUsed") {
        map.insert("heapUsed".to_string(), 0);
    }
    map.insert("external".to_string(), 0);
    map
}

fn process_args() -> Vec<String> {
    std::env::args().skip(1).collect()
}

fn process_argv0() -> String {
    std::env::args().next().unwrap_or_default()
}

fn process_pid() -> u32 {
    std::process::id()
}

fn process_ppid() -> u32 {
    let pid = std::process::id();

    #[cfg(unix)]
    if let Ok(output) = Command::new("ps")
        .args(["-o", "ppid=", "-p", &pid.to_string()])
        .output()
    {
        if output.status.success() {
            if let Ok(ppid) = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
                .unwrap_or("0")
                .parse::<u32>()
            {
                return ppid;
            }
        }
    }

    0
}

fn process_platform() -> String {
    std::env::consts::OS.to_string()
}

fn process_arch() -> String {
    std::env::consts::ARCH.to_string()
}

fn process_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn process_versions() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("vole".to_string(), env!("CARGO_PKG_VERSION").to_string());
    map
}

fn process_stdout_write(data: String) {
    print!("{}", data);
}

fn process_stdout_write_async(data: String) {
    print!("{}", data);
}

fn process_stdout_flush() {
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

fn process_stderr_write(data: String) {
    eprint!("{}", data);
}

fn process_stderr_write_async(data: String) {
    eprint!("{}", data);
}

fn process_stderr_flush() {
    let _ = std::io::Write::flush(&mut std::io::stderr());
}

/// Native module for process metadata and OS-global helpers.
pub struct OsRawModule;

impl ModuleDef for OsRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_os_exit")?;
        decl.declare("__std_os_abort")?;
        decl.declare("__std_os_cwd")?;
        decl.declare("__std_os_chdir")?;
        decl.declare("__std_os_uptime")?;
        decl.declare("__std_os_memoryUsage")?;
        decl.declare("__std_os_args")?;
        decl.declare("__std_os_argv0")?;
        decl.declare("__std_os_pid")?;
        decl.declare("__std_os_ppid")?;
        decl.declare("__std_os_platform")?;
        decl.declare("__std_os_arch")?;
        decl.declare("__std_os_version")?;
        decl.declare("__std_os_versions")?;
        decl.declare("__std_os_getExitCode")?;
        decl.declare("__std_os_setExitCode")?;
        decl.declare("__std_os_stdout_write")?;
        decl.declare("__std_os_stdout_writeAsync")?;
        decl.declare("__std_os_stdout_flush")?;
        decl.declare("__std_os_stderr_write")?;
        decl.declare("__std_os_stderr_writeAsync")?;
        decl.declare("__std_os_stderr_flush")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__std_os_exit", Function::new(ctx.clone(), process_exit))?;
        exports.export("__std_os_abort", Function::new(ctx.clone(), process_abort))?;
        exports.export("__std_os_cwd", Function::new(ctx.clone(), process_cwd))?;
        exports.export("__std_os_chdir", Function::new(ctx.clone(), process_chdir))?;
        exports.export(
            "__std_os_uptime",
            Function::new(ctx.clone(), process_uptime),
        )?;
        exports.export(
            "__std_os_memoryUsage",
            Function::new(ctx.clone(), process_memory_usage),
        )?;
        exports.export("__std_os_args", Function::new(ctx.clone(), process_args))?;
        exports.export("__std_os_argv0", Function::new(ctx.clone(), process_argv0))?;
        exports.export("__std_os_pid", Function::new(ctx.clone(), process_pid))?;
        exports.export("__std_os_ppid", Function::new(ctx.clone(), process_ppid))?;
        exports.export(
            "__std_os_platform",
            Function::new(ctx.clone(), process_platform),
        )?;
        exports.export("__std_os_arch", Function::new(ctx.clone(), process_arch))?;
        exports.export(
            "__std_os_version",
            Function::new(ctx.clone(), process_version),
        )?;
        exports.export(
            "__std_os_versions",
            Function::new(ctx.clone(), process_versions),
        )?;
        exports.export(
            "__std_os_getExitCode",
            Function::new(ctx.clone(), process_exit_code),
        )?;
        exports.export(
            "__std_os_setExitCode",
            Function::new(ctx.clone(), process_set_exit_code),
        )?;
        exports.export(
            "__std_os_stdout_write",
            Function::new(ctx.clone(), process_stdout_write),
        )?;
        exports.export(
            "__std_os_stdout_writeAsync",
            Function::new(ctx.clone(), process_stdout_write_async),
        )?;
        exports.export(
            "__std_os_stdout_flush",
            Function::new(ctx.clone(), process_stdout_flush),
        )?;
        exports.export(
            "__std_os_stderr_write",
            Function::new(ctx.clone(), process_stderr_write),
        )?;
        exports.export(
            "__std_os_stderr_writeAsync",
            Function::new(ctx.clone(), process_stderr_write_async),
        )?;
        exports.export(
            "__std_os_stderr_flush",
            Function::new(ctx.clone(), process_stderr_flush),
        )?;
        Ok(())
    }
}

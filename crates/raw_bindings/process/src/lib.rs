use rquickjs::{
    function::Opt,
    module::{Declarations, Exports, ModuleDef},
    Ctx, Function, Object, Result,
};
use std::collections::HashMap;
use std::time::Instant;

static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn process_exit(code: Option<i32>) {
    std::process::exit(code.unwrap_or(0));
}

fn process_abort() {
    std::process::abort();
}

fn process_cwd() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string())
}

fn process_chdir(path: String) -> Result<()> {
    std::env::set_current_dir(&path).map_err(|_| rquickjs::Error::Exception)
}

fn process_uptime() -> f64 {
    START_TIME.get_or_init(Instant::now).elapsed().as_secs_f64()
}

fn process_memory_usage() -> HashMap<String, u64> {
    let mut map = HashMap::new();
    map.insert("rss".to_string(), 0);
    map.insert("heapTotal".to_string(), 0);
    map.insert("heapUsed".to_string(), 0);
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

fn exec_sync_impl<'js>(
    ctx: Ctx<'js>,
    command: String,
    options: Opt<Object<'js>>,
) -> Result<Object<'js>> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", &command]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", &command]);
        c
    };

    if let Some(opts) = options.0 {
        if let Ok(cwd) = opts.get::<&str, String>("cwd") {
            cmd.current_dir(&cwd);
        }
    }

    let output = cmd.output().map_err(|_| rquickjs::Error::Exception)?;
    let obj = Object::new(ctx.clone())?;
    obj.set(
        "stdout",
        String::from_utf8_lossy(&output.stdout).to_string(),
    )?;
    obj.set(
        "stderr",
        String::from_utf8_lossy(&output.stderr).to_string(),
    )?;
    let status_obj = Object::new(ctx.clone())?;
    status_obj.set("success", output.status.success())?;
    status_obj.set("code", output.status.code().unwrap_or(-1) as f64)?;
    obj.set("status", status_obj)?;
    Ok(obj)
}

fn spawn_impl<'js>(
    ctx: Ctx<'js>,
    command: String,
    args: Opt<Vec<String>>,
    options: Opt<Object<'js>>,
) -> Result<Object<'js>> {
    let mut cmd = std::process::Command::new(&command);
    if let Some(a) = args.0 {
        cmd.args(&a);
    }

    if let Some(opts) = options.0 {
        if let Ok(cwd) = opts.get::<&str, String>("cwd") {
            cmd.current_dir(&cwd);
        }
    }

    let mut child = cmd.spawn().map_err(|_| rquickjs::Error::Exception)?;
    let pid = child.id();

    // Detach the child process to avoid blocking
    std::thread::spawn(move || {
        let _ = child.wait();
    });

    let obj = Object::new(ctx.clone())?;
    obj.set("pid", pid as f64)?;

    // kill method
    let kill_fn = Function::new(ctx.clone(), |_signal: Option<String>| -> bool { false })?;
    obj.set("kill", kill_fn)?;

    Ok(obj)
}

/// Native module exposing the low-level process operations used by the `vole:process` JS shim
/// and related modules (`vole:shell`, `vole:spawn`).
pub struct ProcessRawModule;

impl ModuleDef for ProcessRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_process_exit")?;
        decl.declare("__std_process_abort")?;
        decl.declare("__std_process_cwd")?;
        decl.declare("__std_process_chdir")?;
        decl.declare("__std_process_uptime")?;
        decl.declare("__std_process_memoryUsage")?;
        decl.declare("__std_process_args")?;
        decl.declare("__std_process_argv0")?;
        decl.declare("__std_process_pid")?;
        decl.declare("__std_process_ppid")?;
        decl.declare("__std_process_platform")?;
        decl.declare("__std_process_arch")?;
        decl.declare("__std_process_version")?;
        decl.declare("__std_process_versions")?;
        decl.declare("__std_process_stdout_write")?;
        decl.declare("__std_process_stdout_writeAsync")?;
        decl.declare("__std_process_stdout_flush")?;
        decl.declare("__std_process_stderr_write")?;
        decl.declare("__std_process_stderr_writeAsync")?;
        decl.declare("__std_process_stderr_flush")?;
        decl.declare("__std_process_execSync")?;
        decl.declare("__std_process_exec")?;
        decl.declare("__std_process_spawn")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export(
            "__std_process_exit",
            Function::new(ctx.clone(), process_exit),
        )?;
        exports.export(
            "__std_process_abort",
            Function::new(ctx.clone(), process_abort),
        )?;
        exports.export("__std_process_cwd", Function::new(ctx.clone(), process_cwd))?;
        exports.export(
            "__std_process_chdir",
            Function::new(ctx.clone(), process_chdir),
        )?;
        exports.export(
            "__std_process_uptime",
            Function::new(ctx.clone(), process_uptime),
        )?;
        exports.export(
            "__std_process_memoryUsage",
            Function::new(ctx.clone(), process_memory_usage),
        )?;
        exports.export(
            "__std_process_args",
            Function::new(ctx.clone(), process_args),
        )?;
        exports.export(
            "__std_process_argv0",
            Function::new(ctx.clone(), process_argv0),
        )?;
        exports.export("__std_process_pid", Function::new(ctx.clone(), process_pid))?;
        exports.export(
            "__std_process_ppid",
            Function::new(ctx.clone(), process_ppid),
        )?;
        exports.export(
            "__std_process_platform",
            Function::new(ctx.clone(), process_platform),
        )?;
        exports.export(
            "__std_process_arch",
            Function::new(ctx.clone(), process_arch),
        )?;
        exports.export(
            "__std_process_version",
            Function::new(ctx.clone(), process_version),
        )?;
        exports.export(
            "__std_process_versions",
            Function::new(ctx.clone(), process_versions),
        )?;
        exports.export(
            "__std_process_stdout_write",
            Function::new(ctx.clone(), process_stdout_write),
        )?;
        exports.export(
            "__std_process_stdout_writeAsync",
            Function::new(ctx.clone(), process_stdout_write_async),
        )?;
        exports.export(
            "__std_process_stdout_flush",
            Function::new(ctx.clone(), process_stdout_flush),
        )?;
        exports.export(
            "__std_process_stderr_write",
            Function::new(ctx.clone(), process_stderr_write),
        )?;
        exports.export(
            "__std_process_stderr_writeAsync",
            Function::new(ctx.clone(), process_stderr_write_async),
        )?;
        exports.export(
            "__std_process_stderr_flush",
            Function::new(ctx.clone(), process_stderr_flush),
        )?;
        exports.export(
            "__std_process_execSync",
            Function::new(ctx.clone(), exec_sync_impl),
        )?;
        exports.export(
            "__std_process_exec",
            Function::new(ctx.clone(), exec_sync_impl),
        )?;
        exports.export(
            "__std_process_spawn",
            Function::new(ctx.clone(), spawn_impl),
        )?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `vole:process`.
/// All process helpers are provided through the native module `vole:raw:process`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

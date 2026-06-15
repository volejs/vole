use rquickjs::{
    Ctx,
    Exception,
    Function,
    Object,
    Result,
    function::Opt,
    module::{Declarations, Exports, ModuleDef},
};
use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

type SharedHandleTable = Arc<Mutex<HashMap<u64, SpawnedProcessHandle>>>;
type ProcessHandleMap = HashMap<u64, SpawnedProcessHandle>;

#[derive(Clone)]
enum ParsedSignal {
    Default,
    Check,
    Signal(String),
    Invalid,
}

#[derive(Clone)]
struct SpawnedProcessHandle {
    pid: u32,
}

static PROCESS_HANDLE_TABLE: OnceLock<SharedHandleTable> = OnceLock::new();
static PROCESS_HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);

fn process_handle_table() -> SharedHandleTable {
    PROCESS_HANDLE_TABLE
        .get_or_init(|| Arc::new(Mutex::new(ProcessHandleMap::new())))
        .clone()
}

fn next_handle_id() -> u64 {
    PROCESS_HANDLE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn parse_signal(signal: Option<&str>) -> ParsedSignal {
    fn is_numeric_signal(raw: &str) -> bool {
        raw.chars().all(|c| c.is_ascii_digit())
    }

    let Some(raw) = signal else {
        return ParsedSignal::Default;
    };

    let raw = raw.trim();
    if raw.is_empty() {
        return ParsedSignal::Default;
    }

    let raw = raw
        .trim_start_matches("SIG")
        .trim_start_matches('-')
        .to_ascii_uppercase();
    if raw == "0" {
        return ParsedSignal::Check;
    }
    if raw.is_empty() {
        return ParsedSignal::Invalid;
    }

    let allowed = is_numeric_signal(&raw)
        || matches!(
            raw.as_str(),
            "ABRT"
                | "ALRM"
                | "FPE"
                | "HUP"
                | "ILL"
                | "INT"
                | "KILL"
                | "PIPE"
                | "QUIT"
                | "SEGV"
                | "TERM"
                | "USR1"
                | "USR2"
                | "CONT"
                | "STOP"
                | "TSTP"
                | "TTIN"
                | "TTOU"
        );
    if allowed {
        ParsedSignal::Signal(raw)
    } else {
        ParsedSignal::Invalid
    }
}

fn is_kill_signal(signal: &ParsedSignal) -> bool {
    !matches!(signal, ParsedSignal::Check)
}

fn kill_process(pid: u32, signal: ParsedSignal) -> bool {
    match signal {
        ParsedSignal::Invalid => return false,
        _ => {}
    }

    #[cfg(unix)]
    {
        let mut cmd = Command::new("kill");
        match signal {
            ParsedSignal::Default => {}
            ParsedSignal::Check => {
                cmd.arg("-0");
            }
            ParsedSignal::Signal(sig) => {
                cmd.arg(format!("-{sig}"));
            }
            ParsedSignal::Invalid => unreachable!(),
        }
        return cmd
            .arg(pid.to_string())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
    }

    #[cfg(windows)]
    {
        match signal {
            ParsedSignal::Check => {
                let output = Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
                    .output();
                if let Ok(output) = output {
                    if !output.status.success() {
                        return false;
                    }
                    let text = String::from_utf8_lossy(&output.stdout);
                    text.lines()
                        .any(|line| line.contains(&format!("\"{pid}\"")))
                } else {
                    false
                }
            }
            ParsedSignal::Default | ParsedSignal::Signal(_) | ParsedSignal::Invalid => {
                let mut cmd = Command::new("taskkill");
                let _ = cmd.arg("/PID").arg(pid.to_string()).arg("/T").arg("/F");
                cmd.status().map(|status| status.success()).unwrap_or(false)
            }
        }
    }
}

fn kill_by_handle(handle_id: u64, signal: Option<String>) -> bool {
    let parsed_signal = parse_signal(signal.as_deref());
    if matches!(parsed_signal, ParsedSignal::Invalid) {
        return false;
    }

    let pid = {
        let table = process_handle_table();
        let table = table.lock().unwrap_or_else(|err| err.into_inner());
        match table.get(&handle_id) {
            Some(handle) => handle.pid,
            None => return false,
        }
    };

    let ok = kill_process(pid, parsed_signal.clone());
    if ok && is_kill_signal(&parsed_signal) {
        let table = process_handle_table();
        let mut table = table.lock().unwrap_or_else(|err| err.into_inner());
        let _ = table.remove(&handle_id);
    }

    ok
}

fn kill_pid(pid: u32, signal: Opt<String>) -> bool {
    kill_process(pid, parse_signal(signal.0.as_deref()))
}

fn exec_sync_impl<'js>(
    ctx: Ctx<'js>,
    command: String,
    options: Opt<Object<'js>>,
) -> Result<Object<'js>> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", &command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", &command]);
        c
    };

    if let Some(opts) = options.0 {
        if let Ok(cwd) = opts.get::<&str, String>("cwd") {
            cmd.current_dir(&cwd);
        }
    }

    let output = cmd
        .output()
        .map_err(|err| Exception::throw_type(&ctx, &format!("exec failed: {err}")))?;
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
    let mut cmd = Command::new(&command);
    if let Some(a) = args.0 {
        cmd.args(&a);
    }

    if let Some(opts) = options.0 {
        if let Ok(cwd) = opts.get::<&str, String>("cwd") {
            cmd.current_dir(&cwd);
        }
    }

    let mut child = cmd
        .spawn()
        .map_err(|err| Exception::throw_type(&ctx, &format!("spawn failed: {err}")))?;
    let pid = child.id();
    let handle_id = next_handle_id();

    let table = process_handle_table();
    {
        let mut table = table.lock().unwrap_or_else(|err| err.into_inner());
        table.insert(handle_id, SpawnedProcessHandle { pid });
    }

    let table_for_wait = process_handle_table();
    std::thread::spawn(move || {
        let _ = child.wait();
        let mut table = table_for_wait.lock().unwrap_or_else(|err| err.into_inner());
        let _ = table.remove(&handle_id);
    });

    let obj = Object::new(ctx.clone())?;
    obj.set("pid", pid as f64)?;
    let kill_fn = Function::new(ctx.clone(), move |_signal: Option<String>| -> bool {
        kill_by_handle(handle_id, _signal)
    })?;
    obj.set("kill", kill_fn)?;
    Ok(obj)
}

/// Native module for process execution, shell helpers and child-control.
pub struct OsExecRawModule;

impl ModuleDef for OsExecRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_os_exec")?;
        decl.declare("__std_os_execSync")?;
        decl.declare("__std_os_spawn")?;
        decl.declare("__std_os_kill")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__std_os_exec", Function::new(ctx.clone(), exec_sync_impl))?;
        exports.export(
            "__std_os_execSync",
            Function::new(ctx.clone(), exec_sync_impl),
        )?;
        exports.export("__std_os_spawn", Function::new(ctx.clone(), spawn_impl))?;
        exports.export("__std_os_kill", Function::new(ctx.clone(), kill_pid))?;
        Ok(())
    }
}

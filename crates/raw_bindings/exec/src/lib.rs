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
use std::process::{Command, Child};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use vole_raw_signal::{is_terminating_signal, parse_signal, send_signal, SignalSpec};

type SharedHandleTable = Arc<Mutex<HashMap<u64, SpawnedProcessHandle>>>;
type ProcessHandleMap = HashMap<u64, SpawnedProcessHandle>;

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

fn signal_by_handle(handle_id: u64, signal: Option<String>) -> bool {
    let parsed_signal = parse_signal(signal.as_deref());
    if matches!(parsed_signal, SignalSpec::Invalid) {
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

    let ok = send_signal(pid, parsed_signal.clone());
    if ok && is_terminating_signal(&parsed_signal) {
        let table = process_handle_table();
        let mut table = table.lock().unwrap_or_else(|err| err.into_inner());
        let _ = table.remove(&handle_id);
    }

    ok
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

    let mut child: Child = cmd
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
    let kill_fn = Function::new(ctx.clone(), move |signal: Option<String>| -> bool {
        signal_by_handle(handle_id, signal)
    })?;
    obj.set("kill", kill_fn)?;
    Ok(obj)
}

/// Native module for process execution and shell helpers.
pub struct OsExecRawModule;

impl ModuleDef for OsExecRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_os_exec")?;
        decl.declare("__std_os_execSync")?;
        decl.declare("__std_os_spawn")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__std_os_exec", Function::new(ctx.clone(), exec_sync_impl))?;
        exports.export(
            "__std_os_execSync",
            Function::new(ctx.clone(), exec_sync_impl),
        )?;
        exports.export("__std_os_spawn", Function::new(ctx.clone(), spawn_impl))?;
        Ok(())
    }
}

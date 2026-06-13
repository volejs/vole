use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    Ctx, Result,
};
use std::collections::HashMap;

/// 加载工作目录下的 `.env` 文件到进程环境变量。
pub fn load_dotenv() {
    let dotenv_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".env");

    if !dotenv_path.exists() {
        return;
    }

    let contents = match std::fs::read_to_string(&dotenv_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line = if line.starts_with("export ") {
            &line[7..]
        } else {
            line
        };

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            if !key.is_empty() {
                std::env::set_var(key, value);
            }
        }
    }
}

fn env_get(name: String) -> Option<String> {
    std::env::var(&name).ok()
}

fn env_set(name: String, value: String) {
    std::env::set_var(&name, &value);
}

fn env_has(name: String) -> bool {
    std::env::var(&name).is_ok()
}

fn env_remove(name: String) {
    std::env::remove_var(&name);
}

fn env_entries() -> HashMap<String, String> {
    std::env::vars().collect()
}

/// Native module exposing the low-level env operations used by the `vole:env` JS shim.
pub struct EnvRawModule;

impl ModuleDef for EnvRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_env_get")?;
        decl.declare("__std_env_set")?;
        decl.declare("__std_env_has")?;
        decl.declare("__std_env_remove")?;
        decl.declare("__std_env_entries")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__std_env_get", rquickjs::Function::new(ctx.clone(), env_get))?;
        exports.export("__std_env_set", rquickjs::Function::new(ctx.clone(), env_set))?;
        exports.export("__std_env_has", rquickjs::Function::new(ctx.clone(), env_has))?;
        exports.export("__std_env_remove", rquickjs::Function::new(ctx.clone(), env_remove))?;
        exports.export("__std_env_entries", rquickjs::Function::new(ctx.clone(), env_entries))?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `vole:env`.
/// All env helpers are provided through the native module `vole:raw:env`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

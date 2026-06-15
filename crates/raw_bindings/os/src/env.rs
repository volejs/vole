use dotenvy;
use rquickjs::{
    Ctx, Exception, Result,
    module::{Declarations, Exports, ModuleDef},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

fn resolve_env_file(path: String) -> PathBuf {
    PathBuf::from(path)
}

fn apply_dotenv_file(dotenv_path: &Path) -> StdResult<(), String> {
    let path_display = dotenv_path.to_string_lossy();
    let mut parsed = dotenvy::from_path_iter(dotenv_path)
        .map_err(|err| format!("failed to parse env file {path_display}: {err}"))?;

    for item in parsed.by_ref() {
        let (key, value) =
            item.map_err(|err| format!("failed to parse env entry in {path_display}: {err}"))?;
        // SAFETY: key/value come from dotenvy and the process environment API accepts owned
        // string types provided by the parser.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    Ok(())
}

/// 加载一个指定路径的 `.env` 文件到进程环境变量。
pub fn load_dotenv_with_path(ctx: &Ctx<'_>, path: String) -> Result<()> {
    let dotenv_path = resolve_env_file(path);
    let path_display = dotenv_path.to_string_lossy();

    if !dotenv_path.is_file() {
        if !dotenv_path.exists() {
            return Err(Exception::throw_type(
                ctx,
                &format!("load requires an existing file path: {path_display}"),
            ));
        }

        return Err(Exception::throw_type(
            ctx,
            &format!("load requires a file path: {path_display}"),
        ));
    }

    if let Err(err) = apply_dotenv_file(&dotenv_path) {
        return Err(Exception::throw_type(ctx, &err));
    }
    Ok(())
}

/// 兼容旧调用：加载当前工作目录下 `.env`。
pub fn load_dotenv() {
    let path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".env");
    let _ = apply_dotenv_file(&path);
}

fn env_load(ctx: Ctx, path: String) -> Result<()> {
    load_dotenv_with_path(&ctx, path)
}

fn validate_env_key<'js>(ctx: &Ctx<'js>, name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Exception::throw_type(
            ctx,
            "Environment variable name must not be empty",
        ));
    }

    if name.contains('\0') {
        return Err(Exception::throw_type(
            ctx,
            "Environment variable name must not contain NUL",
        ));
    }

    if name.contains('=') {
        return Err(Exception::throw_type(
            ctx,
            "Environment variable name must not contain '='",
        ));
    }

    Ok(())
}

fn validate_env_value<'js>(ctx: &Ctx<'js>, value: &str) -> Result<()> {
    if value.contains('\0') {
        return Err(Exception::throw_type(
            ctx,
            "Environment variable value must not contain NUL",
        ));
    }
    Ok(())
}

fn env_get(ctx: Ctx, name: String) -> Result<Option<String>> {
    validate_env_key(&ctx, &name)?;
    Ok(std::env::var(&name).ok())
}

fn env_set(ctx: Ctx, name: String, value: String) -> Result<()> {
    validate_env_key(&ctx, &name)?;
    validate_env_value(&ctx, &value)?;

    // SAFETY: name/value are owned Rust UTF-8 strings in current call frame.
    unsafe {
        std::env::set_var(&name, &value);
    }
    Ok(())
}

fn env_has(ctx: Ctx, name: String) -> Result<bool> {
    validate_env_key(&ctx, &name)?;
    Ok(std::env::var(&name).is_ok())
}

fn env_remove(ctx: Ctx, name: String) -> Result<()> {
    validate_env_key(&ctx, &name)?;
    // SAFETY: name is an owned Rust UTF-8 string in current call frame.
    unsafe {
        std::env::remove_var(&name);
    }
    Ok(())
}

fn env_entries() -> HashMap<String, String> {
    std::env::vars().collect()
}

/// Native module exposing the low-level env operations used by `vole:os/env`.
pub struct EnvRawModule;

impl ModuleDef for EnvRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_env_get")?;
        decl.declare("__std_env_set")?;
        decl.declare("__std_env_has")?;
        decl.declare("__std_env_remove")?;
        decl.declare("__std_env_entries")?;
        decl.declare("__std_env_load")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export(
            "__std_env_get",
            rquickjs::Function::new(ctx.clone(), env_get),
        )?;
        exports.export(
            "__std_env_set",
            rquickjs::Function::new(ctx.clone(), env_set),
        )?;
        exports.export(
            "__std_env_has",
            rquickjs::Function::new(ctx.clone(), env_has),
        )?;
        exports.export(
            "__std_env_remove",
            rquickjs::Function::new(ctx.clone(), env_remove),
        )?;
        exports.export(
            "__std_env_entries",
            rquickjs::Function::new(ctx.clone(), env_entries),
        )?;
        exports.export(
            "__std_env_load",
            rquickjs::Function::new(ctx.clone(), env_load),
        )?;
        Ok(())
    }
}

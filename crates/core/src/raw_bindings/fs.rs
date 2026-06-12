use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    Ctx, Object, Result,
};

fn fs_stat_sync<'js>(ctx: Ctx<'js>, path: String) -> Result<Object<'js>> {
    let meta = std::fs::metadata(&path).map_err(|_| rquickjs::Error::Exception)?;
    let obj = Object::new(ctx.clone())?;
    obj.set("size", meta.len() as f64)?;
    obj.set("isFile", meta.is_file())?;
    obj.set("isDirectory", meta.is_dir())?;
    obj.set("isSymlink", meta.is_symlink())?;
    if let Ok(mtime) = meta.modified() {
        let ms = mtime.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as f64;
        obj.set("mtime", ms)?;
    }
    if let Ok(ctime) = meta.created() {
        let ms = ctime.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as f64;
        obj.set("ctime", ms)?;
    }
    Ok(obj)
}

fn fs_read_file_sync(path: String) -> Result<Vec<u8>> {
    std::fs::read(&path).map_err(|_| rquickjs::Error::Exception)
}

fn fs_read_text_file_sync(path: String) -> Result<String> {
    std::fs::read_to_string(&path).map_err(|_| rquickjs::Error::Exception)
}

fn fs_write_file_sync(path: String, data: String) -> Result<()> {
    std::fs::write(&path, data).map_err(|_| rquickjs::Error::Exception)
}

fn fs_write_file_sync_buf(path: String, data: Vec<u8>) -> Result<()> {
    std::fs::write(&path, data).map_err(|_| rquickjs::Error::Exception)
}

fn fs_exists_sync(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

fn fs_mkdir_sync(path: String, recursive: Option<bool>) -> Result<()> {
    let mut b = std::fs::DirBuilder::new();
    b.recursive(recursive.unwrap_or(false));
    b.create(&path).map_err(|_| rquickjs::Error::Exception)
}

fn fs_readdir_sync(path: String) -> Result<Vec<String>> {
    let entries = std::fs::read_dir(&path).map_err(|_| rquickjs::Error::Exception)?;
    let mut names = Vec::new();
    for entry in entries {
        let name = entry.map_err(|_| rquickjs::Error::Exception)?.file_name().to_string_lossy().to_string();
        names.push(name);
    }
    // Sort for deterministic output
    names.sort();
    Ok(names)
}

fn fs_remove_sync(path: String) -> Result<()> {
    let p = std::path::Path::new(&path);
    if p.is_dir() {
        std::fs::remove_dir_all(p).map_err(|_| rquickjs::Error::Exception)
    } else {
        std::fs::remove_file(p).map_err(|_| rquickjs::Error::Exception)
    }
}

fn fs_rename_sync(old: String, new: String) -> Result<()> {
    std::fs::rename(&old, &new).map_err(|_| rquickjs::Error::Exception)
}

fn fs_copy_file_sync(src: String, dest: String) -> Result<()> {
    std::fs::copy(&src, &dest).map_err(|_| rquickjs::Error::Exception)?;
    Ok(())
}

fn fs_real_path_sync(path: String) -> Result<String> {
    let p = std::path::Path::new(&path).canonicalize().map_err(|_| rquickjs::Error::Exception)?;
    Ok(p.to_string_lossy().to_string())
}

/// Native module exposing the low-level fs operations used by the `std:fs` JS shim.
pub struct FsRawModule;

impl ModuleDef for FsRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__fs_readFileSync")?;
        decl.declare("__fs_readTextFileSync")?;
        decl.declare("__fs_writeFileSync")?;
        decl.declare("__fs_writeFileSyncBuf")?;
        decl.declare("__fs_existsSync")?;
        decl.declare("__fs_mkdirSync")?;
        decl.declare("__fs_readdirSync")?;
        decl.declare("__fs_statSync")?;
        decl.declare("__fs_removeSync")?;
        decl.declare("__fs_renameSync")?;
        decl.declare("__fs_copyFileSync")?;
        decl.declare("__fs_realPathSync")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__fs_readFileSync", rquickjs::Function::new(ctx.clone(), fs_read_file_sync))?;
        exports.export("__fs_readTextFileSync", rquickjs::Function::new(ctx.clone(), fs_read_text_file_sync))?;
        exports.export("__fs_writeFileSync", rquickjs::Function::new(ctx.clone(), fs_write_file_sync))?;
        exports.export("__fs_writeFileSyncBuf", rquickjs::Function::new(ctx.clone(), fs_write_file_sync_buf))?;
        exports.export("__fs_existsSync", rquickjs::Function::new(ctx.clone(), fs_exists_sync))?;
        exports.export("__fs_mkdirSync", rquickjs::Function::new(ctx.clone(), fs_mkdir_sync))?;
        exports.export("__fs_readdirSync", rquickjs::Function::new(ctx.clone(), fs_readdir_sync))?;
        exports.export("__fs_statSync", rquickjs::Function::new(ctx.clone(), fs_stat_sync))?;
        exports.export("__fs_removeSync", rquickjs::Function::new(ctx.clone(), fs_remove_sync))?;
        exports.export("__fs_renameSync", rquickjs::Function::new(ctx.clone(), fs_rename_sync))?;
        exports.export("__fs_copyFileSync", rquickjs::Function::new(ctx.clone(), fs_copy_file_sync))?;
        exports.export("__fs_realPathSync", rquickjs::Function::new(ctx.clone(), fs_real_path_sync))?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `std:fs`.
/// All fs helpers are provided through the native module `std:raw:fs`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

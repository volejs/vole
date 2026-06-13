use rquickjs::{
    function::Opt,
    module::{Declarations, Exports, ModuleDef},
    Ctx, Object, Promise, Result, TypedArray,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex as TokioMutex;

// =============================================================================
// Path-based synchronous filesystem helpers
// =============================================================================

fn fs_stat_sync<'js>(ctx: Ctx<'js>, path: String) -> Result<Object<'js>> {
    let meta = std::fs::metadata(&path).map_err(|_| rquickjs::Error::Exception)?;
    stat_obj(ctx, meta)
}

fn stat_obj<'js>(ctx: Ctx<'js>, meta: std::fs::Metadata) -> Result<Object<'js>> {
    let obj = Object::new(ctx)?;
    obj.set("size", meta.len() as f64)?;
    obj.set("isFile", meta.is_file())?;
    obj.set("isDirectory", meta.is_dir())?;
    obj.set("isSymlink", meta.is_symlink())?;
    if let Ok(mtime) = meta.modified() {
        let ms = mtime
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64;
        obj.set("mtime", ms)?;
    }
    if let Ok(ctime) = meta.created() {
        let ms = ctime
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64;
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
        let name = entry
            .map_err(|_| rquickjs::Error::Exception)?
            .file_name()
            .to_string_lossy()
            .to_string();
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
    let p = std::path::Path::new(&path)
        .canonicalize()
        .map_err(|_| rquickjs::Error::Exception)?;
    Ok(p.to_string_lossy().to_string())
}

// =============================================================================
// Path-based asynchronous filesystem helpers (real tokio::fs implementations)
// =============================================================================

fn fs_stat<'js>(ctx: Ctx<'js>, path: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match tokio::fs::metadata(&path).await {
            Ok(meta) => match stat_obj(ctx_clone, meta) {
                Ok(obj) => {
                    let _ = resolve.call::<_, ()>((obj,));
                }
                Err(_) => {
                    let _ = reject.call::<_, ()>(("stat failed",));
                }
            },
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_read_file<'js>(ctx: Ctx<'js>, path: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match tokio::fs::read(&path).await {
            Ok(buf) => match TypedArray::new_copy(ctx_clone, buf) {
                Ok(ta) => {
                    let _ = resolve.call::<_, ()>((ta,));
                }
                Err(_) => {
                    let _ = reject.call::<_, ()>(("read failed",));
                }
            },
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_read_text_file<'js>(ctx: Ctx<'js>, path: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match tokio::fs::read_to_string(&path).await {
            Ok(text) => {
                let _ = resolve.call::<_, ()>((text,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_write_file<'js>(ctx: Ctx<'js>, path: String, data: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match tokio::fs::write(&path, data).await {
            Ok(()) => {
                let _ = resolve.call::<_, ()>(((),));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_write_file_buf<'js>(
    ctx: Ctx<'js>,
    path: String,
    data: TypedArray<'js, u8>,
) -> Result<Promise<'js>> {
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match tokio::fs::write(&path, bytes).await {
            Ok(()) => {
                let _ = resolve.call::<_, ()>(((),));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_exists<'js>(ctx: Ctx<'js>, path: String) -> Result<Promise<'js>> {
    let (promise, resolve, _reject) = ctx.promise()?;
    ctx.spawn(async move {
        let exists = std::path::Path::new(&path).exists();
        let _ = resolve.call::<_, ()>((exists,));
    });
    Ok(promise)
}

fn fs_mkdir<'js>(ctx: Ctx<'js>, path: String, recursive: Opt<bool>) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    let recursive = recursive.0.unwrap_or(false);
    ctx.spawn(async move {
        match tokio::fs::DirBuilder::new()
            .recursive(recursive)
            .create(&path)
            .await
        {
            Ok(()) => {
                let _ = resolve.call::<_, ()>(((),));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_readdir<'js>(ctx: Ctx<'js>, path: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match tokio::fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut names = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    names.push(entry.file_name().to_string_lossy().to_string());
                }
                names.sort();
                let _ = resolve.call::<_, ()>((names,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_remove<'js>(ctx: Ctx<'js>, path: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        let p = std::path::Path::new(&path);
        let res = if p.is_dir() {
            tokio::fs::remove_dir_all(p).await
        } else {
            tokio::fs::remove_file(p).await
        };
        match res {
            Ok(()) => {
                let _ = resolve.call::<_, ()>(((),));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_rename<'js>(ctx: Ctx<'js>, old: String, new: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match tokio::fs::rename(&old, &new).await {
            Ok(()) => {
                let _ = resolve.call::<_, ()>(((),));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_copy_file<'js>(ctx: Ctx<'js>, src: String, dest: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match tokio::fs::copy(&src, &dest).await {
            Ok(_bytes) => {
                let _ = resolve.call::<_, ()>(((),));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_real_path<'js>(ctx: Ctx<'js>, path: String) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match tokio::fs::canonicalize(&path).await {
            Ok(p) => {
                let _ = resolve.call::<_, ()>((p.to_string_lossy().to_string(),));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

// =============================================================================
// File-handle helpers (merged from the old std:file module)
// =============================================================================

static FILES: std::sync::OnceLock<Mutex<HashMap<u64, Arc<TokioMutex<tokio::fs::File>>>>> =
    std::sync::OnceLock::new();
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn files() -> &'static Mutex<HashMap<u64, Arc<TokioMutex<tokio::fs::File>>>> {
    FILES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn fs_open<'js>(ctx: Ctx<'js>, path: String, options: Opt<Object<'js>>) -> Result<Promise<'js>> {
    let mut opts = tokio::fs::OpenOptions::new();
    opts.read(true);

    if let Some(obj) = options.0 {
        if let Ok(v) = obj.get::<&str, bool>("write") {
            if v {
                opts.write(true);
            }
        }
        if let Ok(v) = obj.get::<&str, bool>("append") {
            if v {
                opts.append(true);
            }
        }
        if let Ok(v) = obj.get::<&str, bool>("create") {
            if v {
                opts.create(true);
            }
        }
        if let Ok(v) = obj.get::<&str, bool>("truncate") {
            if v {
                opts.truncate(true);
            }
        }
    }

    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        match opts.open(&path).await {
            Ok(file) => {
                let id = next_id();
                files()
                    .lock()
                    .unwrap()
                    .insert(id, Arc::new(TokioMutex::new(file)));
                let obj = Object::new(ctx_clone).unwrap();
                let _ = obj.set("_id", id);
                let _ = resolve.call::<_, ()>((obj,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_fread<'js>(ctx: Ctx<'js>, id: u64, size: Opt<u32>) -> Result<Promise<'js>> {
    let file = files()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();
    ctx.spawn(async move {
        let mut buf = vec![0u8; size.0.unwrap_or(4096) as usize];
        match file.lock().await.read(&mut buf).await {
            Ok(n) => {
                buf.truncate(n);
                match TypedArray::new_copy(ctx_clone, buf) {
                    Ok(ta) => {
                        let _ = resolve.call::<_, ()>((ta,));
                    }
                    Err(_) => {
                        let _ = reject.call::<_, ()>(("read failed",));
                    }
                }
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_fwrite<'js>(ctx: Ctx<'js>, id: u64, data: TypedArray<'js, u8>) -> Result<Promise<'js>> {
    let file = files()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match file.lock().await.write_all(&bytes).await {
            Ok(()) => {
                let _ = resolve.call::<_, ()>((bytes.len() as f64,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_fseek<'js>(ctx: Ctx<'js>, id: u64, offset: i64, whence: String) -> Result<Promise<'js>> {
    let file = files()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;
    let from = match whence.as_str() {
        "start" => std::io::SeekFrom::Start(offset as u64),
        "current" => std::io::SeekFrom::Current(offset),
        "end" => std::io::SeekFrom::End(offset),
        _ => std::io::SeekFrom::Start(offset as u64),
    };
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        match file.lock().await.seek(from).await {
            Ok(pos) => {
                let _ = resolve.call::<_, ()>((pos as f64,));
            }
            Err(e) => {
                let _ = reject.call::<_, ()>((e.to_string(),));
            }
        }
    });
    Ok(promise)
}

fn fs_fclose<'js>(ctx: Ctx<'js>, id: u64) -> Result<Promise<'js>> {
    let _ = files().lock().unwrap().remove(&id);
    let (promise, resolve, _reject) = ctx.promise()?;
    let _ = resolve.call::<_, ()>(((),));
    Ok(promise)
}

// =============================================================================
// Native module definition
// =============================================================================

/// Native module exposing all low-level fs operations used by the `std:fs` JS shim.
pub struct FsRawModule;

impl ModuleDef for FsRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        // Path-based sync
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

        // Path-based async
        decl.declare("__fs_readFile")?;
        decl.declare("__fs_readTextFile")?;
        decl.declare("__fs_writeFile")?;
        decl.declare("__fs_writeFileBuf")?;
        decl.declare("__fs_exists")?;
        decl.declare("__fs_mkdir")?;
        decl.declare("__fs_readdir")?;
        decl.declare("__fs_stat")?;
        decl.declare("__fs_remove")?;
        decl.declare("__fs_rename")?;
        decl.declare("__fs_copyFile")?;
        decl.declare("__fs_realPath")?;

        // File-handle async
        decl.declare("__fs_open")?;
        decl.declare("__fs_fread")?;
        decl.declare("__fs_fwrite")?;
        decl.declare("__fs_fseek")?;
        decl.declare("__fs_fclose")?;

        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        // Path-based sync
        exports.export(
            "__fs_readFileSync",
            rquickjs::Function::new(ctx.clone(), fs_read_file_sync),
        )?;
        exports.export(
            "__fs_readTextFileSync",
            rquickjs::Function::new(ctx.clone(), fs_read_text_file_sync),
        )?;
        exports.export(
            "__fs_writeFileSync",
            rquickjs::Function::new(ctx.clone(), fs_write_file_sync),
        )?;
        exports.export(
            "__fs_writeFileSyncBuf",
            rquickjs::Function::new(ctx.clone(), fs_write_file_sync_buf),
        )?;
        exports.export(
            "__fs_existsSync",
            rquickjs::Function::new(ctx.clone(), fs_exists_sync),
        )?;
        exports.export(
            "__fs_mkdirSync",
            rquickjs::Function::new(ctx.clone(), fs_mkdir_sync),
        )?;
        exports.export(
            "__fs_readdirSync",
            rquickjs::Function::new(ctx.clone(), fs_readdir_sync),
        )?;
        exports.export(
            "__fs_statSync",
            rquickjs::Function::new(ctx.clone(), fs_stat_sync),
        )?;
        exports.export(
            "__fs_removeSync",
            rquickjs::Function::new(ctx.clone(), fs_remove_sync),
        )?;
        exports.export(
            "__fs_renameSync",
            rquickjs::Function::new(ctx.clone(), fs_rename_sync),
        )?;
        exports.export(
            "__fs_copyFileSync",
            rquickjs::Function::new(ctx.clone(), fs_copy_file_sync),
        )?;
        exports.export(
            "__fs_realPathSync",
            rquickjs::Function::new(ctx.clone(), fs_real_path_sync),
        )?;

        // Path-based async
        exports.export(
            "__fs_readFile",
            rquickjs::Function::new(ctx.clone(), fs_read_file),
        )?;
        exports.export(
            "__fs_readTextFile",
            rquickjs::Function::new(ctx.clone(), fs_read_text_file),
        )?;
        exports.export(
            "__fs_writeFile",
            rquickjs::Function::new(ctx.clone(), fs_write_file),
        )?;
        exports.export(
            "__fs_writeFileBuf",
            rquickjs::Function::new(ctx.clone(), fs_write_file_buf),
        )?;
        exports.export(
            "__fs_exists",
            rquickjs::Function::new(ctx.clone(), fs_exists),
        )?;
        exports.export("__fs_mkdir", rquickjs::Function::new(ctx.clone(), fs_mkdir))?;
        exports.export(
            "__fs_readdir",
            rquickjs::Function::new(ctx.clone(), fs_readdir),
        )?;
        exports.export("__fs_stat", rquickjs::Function::new(ctx.clone(), fs_stat))?;
        exports.export(
            "__fs_remove",
            rquickjs::Function::new(ctx.clone(), fs_remove),
        )?;
        exports.export(
            "__fs_rename",
            rquickjs::Function::new(ctx.clone(), fs_rename),
        )?;
        exports.export(
            "__fs_copyFile",
            rquickjs::Function::new(ctx.clone(), fs_copy_file),
        )?;
        exports.export(
            "__fs_realPath",
            rquickjs::Function::new(ctx.clone(), fs_real_path),
        )?;

        // File-handle async
        exports.export("__fs_open", rquickjs::Function::new(ctx.clone(), fs_open))?;
        exports.export("__fs_fread", rquickjs::Function::new(ctx.clone(), fs_fread))?;
        exports.export(
            "__fs_fwrite",
            rquickjs::Function::new(ctx.clone(), fs_fwrite),
        )?;
        exports.export("__fs_fseek", rquickjs::Function::new(ctx.clone(), fs_fseek))?;
        exports.export(
            "__fs_fclose",
            rquickjs::Function::new(ctx.clone(), fs_fclose),
        )?;

        Ok(())
    }
}

/// Legacy global init is now a no-op for `std:fs`.
/// All fs helpers are provided through the native module `std:raw:fs`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

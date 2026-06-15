use rquickjs::{
    Class, Constructor, Ctx, Function, IntoJs, Object, Promise, Result, TypedArray, Value,
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
    module::{Declarations, Exports, ModuleDef},
    prelude::{Async, Func},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex as TokioMutex;

// =============================================================================
// Path-based synchronous filesystem helpers
// =============================================================================

fn fs_stat_sync<'js>(ctx: Ctx<'js>, path: String) -> Result<Value<'js>> {
    let meta = std::fs::metadata(&path).map_err(|_| rquickjs::Error::Exception)?;
    stat_obj(ctx, meta)
}

fn stat_obj<'js>(ctx: Ctx<'js>, meta: std::fs::Metadata) -> Result<Value<'js>> {
    Stats::from(meta).into_js(&ctx)
}

// ---- native Stats class -----------------------------------------------------

#[derive(Clone, Debug)]
pub struct Stats {
    size: u64,
    is_file: bool,
    is_directory: bool,
    is_symlink: bool,
    mtime_ms: Option<u64>,
    ctime_ms: Option<u64>,
}

impl From<std::fs::Metadata> for Stats {
    fn from(meta: std::fs::Metadata) -> Self {
        let mtime_ms = meta.modified().ok().map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });
        let ctime_ms = meta.created().ok().map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });
        Self {
            size: meta.len(),
            is_file: meta.is_file(),
            is_directory: meta.is_dir(),
            is_symlink: meta.is_symlink(),
            mtime_ms,
            ctime_ms,
        }
    }
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for Stats {
    type Changed<'to> = Stats;
}

impl<'js> Trace<'js> for Stats {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for Stats {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> JsClass<'js> for Stats {
    const NAME: &'static str = "Stats";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("size", Function::new(ctx.clone(), stats_get_size)?)?;
        proto.set("mtime", Function::new(ctx.clone(), stats_get_mtime)?)?;
        proto.set("ctime", Function::new(ctx.clone(), stats_get_ctime)?)?;
        proto.set("mtimeMs", Function::new(ctx.clone(), stats_get_mtime_ms)?)?;
        proto.set("ctimeMs", Function::new(ctx.clone(), stats_get_ctime_ms)?)?;
        proto.set("isFile", Function::new(ctx.clone(), stats_is_file)?)?;
        proto.set(
            "isDirectory",
            Function::new(ctx.clone(), stats_is_directory)?,
        )?;
        proto.set(
            "isSymbolicLink",
            Function::new(ctx.clone(), stats_is_symlink)?,
        )?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor<'js>(_ctx: Ctx<'js>) -> Result<Stats> {
            Err(rquickjs::Error::Exception)
        }
        let constr = Constructor::new_class::<Stats, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}

fn stats_get_size<'js>(this: This<Class<'js, Stats>>) -> Result<u64> {
    Ok(this.try_borrow()?.size)
}

fn stats_get_mtime<'js>(ctx: Ctx<'js>, this: This<Class<'js, Stats>>) -> Result<Value<'js>> {
    let ms = this.try_borrow()?.mtime_ms;
    match ms {
        Some(ms) => {
            let date_ctor: Function = ctx.globals().get("Date")?;
            let value: Value = date_ctor.call((ms as f64,))?;
            Ok(value)
        }
        None => Ok(Value::new_undefined(ctx)),
    }
}

fn stats_get_ctime<'js>(ctx: Ctx<'js>, this: This<Class<'js, Stats>>) -> Result<Value<'js>> {
    let ms = this.try_borrow()?.ctime_ms;
    match ms {
        Some(ms) => {
            let date_ctor: Function = ctx.globals().get("Date")?;
            let value: Value = date_ctor.call((ms as f64,))?;
            Ok(value)
        }
        None => Ok(Value::new_undefined(ctx)),
    }
}

fn stats_get_mtime_ms<'js>(this: This<Class<'js, Stats>>) -> Result<Value<'js>> {
    let ms = this.try_borrow()?.mtime_ms;
    let ctx = this.ctx().clone();
    Ok(match ms {
        Some(ms) => Value::new_number(ctx, ms as f64),
        None => Value::new_undefined(ctx),
    })
}

fn stats_get_ctime_ms<'js>(this: This<Class<'js, Stats>>) -> Result<Value<'js>> {
    let ms = this.try_borrow()?.ctime_ms;
    let ctx = this.ctx().clone();
    Ok(match ms {
        Some(ms) => Value::new_number(ctx, ms as f64),
        None => Value::new_undefined(ctx),
    })
}

fn stats_is_file<'js>(this: This<Class<'js, Stats>>) -> Result<bool> {
    Ok(this.try_borrow()?.is_file)
}

fn stats_is_directory<'js>(this: This<Class<'js, Stats>>) -> Result<bool> {
    Ok(this.try_borrow()?.is_directory)
}

fn stats_is_symlink<'js>(this: This<Class<'js, Stats>>) -> Result<bool> {
    Ok(this.try_borrow()?.is_symlink)
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

fn fs_write_file_sync_buf(path: String, data: TypedArray<u8>) -> Result<()> {
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };
    std::fs::write(&path, bytes).map_err(|_| rquickjs::Error::Exception)
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

async fn fs_stat<'js>(_ctx: Ctx<'js>, path: String) -> Result<Stats> {
    let meta = tokio::fs::metadata(&path)
        .await
        .map_err(|_| rquickjs::Error::Exception)?;
    Ok(Stats::from(meta))
}

async fn fs_read_file<'js>(ctx: Ctx<'js>, path: String) -> Result<TypedArray<'js, u8>> {
    let buf = tokio::fs::read(&path)
        .await
        .map_err(|_| rquickjs::Error::Exception)?;
    TypedArray::new_copy(ctx, buf)
}

async fn fs_read_text_file(_ctx: Ctx<'_>, path: String) -> Result<String> {
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| rquickjs::Error::Exception)
}

async fn fs_write_file(_ctx: Ctx<'_>, path: String, data: String) -> Result<()> {
    tokio::fs::write(&path, data)
        .await
        .map_err(|_| rquickjs::Error::Exception)
}

async fn fs_write_file_buf<'js>(
    _ctx: Ctx<'js>,
    path: String,
    data: TypedArray<'js, u8>,
) -> Result<()> {
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };
    tokio::fs::write(&path, bytes)
        .await
        .map_err(|_| rquickjs::Error::Exception)
}

async fn fs_exists(_ctx: Ctx<'_>, path: String) -> bool {
    std::path::Path::new(&path).exists()
}

async fn fs_mkdir(_ctx: Ctx<'_>, path: String, recursive: Opt<bool>) -> Result<()> {
    tokio::fs::DirBuilder::new()
        .recursive(recursive.0.unwrap_or(false))
        .create(&path)
        .await
        .map_err(|_| rquickjs::Error::Exception)
}

async fn fs_readdir(_ctx: Ctx<'_>, path: String) -> Result<Vec<String>> {
    let mut entries = tokio::fs::read_dir(&path)
        .await
        .map_err(|_| rquickjs::Error::Exception)?;
    let mut names = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        names.push(entry.file_name().to_string_lossy().to_string());
    }
    names.sort();
    Ok(names)
}

async fn fs_remove(_ctx: Ctx<'_>, path: String) -> Result<()> {
    let p = std::path::Path::new(&path);
    let res = if p.is_dir() {
        tokio::fs::remove_dir_all(p).await
    } else {
        tokio::fs::remove_file(p).await
    };
    res.map_err(|_| rquickjs::Error::Exception)
}

async fn fs_rename(_ctx: Ctx<'_>, old: String, new: String) -> Result<()> {
    tokio::fs::rename(&old, &new)
        .await
        .map_err(|_| rquickjs::Error::Exception)
}

async fn fs_copy_file(_ctx: Ctx<'_>, src: String, dest: String) -> Result<()> {
    tokio::fs::copy(&src, &dest)
        .await
        .map_err(|_| rquickjs::Error::Exception)?;
    Ok(())
}

async fn fs_real_path(_ctx: Ctx<'_>, path: String) -> Result<String> {
    let p = tokio::fs::canonicalize(&path)
        .await
        .map_err(|_| rquickjs::Error::Exception)?;
    Ok(p.to_string_lossy().to_string())
}

// =============================================================================
// File-handle helpers (merged from the old vole:file module)
// =============================================================================

#[derive(Clone)]
enum HandleEntry {
    File(Arc<TokioMutex<tokio::fs::File>>),
    Dir(Arc<TokioMutex<tokio::fs::ReadDir>>),
}

static FILES: std::sync::OnceLock<Mutex<HashMap<u64, HandleEntry>>> =
    std::sync::OnceLock::new();
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn files() -> &'static Mutex<HashMap<u64, HandleEntry>> {
    FILES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn get_file(id: u64) -> Result<Arc<TokioMutex<tokio::fs::File>>> {
    match files()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
    {
        Some(HandleEntry::File(handle)) => Ok(handle),
        Some(HandleEntry::Dir(_)) => Err(rquickjs::Error::Exception),
        None => Err(rquickjs::Error::Exception),
    }
}

fn get_dir(id: u64) -> Result<Arc<TokioMutex<tokio::fs::ReadDir>>> {
    match files()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
    {
        Some(HandleEntry::Dir(handle)) => Ok(handle),
        Some(HandleEntry::File(_)) => Err(rquickjs::Error::Exception),
        None => Err(rquickjs::Error::Exception),
    }
}

// ---- native FileHandle class ------------------------------------------------

#[derive(Clone, Debug)]
pub struct FileHandle {
    id: u64,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for FileHandle {
    type Changed<'to> = FileHandle;
}

impl<'js> Trace<'js> for FileHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for FileHandle {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> JsClass<'js> for FileHandle {
    const NAME: &'static str = "FileHandle";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("read", Function::new(ctx.clone(), file_handle_read)?)?;
        proto.set("readEntries", Function::new(ctx.clone(), file_handle_read_entries)?)?;
        proto.set("write", Function::new(ctx.clone(), file_handle_write)?)?;
        proto.set("seek", Function::new(ctx.clone(), file_handle_seek)?)?;
        proto.set("close", Function::new(ctx.clone(), file_handle_close)?)?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor<'js>(_ctx: Ctx<'js>, _id: Opt<u64>) -> Result<FileHandle> {
            Err(rquickjs::Error::Exception)
        }
        let constr = Constructor::new_class::<FileHandle, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}

fn file_handle_read<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, FileHandle>>,
    size: Opt<u32>,
) -> Result<Promise<'js>> {
    let id = this.try_borrow()?.id;
    let file = get_file(id)?;
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

fn file_handle_read_entries<'js>(ctx: Ctx<'js>, this: This<Class<'js, FileHandle>>) -> Result<Promise<'js>> {
    let id = this.try_borrow()?.id;
    let dir = get_dir(id)?;
    let (promise, resolve, reject) = ctx.promise()?;
    ctx.spawn(async move {
        let mut entries = Vec::new();
        let mut dir = dir.lock().await;
        loop {
            match dir.next_entry().await {
                Ok(Some(entry)) => entries.push(entry.file_name().to_string_lossy().to_string()),
                Ok(None) => {
                    let _ = resolve.call::<_, ()>((entries,));
                    break;
                }
                Err(e) => {
                    let _ = reject.call::<_, ()>((e.to_string(),));
                    break;
                }
            }
        }
    });
    Ok(promise)
}

fn file_handle_write<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, FileHandle>>,
    data: TypedArray<'js, u8>,
) -> Result<Promise<'js>> {
    let id = this.try_borrow()?.id;
    let file = get_file(id)?;
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

fn file_handle_seek<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, FileHandle>>,
    offset: i64,
    whence: Opt<String>,
) -> Result<Promise<'js>> {
    let id = this.try_borrow()?.id;
    let file = get_file(id)?;
    let whence = whence.0.unwrap_or_else(|| "start".to_string());
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

fn file_handle_close<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, FileHandle>>,
) -> Result<Promise<'js>> {
    let id = this.try_borrow()?.id;
    let _ = files().lock().unwrap().remove(&id);
    let (promise, resolve, _reject) = ctx.promise()?;
    let _ = resolve.call::<_, ()>(((),));
    Ok(promise)
}

async fn fs_open<'js>(
    _ctx: Ctx<'js>,
    path: String,
    options: Opt<Object<'js>>,
) -> Result<FileHandle> {
    let mut read = true;
    let mut write = false;
    let mut append = false;
    let mut create = false;
    let mut truncate = false;

    let mut opts = tokio::fs::OpenOptions::new();
    opts.read(true);

    if let Some(obj) = options.0 {
        if let Ok(v) = obj.get::<&str, bool>("write") {
            if v {
                write = true;
                opts.write(true);
            }
        }
        if let Ok(v) = obj.get::<&str, bool>("append") {
            if v {
                append = true;
                opts.append(true);
            }
        }
        if let Ok(v) = obj.get::<&str, bool>("create") {
            if v {
                create = true;
                opts.create(true);
            }
        }
        if let Ok(v) = obj.get::<&str, bool>("truncate") {
            if v {
                truncate = true;
                opts.truncate(true);
            }
        }
    }

    if let Ok(meta) = tokio::fs::metadata(&path).await {
        if meta.is_dir() {
            if write || append || create || truncate {
                return Err(rquickjs::Error::Exception);
            }
            let dir = tokio::fs::read_dir(&path)
                .await
                .map_err(|_| rquickjs::Error::Exception)?;
            let id = next_id();
            files()
                .lock()
                .unwrap()
                .insert(id, HandleEntry::Dir(Arc::new(TokioMutex::new(dir))));
            return Ok(FileHandle { id });
        }
        read = true;
    }

    if !read && !create {
        opts.read(true);
    }

    let file = opts
        .open(&path)
        .await
        .map_err(|_| rquickjs::Error::Exception)?;
    let id = next_id();
    files()
        .lock()
        .unwrap()
        .insert(id, HandleEntry::File(Arc::new(TokioMutex::new(file))));
    Ok(FileHandle { id })
}

// =============================================================================
// Native module definition
// =============================================================================

/// Native module exposing all low-level fs operations used by `vole:os/fs`.
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
        decl.declare("FileHandle")?;

        // Native classes
        decl.declare("Stats")?;

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
        exports.export("__fs_readFile", Func::from(Async(fs_read_file)))?;
        exports.export("__fs_readTextFile", Func::from(Async(fs_read_text_file)))?;
        exports.export("__fs_writeFile", Func::from(Async(fs_write_file)))?;
        exports.export("__fs_writeFileBuf", Func::from(Async(fs_write_file_buf)))?;
        exports.export("__fs_exists", Func::from(Async(fs_exists)))?;
        exports.export("__fs_mkdir", Func::from(Async(fs_mkdir)))?;
        exports.export("__fs_readdir", Func::from(Async(fs_readdir)))?;
        exports.export("__fs_stat", Func::from(Async(fs_stat)))?;
        exports.export("__fs_remove", Func::from(Async(fs_remove)))?;
        exports.export("__fs_rename", Func::from(Async(fs_rename)))?;
        exports.export("__fs_copyFile", Func::from(Async(fs_copy_file)))?;
        exports.export("__fs_realPath", Func::from(Async(fs_real_path)))?;

        // File-handle async
        exports.export("__fs_open", Func::from(Async(fs_open)))?;

        // Register native classes and export their constructors so
        // `instanceof fs.FileHandle` / `instanceof fs.Stats` work.
        Class::<FileHandle>::define(&ctx.globals())?;
        let file_handle_ctor: rquickjs::Function = ctx.globals().get("FileHandle")?;
        exports.export("FileHandle", file_handle_ctor)?;

        Class::<Stats>::define(&ctx.globals())?;
        let stats_ctor: rquickjs::Function = ctx.globals().get("Stats")?;
        exports.export("Stats", stats_ctor)?;

        // Convert Stats prototype functions to getters (size/mtime/ctime/*Ms).
        ctx.eval::<(), _>(
            r#"
            (function() {
                function wrapGetter(proto, name) {
                    const fn = proto[name];
                    Object.defineProperty(proto, name, {
                        get: fn,
                        enumerable: true,
                        configurable: true,
                    });
                }
                wrapGetter(Stats.prototype, "size");
                wrapGetter(Stats.prototype, "mtime");
                wrapGetter(Stats.prototype, "ctime");
                wrapGetter(Stats.prototype, "mtimeMs");
                wrapGetter(Stats.prototype, "ctimeMs");
            })();
        "#,
        )?;

        Ok(())
    }
}

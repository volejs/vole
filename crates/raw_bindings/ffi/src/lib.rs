use libffi::middle::{Arg, Cif, CodePtr, Type as FfiType};
use libloading::Library;
use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    ArrayBuffer, Ctx, Function, Result, TypedArray, Value,
};
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CString};
use std::sync::Mutex;

static LIBRARIES: Mutex<Option<HashMap<u64, Library>>> = Mutex::new(None);
static NEXT_HANDLE: Mutex<u64> = Mutex::new(1);

fn get_libraries() -> std::sync::MutexGuard<'static, Option<HashMap<u64, Library>>> {
    let mut guard = LIBRARIES.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    guard
}

fn next_handle() -> u64 {
    let mut next = NEXT_HANDLE.lock().unwrap();
    let handle = *next;
    *next += 1;
    handle
}

fn parse_type(type_name: &str) -> Result<FfiType> {
    match type_name {
        "void" => Ok(FfiType::void()),
        "bool" => Ok(FfiType::u8()),
        "i8" => Ok(FfiType::i8()),
        "i16" => Ok(FfiType::i16()),
        "i32" => Ok(FfiType::i32()),
        "i64" => Ok(FfiType::i64()),
        "u8" => Ok(FfiType::u8()),
        "u16" => Ok(FfiType::u16()),
        "u32" => Ok(FfiType::u32()),
        "u64" => Ok(FfiType::u64()),
        "f32" => Ok(FfiType::f32()),
        "f64" => Ok(FfiType::f64()),
        "pointer" | "string" | "cstring" => Ok(FfiType::pointer()),
        "usize" => Ok(FfiType::usize()),
        "isize" => Ok(FfiType::isize()),
        _ => Err(rquickjs::Error::Exception),
    }
}

fn store_bytes<T: Copy + 'static>(value: T) -> (Box<Vec<u8>>, Arg) {
    let size = std::mem::size_of::<T>();
    let mut storage = vec![0u8; size];
    unsafe {
        (storage.as_mut_ptr() as *mut T).write(value);
    }
    let arg = Arg::new(&storage[0]);
    (Box::new(storage), arg)
}

/// Convert a JS value to an FFI argument, keeping backing storage alive.
/// Returns the backing storage (must outlive the call) and the libffi Arg.
fn value_to_arg<'js>(value: Value<'js>, type_name: &str) -> Result<(Box<dyn Send>, Arg)> {
    match type_name {
        "void" => {
            let (storage, arg) = store_bytes(0i32);
            Ok((storage, arg))
        }
        "bool" => {
            let b = value.as_bool().unwrap_or(false);
            let (storage, arg) = store_bytes::<u8>(if b { 1 } else { 0 });
            Ok((storage, arg))
        }
        "i8" => {
            let v = value.as_int().unwrap_or(0) as i8;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "i16" => {
            let v = value.as_int().unwrap_or(0) as i16;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "i32" => {
            let v = value.as_int().unwrap_or(0) as i32;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "i64" => {
            let v = value
                .as_float()
                .unwrap_or(value.as_int().unwrap_or(0) as f64) as i64;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "u8" => {
            let v = value.as_int().unwrap_or(0) as u8;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "u16" => {
            let v = value.as_int().unwrap_or(0) as u16;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "u32" => {
            let v = value.as_int().unwrap_or(0) as u32;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "u64" => {
            let v = value
                .as_float()
                .unwrap_or(value.as_int().unwrap_or(0) as f64) as u64;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "f32" => {
            let v = value.as_float().unwrap_or(0.0) as f32;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "f64" => {
            let v = value.as_float().unwrap_or(0.0);
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "usize" => {
            let v = value.as_int().unwrap_or(0) as usize;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "isize" => {
            let v = value.as_int().unwrap_or(0) as isize;
            let (storage, arg) = store_bytes(v);
            Ok((storage, arg))
        }
        "pointer" => {
            let addr: usize = if value.is_null() || value.is_undefined() {
                0
            } else if let Ok(bytes) = value_bytes(value.clone()) {
                if bytes.is_empty() {
                    0
                } else {
                    let leaked: *mut u8 = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;
                    leaked as usize
                }
            } else if let Some(n) = value.as_int() {
                n as usize
            } else if let Some(n) = value.as_float() {
                n as usize
            } else {
                0
            };
            // For non-null array pointers we intentionally leak; callers must
            // keep the JS buffer alive for the duration of the call.
            let (storage, arg) = store_bytes(addr);
            Ok((storage, arg))
        }
        "string" | "cstring" => {
            let s = if let Some(s) = value.as_string() {
                s.to_string()?
            } else {
                String::new()
            };
            let cstring = CString::new(s).map_err(|_| rquickjs::Error::Exception)?;
            let ptr = cstring.as_ptr() as usize;
            // Leak the CString; the JS API expects the caller to manage lifetime.
            std::mem::forget(cstring);
            let (storage, arg) = store_bytes(ptr);
            Ok((storage, arg))
        }
        _ => Err(rquickjs::Error::Exception),
    }
}

fn value_bytes<'js>(value: Value<'js>) -> Result<Vec<u8>> {
    if let Ok(ab) = value.get::<ArrayBuffer<'js>>() {
        Ok(<ArrayBuffer<'js> as AsRef<[u8]>>::as_ref(&ab).to_vec())
    } else if let Ok(ta) = value.get::<TypedArray<'js, u8>>() {
        Ok(<TypedArray<'js, u8> as AsRef<[u8]>>::as_ref(&ta).to_vec())
    } else {
        Err(rquickjs::Error::Exception)
    }
}

unsafe fn result_to_value<'js>(
    ctx: &Ctx<'js>,
    result: *mut c_void,
    type_name: &str,
) -> Result<Value<'js>> {
    // libffi returns small values (<= pointer size) directly in the `result`
    // word; larger values are returned via a hidden pointer. We treat all
    // scalar/pointer returns as the value itself.
    // libffi returns scalar values directly in the `result` word/pointer.
    // Interpret the bits of `result` itself rather than dereferencing it.
    let bits = result as usize;
    match type_name {
        "void" => Ok(Value::new_undefined(ctx.clone())),
        "bool" => Ok(Value::new_bool(ctx.clone(), (bits as u8) != 0)),
        "i8" => Ok(Value::new_int(ctx.clone(), bits as i8 as i32)),
        "i16" => Ok(Value::new_int(ctx.clone(), bits as i16 as i32)),
        "i32" => Ok(Value::new_int(ctx.clone(), bits as i32)),
        "i64" => Ok(Value::new_float(ctx.clone(), bits as i64 as f64)),
        "u8" => Ok(Value::new_int(ctx.clone(), bits as u8 as i32)),
        "u16" => Ok(Value::new_int(ctx.clone(), bits as u16 as i32)),
        "u32" => Ok(Value::new_int(ctx.clone(), bits as u32 as i32)),
        "u64" => Ok(Value::new_float(ctx.clone(), bits as u64 as f64)),
        "f32" => Ok(Value::new_float(ctx.clone(), f32::from_bits(bits as u32) as f64)),
        "f64" => Ok(Value::new_float(ctx.clone(), f64::from_bits(bits as u64))),
        "usize" => Ok(Value::new_float(ctx.clone(), bits as f64)),
        "isize" => Ok(Value::new_float(ctx.clone(), bits as isize as f64)),
        "pointer" => {
            let ptr = result as usize;
            if ptr == 0 {
                Ok(Value::new_null(ctx.clone()))
            } else {
                Ok(Value::new_float(ctx.clone(), ptr as f64))
            }
        }
        "string" | "cstring" => {
            let ptr = result as *const c_char;
            if ptr.is_null() {
                Ok(Value::new_null(ctx.clone()))
            } else {
                let cstr = std::ffi::CStr::from_ptr(ptr);
                let s = cstr.to_str().unwrap_or("").to_string();
                let js_str = rquickjs::String::from_str(ctx.clone(), &s)?;
                Ok(Value::from_string(js_str))
            }
        }
        _ => Err(rquickjs::Error::Exception),
    }
}

fn ffi_open<'js>(ctx: Ctx<'js>, path: String) -> Result<Value<'js>> {
    let lib = unsafe { Library::new(path).map_err(|_| rquickjs::Error::Exception)? };
    let handle = next_handle();
    get_libraries().as_mut().unwrap().insert(handle, lib);
    Ok(Value::new_float(ctx, handle as f64))
}

fn ffi_close(_ctx: Ctx, handle: f64) -> Result<()> {
    let handle = handle as u64;
    get_libraries().as_mut().unwrap().remove(&handle);
    Ok(())
}

fn ffi_call<'js>(
    ctx: Ctx<'js>,
    handle: f64,
    symbol: String,
    arg_types: Vec<String>,
    ret_type: String,
    args: Vec<Value<'js>>,
) -> Result<Value<'js>> {
    let handle = handle as u64;
    let libraries = get_libraries();
    let lib = libraries
        .as_ref()
        .unwrap()
        .get(&handle)
        .ok_or(rquickjs::Error::Exception)?;

    let cif_arg_types: Vec<FfiType> = arg_types
        .iter()
        .map(|t| parse_type(t))
        .collect::<Result<Vec<_>>>()?;
    let cif_ret_type = parse_type(&ret_type)?;

    let cif = Cif::new(cif_arg_types.into_iter(), cif_ret_type);

    let mut ffi_args: Vec<Arg> = Vec::with_capacity(args.len());
    let mut _storages: Vec<Box<dyn Send>> = Vec::with_capacity(args.len());
    for (i, arg) in args.into_iter().enumerate() {
        let type_name = arg_types.get(i).ok_or(rquickjs::Error::Exception)?;
        let (storage, ffi_arg) = value_to_arg(arg, type_name)?;
        _storages.push(storage);
        ffi_args.push(ffi_arg);
    }

    let code_ptr: *mut c_void = unsafe {
        let sym: libloading::Symbol<'_, *mut c_void> = lib
            .get(symbol.as_bytes())
            .map_err(|_| rquickjs::Error::Exception)?;
        *sym
    };

    let result: *mut c_void = unsafe {
        if ffi_args.is_empty() {
            cif.call(CodePtr(code_ptr), &[])
        } else {
            cif.call(CodePtr(code_ptr), &ffi_args)
        }
    };

    unsafe { result_to_value(&ctx, result, &ret_type) }
}

/// Native module exposing the low-level FFI operations used by the `vole:ffi` JS shim.
pub struct FfiRawModule;

impl ModuleDef for FfiRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__ffi_open")?;
        decl.declare("__ffi_close")?;
        decl.declare("__ffi_call")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__ffi_open", Function::new(ctx.clone(), ffi_open))?;
        exports.export("__ffi_close", Function::new(ctx.clone(), ffi_close))?;
        exports.export("__ffi_call", Function::new(ctx.clone(), ffi_call))?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `vole:ffi`.
/// All FFI helpers are provided through the native module `vole:raw:ffi`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

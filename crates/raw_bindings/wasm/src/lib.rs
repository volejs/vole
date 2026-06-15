use rquickjs::module::{Declarations, Exports, ModuleDef};
use rquickjs::{Array, Ctx, Function, Object, Result, TypedArray, Value, qjs};
use std::collections::HashMap;
use std::sync::Mutex;
use wasmi::{Engine, Func, FuncType, Instance, Linker, Module, Store, Val, ValType};

/// Wrapper to send a raw JSContext pointer across thread boundaries.
/// All actual access happens on the QuickJS owner thread, so this is safe in
/// practice for our single-threaded-per-VM usage model.
#[derive(Clone, Copy)]
struct SendCtxPtr(*mut qjs::JSContext);
unsafe impl Send for SendCtxPtr {}
unsafe impl Sync for SendCtxPtr {}

static ENGINE: std::sync::OnceLock<Engine> = std::sync::OnceLock::new();

fn engine() -> &'static Engine {
    ENGINE.get_or_init(Engine::default)
}

struct WasmStore {
    store: Store<()>,
}

static MODULES: std::sync::OnceLock<Mutex<HashMap<u64, Module>>> = std::sync::OnceLock::new();
static INSTANCES: std::sync::OnceLock<Mutex<HashMap<u64, (Instance, WasmStore)>>> =
    std::sync::OnceLock::new();
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn modules() -> &'static Mutex<HashMap<u64, Module>> {
    MODULES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn instances() -> &'static Mutex<HashMap<u64, (Instance, WasmStore)>> {
    INSTANCES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

fn wasm_compile(_ctx: Ctx, data: TypedArray<u8>) -> Result<u64> {
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }
    } else {
        return Err(rquickjs::Error::Exception);
    };

    let module = Module::new(engine(), bytes).map_err(|_| rquickjs::Error::Exception)?;
    let id = next_id();
    modules().lock().unwrap().insert(id, module);
    Ok(id)
}

fn wasm_instantiate<'js>(
    ctx: Ctx<'js>,
    module_id: u64,
    imports: rquickjs::function::Opt<Object<'js>>,
) -> Result<u64> {
    let module = modules()
        .lock()
        .unwrap()
        .get(&module_id)
        .cloned()
        .ok_or(rquickjs::Error::Exception)?;

    let mut store = Store::new(engine(), ());
    let mut linker: Linker<()> = Linker::new(engine());

    let instance_id = next_id();

    // Store import object on the global object so closures can access it
    let import_object = imports
        .0
        .unwrap_or_else(|| Object::new(ctx.clone()).unwrap());
    let import_key = format!("__wasm_imports_{}", instance_id);
    ctx.globals().set(&import_key, import_object)?;

    // Capture the raw JSContext pointer for use in closures.
    // SAFETY: Ctx is a (NonNull<JSContext>, ZST lifetime marker) tuple. The
    // clone inside the closure balances the Drop, so the underlying JSContext
    // ref-count is not artificially extended past this function.
    let raw_ctx: *mut qjs::JSContext = unsafe { std::mem::transmute_copy(&ctx) };
    let ctx_ptr = SendCtxPtr(raw_ctx);

    // Collect function imports
    let mut func_imports: Vec<(String, String, FuncType)> = Vec::new();
    for import in module.imports() {
        if let wasmi::ExternType::Func(func_type) = import.ty() {
            func_imports.push((
                import.module().to_string(),
                import.name().to_string(),
                func_type.clone(),
            ));
        }
    }

    // Register each function import with the linker
    for (mod_name, func_name, func_type) in func_imports {
        let import_key = format!("__wasm_imports_{}", instance_id);
        let mod_name_clone = mod_name.clone();
        let func_name_clone = func_name.clone();

        let _param_types = func_type.params();
        let result_types: Vec<ValType> = func_type.results().to_vec();

        let func = Func::new(
            &mut store,
            func_type.clone(),
            move |_caller: wasmi::Caller<'_, ()>, inputs: &[Val], outputs: &mut [Val]| {
                // Capture the whole SendCtxPtr so the closure is Send/Sync.
                let ctx_ptr = ctx_ptr;
                // Create a temporary Ctx handle with a balanced ref-count.
                unsafe { qjs::JS_DupContext(ctx_ptr.0) };
                let ctx: Ctx<'_> = unsafe { std::mem::transmute_copy(&ctx_ptr.0) };

                let imports_obj: Object = ctx
                    .globals()
                    .get(&import_key)
                    .map_err(|_| wasmi::Error::new("failed to get wasm imports"))?;
                let module_obj: Object = imports_obj
                    .get(&mod_name_clone)
                    .map_err(|_| wasmi::Error::new("wasm import module not found"))?;
                let js_func: Function = module_obj
                    .get(&func_name_clone)
                    .map_err(|_| wasmi::Error::new("wasm import function not found"))?;

                // Build JS args
                let mut js_args = rquickjs::function::Args::new(ctx.clone(), inputs.len());
                for arg in inputs.iter() {
                    let val = match arg {
                        Val::I32(v) => Value::new_int(ctx.clone(), *v),
                        Val::I64(v) => Value::new_float(ctx.clone(), *v as f64),
                        Val::F32(v) => Value::new_float(ctx.clone(), f32::from(*v) as f64),
                        Val::F64(v) => Value::new_float(ctx.clone(), f64::from(*v)),
                        _ => return Err(wasmi::Error::new("unsupported wasm arg type")),
                    };
                    js_args
                        .push_arg(val)
                        .map_err(|_| wasmi::Error::new("failed to push arg"))?;
                }

                // Call the JS function
                let result = js_func
                    .call_arg::<Value>(js_args)
                    .map_err(|e| wasmi::Error::new(format!("JS function call failed: {:?}", e)))?;

                // Convert result back to wasm values
                if result_types.is_empty() {
                    // nothing to do
                } else if result_types.len() == 1 {
                    outputs[0] =
                        jsval_to_wasm_val(&ctx, &result_types[0], &result).map_err(|e| {
                            wasmi::Error::new(format!("result conversion failed: {:?}", e))
                        })?;
                } else {
                    let arr: Array = result
                        .get()
                        .map_err(|_| wasmi::Error::new("expected array result for multi-value"))?;
                    if arr.len() as usize != result_types.len() {
                        return Err(wasmi::Error::new("result array length mismatch"));
                    }
                    for (i, ty) in result_types.iter().enumerate() {
                        let elem: Value = arr
                            .get(i)
                            .map_err(|_| wasmi::Error::new("failed to get result element"))?;
                        outputs[i] = jsval_to_wasm_val(&ctx, ty, &elem).map_err(|e| {
                            wasmi::Error::new(format!("result conversion failed: {:?}", e))
                        })?;
                    }
                }

                Ok(())
            },
        );

        linker
            .define(&mod_name, &func_name, func)
            .map_err(|_| rquickjs::Error::Exception)?;
    }

    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(|_| rquickjs::Error::Exception)?;

    instances()
        .lock()
        .unwrap()
        .insert(instance_id, (instance, WasmStore { store }));
    Ok(instance_id)
}

fn jsval_to_wasm_val(_ctx: &Ctx, ty: &ValType, val: &Value) -> Result<Val> {
    Ok(match ty {
        ValType::I32 => Val::I32(val.as_int().ok_or(rquickjs::Error::Exception)?),
        ValType::I64 => Val::I64(val.as_float().ok_or(rquickjs::Error::Exception)? as i64),
        ValType::F32 => Val::F32((val.as_float().ok_or(rquickjs::Error::Exception)? as f32).into()),
        ValType::F64 => Val::F64(val.as_float().ok_or(rquickjs::Error::Exception)?.into()),
        _ => return Err(rquickjs::Error::Exception),
    })
}

fn wasm_instance_exports<'js>(ctx: Ctx<'js>, instance_id: u64) -> Result<Object<'js>> {
    let instances_lock = instances().lock().unwrap();
    let (instance, store) = instances_lock
        .get(&instance_id)
        .ok_or(rquickjs::Error::Exception)?;

    let result = Object::new(ctx.clone())?;
    let funcs = Array::new(ctx.clone())?;
    let mems = Array::new(ctx.clone())?;

    let mut func_idx = 0u32;
    let mut mem_idx = 0u32;

    for export in instance.exports(&store.store) {
        let name = export.name();
        let export_clone = export.clone();
        if let Some(func) = export_clone.into_func() {
            let ty = func.ty(&store.store);
            let params: Vec<String> = ty.params().iter().map(|t| format!("{:?}", t)).collect();
            let results: Vec<String> = ty.results().iter().map(|t| format!("{:?}", t)).collect();
            let obj = Object::new(ctx.clone())?;
            obj.set("name", name)?;
            obj.set("kind", "function")?;
            obj.set("params", params.join(","))?;
            obj.set("results", results.join(","))?;
            funcs.set(func_idx as usize, obj)?;
            func_idx += 1;
        } else if let Some(_mem) = export.into_memory() {
            let obj = Object::new(ctx.clone())?;
            obj.set("name", name)?;
            obj.set("kind", "memory")?;
            mems.set(mem_idx as usize, obj)?;
            mem_idx += 1;
        }
    }

    result.set("functions", funcs)?;
    result.set("memories", mems)?;
    Ok(result)
}

fn wasm_call_function<'js>(
    ctx: Ctx<'js>,
    instance_id: u64,
    name: String,
    args: Array<'js>,
) -> Result<Value<'js>> {
    let mut instances_lock = instances().lock().unwrap();
    let (instance, store) = instances_lock
        .get_mut(&instance_id)
        .ok_or(rquickjs::Error::Exception)?;

    let func = instance
        .get_func(&store.store, &name)
        .ok_or(rquickjs::Error::Exception)?;

    let ty = func.ty(&store.store);
    let param_types: Vec<ValType> = ty.params().to_vec();
    let result_types: Vec<ValType> = ty.results().to_vec();

    if args.len() as usize != param_types.len() {
        return Err(rquickjs::Error::Exception);
    }

    let mut inputs: Vec<Val> = Vec::with_capacity(param_types.len());
    for (i, ty) in param_types.iter().enumerate() {
        let val: i32 = args.get(i).map_err(|_| rquickjs::Error::Exception)?;
        match ty {
            ValType::I32 => inputs.push(Val::I32(val)),
            ValType::I64 => inputs.push(Val::I64(val as i64)),
            ValType::F32 => inputs.push(Val::F32((val as f32).into())),
            ValType::F64 => inputs.push(Val::F64((val as f64).into())),
            _ => return Err(rquickjs::Error::Exception),
        }
    }

    let mut outputs: Vec<Val> = result_types.iter().map(|t| default_val(t)).collect();

    func.call(&mut store.store, &inputs, &mut outputs)
        .map_err(|_| rquickjs::Error::Exception)?;

    if outputs.is_empty() {
        Ok(Value::new_undefined(ctx))
    } else {
        match outputs[0] {
            Val::I32(v) => Ok(Value::new_int(ctx, v)),
            Val::I64(v) => Ok(Value::new_float(ctx, v as f64)),
            Val::F32(v) => Ok(Value::new_float(ctx, f32::from(v) as f64)),
            Val::F64(v) => Ok(Value::new_float(ctx, f64::from(v))),
            _ => Ok(Value::new_undefined(ctx)),
        }
    }
}

fn default_val(ty: &ValType) -> Val {
    match ty {
        ValType::I32 => Val::I32(0),
        ValType::I64 => Val::I64(0),
        ValType::F32 => Val::F32(0.0f32.into()),
        ValType::F64 => Val::F64(0.0f64.into()),
        _ => Val::I32(0),
    }
}

fn wasm_memory_read<'js>(
    ctx: Ctx<'js>,
    instance_id: u64,
    name: String,
    offset: u32,
    length: u32,
) -> Result<TypedArray<'js, u8>> {
    let mut instances_lock = instances().lock().unwrap();
    let (instance, store) = instances_lock
        .get_mut(&instance_id)
        .ok_or(rquickjs::Error::Exception)?;

    let memory = instance
        .get_memory(&store.store, &name)
        .ok_or(rquickjs::Error::Exception)?;

    let data = memory.data(&store.store);
    let start = offset as usize;
    let end = (offset + length) as usize;
    if end > data.len() {
        return Err(rquickjs::Error::Exception);
    }

    TypedArray::new_copy(ctx, &data[start..end])
}

fn wasm_memory_write<'js>(
    _ctx: Ctx<'js>,
    instance_id: u64,
    name: String,
    offset: u32,
    data: TypedArray<'js, u8>,
) -> Result<()> {
    let mut instances_lock = instances().lock().unwrap();
    let (instance, store) = instances_lock
        .get_mut(&instance_id)
        .ok_or(rquickjs::Error::Exception)?;

    let memory = instance
        .get_memory(&store.store, &name)
        .ok_or(rquickjs::Error::Exception)?;

    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }
    } else {
        return Err(rquickjs::Error::Exception);
    };

    let mem_data = memory.data_mut(&mut store.store);
    let start = offset as usize;
    let end = start + bytes.len();
    if end > mem_data.len() {
        return Err(rquickjs::Error::Exception);
    }

    mem_data[start..end].copy_from_slice(bytes);
    Ok(())
}

pub struct WasmRawModule;

impl ModuleDef for WasmRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("__wasm_compile")?;
        decl.declare("__wasm_instantiate")?;
        decl.declare("__wasm_instance_exports")?;
        decl.declare("__wasm_call_function")?;
        decl.declare("__wasm_memory_read")?;
        decl.declare("__wasm_memory_write")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> rquickjs::Result<()> {
        exports.export("__wasm_compile", Function::new(ctx.clone(), wasm_compile))?;
        exports.export(
            "__wasm_instantiate",
            Function::new(ctx.clone(), wasm_instantiate),
        )?;
        exports.export(
            "__wasm_instance_exports",
            Function::new(ctx.clone(), wasm_instance_exports),
        )?;
        exports.export(
            "__wasm_call_function",
            Function::new(ctx.clone(), wasm_call_function),
        )?;
        exports.export(
            "__wasm_memory_read",
            Function::new(ctx.clone(), wasm_memory_read),
        )?;
        exports.export(
            "__wasm_memory_write",
            Function::new(ctx.clone(), wasm_memory_write),
        )?;
        Ok(())
    }
}

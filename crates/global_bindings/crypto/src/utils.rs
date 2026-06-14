use rquickjs::{Array, ArrayBuffer, Ctx, IntoJs, Object, Result, TypedArray, Value};
use vole_core::webidl::buffer_source_to_vec;

pub(crate) fn value_to_bytes<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<Vec<u8>> {
    buffer_source_to_vec(ctx, value)
}

pub(crate) fn value_as_bytes<'js>(value: &Value<'js>) -> Option<Vec<u8>> {
    if let Ok(ta) = value.get::<TypedArray<'js, u8>>() {
        if let Some(bytes) = ta.as_bytes() {
            return Some(bytes.to_vec());
        }
    }
    if let Ok(ab) = value.get::<ArrayBuffer<'js>>() {
        return Some(ab.as_bytes().unwrap_or_default().to_vec());
    }
    if let Some(obj) = value.as_object() {
        if let Ok(buffer_val) = obj.get::<&str, Value>("buffer") {
            if let Ok(ab) = buffer_val.get::<ArrayBuffer<'js>>() {
                let offset = obj.get::<&str, i64>("byteOffset").unwrap_or(0).max(0) as usize;
                let length = obj.get::<&str, i64>("byteLength").unwrap_or(0).max(0) as usize;
                let bytes = ab.as_bytes().unwrap_or_default();
                if offset < bytes.len() {
                    let end = (offset + length).min(bytes.len());
                    return Some(bytes[offset..end].to_vec());
                }
            }
        }
    }
    None
}

pub(crate) fn js_value_to_json<'js>(
    ctx: &Ctx<'js>,
    value: &Value<'js>,
) -> Result<serde_json::Value> {
    if value.is_undefined() || value.is_null() {
        return Ok(serde_json::Value::Null);
    }
    if let Some(b) = value.as_bool() {
        return Ok(serde_json::Value::Bool(b));
    }
    if let Some(n) = value.as_number() {
        let json_val = if n.is_finite() && n.fract() == 0.0 {
            if n >= 0.0 && n <= u64::MAX as f64 {
                serde_json::Value::Number((n as u64).into())
            } else if n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                serde_json::Value::Number((n as i64).into())
            } else {
                serde_json::Number::from_f64(n)
                    .map_or(serde_json::Value::Null, serde_json::Value::Number)
            }
        } else {
            serde_json::Number::from_f64(n)
                .map_or(serde_json::Value::Null, serde_json::Value::Number)
        };
        return Ok(json_val);
    }
    if let Some(s) = value.as_string() {
        return Ok(serde_json::Value::String(
            s.to_string().ok().unwrap_or_default(),
        ));
    }
    if let Some(bytes) = value_as_bytes(value) {
        return Ok(serde_json::Value::Array(
            bytes
                .into_iter()
                .map(|b| serde_json::Value::Number(b.into()))
                .collect(),
        ));
    }
    if let Ok(arr) = value.get::<Array<'js>>() {
        let mut out = Vec::new();
        for v in arr.iter::<Value<'js>>() {
            out.push(js_value_to_json(ctx, &v?)?);
        }
        return Ok(serde_json::Value::Array(out));
    }
    if let Some(obj) = value.as_object() {
        let mut map = serde_json::Map::new();
        for entry in obj.props::<String, Value>() {
            let (k, v) = entry?;
            map.insert(k, js_value_to_json(ctx, &v)?);
        }
        return Ok(serde_json::Value::Object(map));
    }
    Ok(serde_json::Value::Null)
}

pub(crate) fn json_value_to_js_value<'js>(
    ctx: &Ctx<'js>,
    value: &serde_json::Value,
) -> Result<Value<'js>> {
    match value {
        serde_json::Value::Null => Ok(Value::new_null(ctx.clone())),
        serde_json::Value::Bool(b) => Ok(Value::new_bool(ctx.clone(), *b)),
        serde_json::Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            Ok(Value::new_number(ctx.clone(), f))
        }
        serde_json::Value::String(s) => Ok(s.clone().into_js(ctx)?),
        serde_json::Value::Array(arr) => {
            let a = Array::new(ctx.clone())?;
            for (i, v) in arr.iter().enumerate() {
                a.set(i, json_value_to_js_value(ctx, v)?)?;
            }
            Ok(a.into_value())
        }
        serde_json::Value::Object(map) => {
            let o = Object::new(ctx.clone())?;
            for (k, v) in map {
                o.set(k.as_str(), json_value_to_js_value(ctx, v)?)?;
            }
            Ok(o.into_value())
        }
    }
}

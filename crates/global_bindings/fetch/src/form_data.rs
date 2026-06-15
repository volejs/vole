// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use std::{cell::RefCell, io::Write, rc::Rc};

use hyper::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use vole_global_blob::{Blob, File};
use vole_core::{class::IteratorDef, object::map_to_entries, result::ResultExt};
use rand::RngExt;
use rquickjs::{
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::{Opt, This},
    Array, Class, Coerced, Ctx, Exception, FromJs, Function, IntoJs, JsLifetime, Result, Value,
};

#[derive(Clone, Trace, JsLifetime)]
enum FormValue<'js> {
    Text(String),
    File(File<'js>),
}

impl<'js> IntoJs<'js> for FormValue<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        match self {
            FormValue::Text(s) => s.into_js(ctx),
            FormValue::File(f) => f.into_js(ctx),
        }
    }
}

#[derive(Clone, Default)]
struct FormEntries<'js>(Vec<(String, FormValue<'js>)>);

impl<'js> Trace<'js> for FormEntries<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.0.trace(tracer);
    }
}

unsafe impl<'js> JsLifetime<'js> for FormEntries<'js> {
    type Changed<'to> = FormEntries<'to>;
}

impl<'js> std::ops::Deref for FormEntries<'js> {
    type Target = Vec<(String, FormValue<'js>)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'js> std::ops::DerefMut for FormEntries<'js> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, JsLifetime, Default)]
#[rquickjs::class]
pub struct FormData<'js> {
    entries: Rc<RefCell<FormEntries<'js>>>,
}

impl<'js> Trace<'js> for FormData<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        if let Ok(entries) = self.entries.try_borrow() {
            entries.trace(tracer);
        }
    }
}

impl<'js> IteratorDef<'js> for FormData<'js> {
    fn js_entries(&self, ctx: Ctx<'js>) -> Result<Array<'js>> {
        let entries = self.entries.borrow();
        map_to_entries(&ctx, entries.0.clone())
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> FormData<'js> {
    #[qjs(constructor)]
    pub fn new(_form: Opt<Value<'js>>, _submitter: Opt<Value<'js>>) -> Self {
        Self::default()
    }

    pub fn append(
        &self,
        ctx: Ctx<'js>,
        name: String,
        value: Value<'js>,
        filename: Opt<String>,
    ) -> Result<()> {
        let entry = value_to_form_entry(&ctx, name, value, filename.0)?;
        self.entries.borrow_mut().push(entry);
        Ok(())
    }

    pub fn get(&self, ctx: Ctx<'js>, name: String) -> Result<Option<Value<'js>>> {
        let value = self
            .entries
            .borrow()
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.clone());
        match value {
            Some(v) => Ok(v.into_js(&ctx).ok()),
            None => Ok(None),
        }
    }

    pub fn get_all(&self, ctx: Ctx<'js>, name: String) -> Result<Vec<Value<'js>>> {
        let values: Vec<FormValue<'js>> = self
            .entries
            .borrow()
            .iter()
            .filter(|(k, _)| *k == name)
            .map(|(_, v)| v.clone())
            .collect();
        Ok(values
            .into_iter()
            .filter_map(|v| v.into_js(&ctx).ok())
            .collect())
    }

    pub fn has(&self, _ctx: Ctx<'js>, name: String) -> Result<bool> {
        let entries = self.entries.borrow();
        Ok(entries.iter().any(|(n, _)| n == &name))
    }

    pub fn set(
        &self,
        ctx: Ctx<'js>,
        name: String,
        value: Value<'js>,
        filename: Opt<String>,
    ) -> Result<()> {
        let entry = value_to_form_entry(&ctx, name, value, filename.0)?;
        let mut entries = self.entries.borrow_mut();
        if let Some(index) = entries.iter().position(|(k, _)| *k == entry.0) {
            let name = entry.0.clone();
            entries[index] = entry;
            let mut seen = false;
            entries.retain(|(k, _)| {
                if *k != name {
                    true
                } else if seen {
                    false
                } else {
                    seen = true;
                    true
                }
            });
        } else {
            entries.push(entry);
        }
        Ok(())
    }

    pub fn delete(&self, _ctx: Ctx<'js>, name: String) -> Result<()> {
        self.entries.borrow_mut().retain(|(k, _)| *k != name);
        Ok(())
    }

    pub fn keys(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let keys = Array::new(ctx.clone())?;
        for (index, (name, _)) in self.entries.borrow().iter().enumerate() {
            keys.set(index, name.clone())?;
        }
        array_values_iterator(keys)
    }

    pub fn values(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let values = Array::new(ctx.clone())?;
        for (index, (_, value)) in self.entries.borrow().iter().enumerate() {
            values.set(index, value.clone().into_js(&ctx)?)?;
        }
        array_values_iterator(values)
    }

    pub fn entries(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        self.js_iterator(ctx)
    }

    #[qjs(rename = PredefinedAtom::SymbolIterator)]
    pub fn iterator(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        self.js_iterator(ctx)
    }

    pub fn for_each(this: This<Class<'js, FormData<'js>>>, callback: Function<'js>) -> Result<()> {
        let ctx = this.0.ctx().clone();
        let entries = this.0.borrow().entries.borrow().0.clone();
        for (name, value) in entries.into_iter() {
            let val = value.into_js(&ctx)?;
            () = callback.call((val, name, this.0.clone()))?;
        }
        Ok(())
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        stringify!(FormData)
    }
}

impl<'js> FormData<'js> {
    #[allow(private_interfaces)]
    pub fn iter(&self) -> Vec<(String, FormValue<'js>)> {
        self.entries.borrow().0.clone()
    }

    pub fn from_multipart_bytes(
        ctx: &Ctx<'js>,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<Self> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }

        let Some(boundary) = extract_boundary(content_type) else {
            return Ok(Self::default());
        };
        let boundary_marker = ["--", &boundary].concat().into_bytes();

        let mut entries = Vec::new();
        let parts = bytes.split(|b| *b == b'\n').collect::<Vec<_>>();

        let mut current_headers = Vec::new();
        let mut current_data = Vec::new();
        let mut in_headers = false;

        let mut name: Option<String> = None;
        let mut filename: Option<String> = None;
        let mut mime_type: Option<String> = None;

        for line in parts {
            if line.starts_with(&boundary_marker) {
                if !current_data.is_empty() && name.is_some() {
                    let data = std::mem::take(&mut current_data);
                    if let Some(filename) = filename.take() {
                        let file = File::from_bytes(ctx, data, filename, mime_type)?;
                        entries.push((name.take().or_throw(ctx)?, FormValue::File(file)));
                    } else {
                        let text = String::from_utf8_lossy(&data).into_owned();
                        entries.push((name.take().or_throw(ctx)?, FormValue::Text(text)));
                    }
                }
                current_headers.clear();
                current_data.clear();
                name = None;
                filename = None;
                mime_type = None;
                in_headers = true;
                continue;
            }

            if in_headers {
                let line_str = String::from_utf8_lossy(line);
                if line_str.trim().is_empty() {
                    in_headers = false;
                } else {
                    current_headers.push(line_str.to_string());
                    if starts_with_ignore_case(&line_str, CONTENT_DISPOSITION.as_str()) {
                        for seg in line_str.split(';') {
                            let seg = seg.trim();
                            if let Some(n) = seg.strip_prefix("name=") {
                                name = Some(n.trim_matches('"').into());
                            } else if let Some(f) = seg.strip_prefix("filename=") {
                                filename = Some(f.trim_matches('"').into());
                            }
                        }
                    } else if starts_with_ignore_case(&line_str, CONTENT_TYPE.as_str()) {
                        if let Some(ct) = line_str.split(':').nth(1) {
                            mime_type = Some(ct.trim().into());
                        }
                    }
                }
            } else {
                current_data.extend_from_slice(line);
                current_data.push(b'\n');
            }
        }

        Ok(Self {
            entries: Rc::new(RefCell::new(FormEntries(entries))),
        })
    }

    pub fn to_multipart_bytes(&self, _ctx: &Ctx<'js>) -> Result<(Vec<u8>, String)> {
        let boundary = generate_boundary();
        let mut body = Vec::new();
        let entries = self.entries.borrow();
        if entries.is_empty() {
            // Empty FormData serialises to an empty body in browsers.
            return Ok((body, boundary));
        }

        for (name, value) in entries.iter() {
            match value {
                FormValue::Text(text) => {
                    write!(
                        body,
                        "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{text}\r\n"
                    )?;
                },
                FormValue::File(file) => {
                    let filename = file.name().clone();
                    let content_type = file.mime_type().clone();
                    let blob = file.get_blob();
                    let bytes = blob.as_bytes();
                    write!(
                        body,
                        "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
                    )?;
                    body.extend_from_slice(bytes);
                    body.extend_from_slice(b"\r\n");
                },
            }
        }

        write!(body, "--{boundary}--\r\n")?;

        Ok((body, boundary))
    }
}

fn value_to_form_entry<'js>(
    ctx: &Ctx<'js>,
    name: String,
    value: Value<'js>,
    filename: Option<String>,
) -> Result<(String, FormValue<'js>)> {
    if let Some(obj) = value.as_object() {
        if let Some(f) = Class::<File>::from_object(obj) {
            if let Some(filename) = filename {
                let blob = f.borrow().get_blob();
                return Ok((
                    name,
                    FormValue::File(File::from_bytes(
                        ctx,
                        blob.get_bytes(),
                        filename,
                        Some(blob.mime_type()),
                    )?),
                ));
            }
            return Ok((name, FormValue::File(f.borrow().clone())));
        }
        if let Some(b) = Class::<Blob>::from_object(obj) {
            let blob = b.borrow();
            return Ok((
                name,
                FormValue::File(File::from_bytes(
                    ctx,
                    blob.get_bytes(),
                    filename.unwrap_or_else(|| "blob".into()),
                    Some(blob.mime_type()),
                )?),
            ));
        }
    }
    if filename.is_some() {
        return Err(Exception::throw_type(
            ctx,
            "FormData filename can only be specified for Blob values",
        ));
    }
    if let Some(s) = value.as_string() {
        return Ok((name, FormValue::Text(s.to_string().or_throw(ctx)?)));
    }
    let Coerced(text) = Coerced::<String>::from_js(ctx, value)?;
    Ok((name, FormValue::Text(text)))
}

fn extract_boundary(content_type: &str) -> Option<String> {
    content_type.split(';').find_map(|part| {
        let part = part.trim();
        part.find("boundary=").map(|idx| {
            part[(idx + "boundary=".len())..]
                .trim()
                .trim_matches('"')
                .into()
        })
    })
}

fn generate_boundary() -> String {
    let rand_string: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    ["----WebKitFormBoundary", &rand_string].concat()
}

fn starts_with_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.len() >= needle.len()
        && haystack.as_bytes()[..needle.len()].eq_ignore_ascii_case(needle.as_bytes())
}

fn array_values_iterator<'js>(array: Array<'js>) -> Result<Value<'js>> {
    let values_fn: Function = array.as_object().get(PredefinedAtom::Values)?;
    values_fn.call((This(array),))
}

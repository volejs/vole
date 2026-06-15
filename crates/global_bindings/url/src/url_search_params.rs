use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rquickjs::{
    Array, Class, Coerced, Ctx, Exception, FromJs, Function, IntoJs, Null, Object, Result, Symbol,
    Value,
    atom::PredefinedAtom,
    class::Trace,
    function::{Opt, This},
};
use url::Url;
use vole_core::{
    class::IteratorDef,
    primordials::{BasePrimordials, Primordial},
    string::get_lossy_string,
};

use super::convert_trailing_space;

#[derive(Clone, Trace, rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct URLSearchParams {
    #[qjs(skip_trace)]
    pub url: Rc<RefCell<Url>>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> URLSearchParams {
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>, init: Opt<Value<'js>>) -> Result<Self> {
        if let Some(init) = init.into_inner() {
            if init.is_string() {
                let string = get_lossy_string(init)?;
                return Ok(Self::from_str(string));
            } else if init.is_array() {
                return Self::from_array(&ctx, unsafe { init.into_array().unwrap_unchecked() });
            } else if init.is_object() {
                return Self::from_object(&ctx, unsafe { init.into_object().unwrap_unchecked() });
            }
        }
        let url: Url = unsafe { "http://example.com".parse().unwrap_unchecked() };

        Ok(Self {
            url: Rc::new(RefCell::new(url)),
        })
    }

    #[qjs(get)]
    pub fn size(&self) -> usize {
        self.url.borrow().query_pairs().count()
    }

    #[qjs(prop, rename = PredefinedAtom::SymbolToStringTag, configurable)]
    pub fn to_string_tag() -> &'static str {
        stringify!(URLSearchParams)
    }

    pub fn append(&mut self, key: Coerced<String>, value: Coerced<String>) {
        convert_trailing_space(&mut self.url.borrow_mut());
        self.url
            .borrow_mut()
            .query_pairs_mut()
            .append_pair(key.as_str(), value.as_str());
        self.sync_query();
    }

    pub fn delete(&mut self, ctx: Ctx<'js>, key: Coerced<String>, value: Opt<Value<'js>>) {
        convert_trailing_space(&mut self.url.borrow_mut());

        let key = key.0;
        let value = get_coerced_string_value(&ctx, value);

        let new_pairs: Vec<_> = self
            .url
            .borrow()
            .query_pairs()
            .filter(|(k, v)| {
                if let Some(value) = value.as_ref() {
                    return !(*k == key && *v == *value);
                }
                *k != key
            })
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        if !new_pairs.is_empty() {
            self.url
                .borrow_mut()
                .query_pairs_mut()
                .clear()
                .extend_pairs(new_pairs);
        } else {
            self.url.borrow_mut().set_query(None);
        }
        self.sync_query();
    }

    pub fn entries(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        self.js_iterator(ctx)
    }

    #[qjs(rename = "forEach")]
    pub fn for_each(
        &self,
        ctx: Ctx<'js>,
        callback: Function<'js>,
        this_arg: Opt<Value<'js>>,
    ) -> Result<()> {
        let this_arg = this_arg
            .0
            .unwrap_or_else(|| Value::new_undefined(ctx.clone()));
        self.url
            .borrow()
            .query_pairs()
            .into_owned()
            .try_for_each(|(k, v)| callback.call((This(this_arg.clone()), v, k, self.clone())))?;
        Ok(())
    }

    pub fn get(&mut self, ctx: Ctx<'js>, key: Coerced<String>) -> Result<Value<'js>> {
        let key = key.0;
        match self
            .url
            .borrow()
            .query_pairs()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
        {
            Some(value) => value.into_js(&ctx),
            None => Null.into_js(&ctx),
        }
    }

    #[qjs(rename = "getAll")]
    pub fn get_all(&mut self, key: Coerced<String>) -> Vec<String> {
        let key = key.0;
        self.url
            .borrow()
            .query_pairs()
            .filter_map(|(k, v)| if k == key { Some(v.to_string()) } else { None })
            .collect()
    }

    pub fn has(&self, ctx: Ctx<'js>, key: Coerced<String>, value: Opt<Value<'js>>) -> bool {
        let value = get_coerced_string_value(&ctx, value);
        let key = key.0;
        self.url.borrow().query_pairs().any(|(k, v)| {
            if let Some(value) = value.as_ref() {
                return *k == key && *v == *value;
            }
            *k == key
        })
    }

    pub fn keys(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let array = Array::new(ctx.clone())?;
        for (idx, (key, _)) in self.url.borrow().query_pairs().into_owned().enumerate() {
            array.set(idx, key)?;
        }
        let values_fn: Function = array.as_object().get(PredefinedAtom::Values)?;
        values_fn.call((rquickjs::function::This(array),))
    }

    pub fn set(&mut self, key: Coerced<String>, value: Coerced<String>) {
        convert_trailing_space(&mut self.url.borrow_mut());

        let key = key.0;
        let value = value.0;

        let mut new_query_pairs: Vec<(String, String)> = Vec::new();
        let mut found = false;

        for (k, v) in self.url.borrow().query_pairs() {
            if k == key {
                if !found {
                    new_query_pairs.push((key.clone(), value.clone()));
                    found = true;
                }
            } else {
                new_query_pairs.push((k.to_string(), v.to_string()));
            }
        }

        if !found {
            new_query_pairs.push((key, value));
        }

        self.url
            .borrow_mut()
            .query_pairs_mut()
            .clear()
            .extend_pairs(new_query_pairs);
        self.sync_query();
    }

    pub fn sort(&mut self) {
        let mut new_pairs: Vec<(String, String)> =
            self.url.borrow().query_pairs().into_owned().collect();
        new_pairs.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));

        if new_pairs.is_empty() {
            self.url.borrow_mut().set_query(None);
        } else {
            self.url
                .borrow_mut()
                .query_pairs_mut()
                .clear()
                .extend_pairs(new_pairs);
        }
        self.sync_query();
    }

    pub fn to_string(&self) -> String {
        let url = self.url.borrow();
        url.query_pairs().fold(
            String::with_capacity(url.query().map_or(0, |q| q.len())),
            |mut acc, (key, value)| {
                if !acc.is_empty() {
                    acc.push('&');
                }
                url::form_urlencoded::byte_serialize(key.as_bytes()).for_each(|b| acc.push_str(b));
                acc.push('=');
                url::form_urlencoded::byte_serialize(value.as_bytes())
                    .for_each(|b| acc.push_str(b));
                acc
            },
        )
    }

    pub fn values(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let array = Array::new(ctx.clone())?;
        for (idx, (_, value)) in self.url.borrow().query_pairs().into_owned().enumerate() {
            array.set(idx, value)?;
        }
        let values_fn: Function = array.as_object().get(PredefinedAtom::Values)?;
        values_fn.call((rquickjs::function::This(array),))
    }

    #[qjs(rename = PredefinedAtom::SymbolIterator)]
    pub fn iterator(&self, ctx: Ctx<'js>) -> Result<Class<'js, URLSearchParamsIter>> {
        Class::instance(
            ctx,
            URLSearchParamsIter {
                index: 0,
                params: self.clone(),
            },
        )
    }
}

impl<'js> URLSearchParams {
    fn sync_query(&self) {
        let query = self.to_string();
        let mut url = self.url.borrow_mut();
        if query.is_empty() {
            url.set_query(None);
        } else {
            url.set_query(Some(&query));
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(query: String) -> Self {
        let query = if !query.starts_with('?') {
            ["?", &query].concat()
        } else {
            query
        };
        let url = unsafe {
            "http://example.com"
                .parse::<Url>()
                .unwrap_unchecked()
                .join(&query)
                .unwrap_unchecked()
        };
        Self {
            url: Rc::new(RefCell::new(url)),
        }
    }

    pub fn from_url(url: &Rc<RefCell<Url>>) -> Self {
        Self {
            url: Rc::clone(url),
        }
    }

    pub fn from_array(ctx: &Ctx<'js>, array: Array<'js>) -> Result<Self> {
        let mut url: Url = "http://example.com".parse().unwrap();
        let query_pairs: Vec<(String, String)> = array
            .into_iter()
            .map(|value| {
                if let Ok(value) = value {
                    if let Some(pair) = value.as_array() {
                        if pair.len() == 2 {
                            let key_val: Value = pair.get(0)?;
                            let val_val: Value = pair.get(1)?;
                            let key = if key_val.is_string() {
                                get_lossy_string(key_val)?
                            } else {
                                Coerced::<String>::from_js(ctx, key_val)?.0
                            };
                            let value = if val_val.is_string() {
                                get_lossy_string(val_val)?
                            } else {
                                Coerced::<String>::from_js(ctx, val_val)?.0
                            };
                            return Ok((key, value));
                        }
                    }
                };
                Err(Exception::throw_type(
                    ctx,
                    "Invalid tuple: Each query pair must be an iterable [name, value] tuple",
                ))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect();

        url.query_pairs_mut().extend_pairs(query_pairs);

        Ok(Self {
            url: Rc::new(RefCell::new(url)),
        })
    }

    pub fn from_object(ctx: &Ctx<'js>, object: Object<'js>) -> Result<Self> {
        let iterator = Symbol::iterator(ctx.clone());
        if object.contains_key(iterator)? {
            let query_pairs: Array = BasePrimordials::get(ctx)?
                .function_array_from
                .call((object,))?;
            return Self::from_array(ctx, query_pairs);
        }

        let mut url: Url = "http://example.com".parse().unwrap();
        let raw_pairs: Vec<(String, String)> = object
            .keys::<Value<'js>>()
            .map(|key| {
                let key = key?;
                let key_string = if key.is_string() {
                    get_lossy_string(key.clone())?
                } else {
                    Coerced::<String>::from_js(ctx, key.clone())?.0
                };
                let value_val: Value = object.get(key)?;
                let value = if value_val.is_string() {
                    get_lossy_string(value_val)?
                } else {
                    Coerced::<String>::from_js(ctx, value_val)?.0
                };
                Ok((key_string, value))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut order: Vec<String> = Vec::with_capacity(raw_pairs.len());
        let mut map: HashMap<String, String> = HashMap::with_capacity(raw_pairs.len());
        for (k, v) in raw_pairs {
            if !map.contains_key(&k) {
                order.push(k.clone());
            }
            map.insert(k, v);
        }
        let query_pairs: Vec<(String, String)> = order
            .into_iter()
            .map(|k| {
                let v = map.remove(&k).unwrap_or_default();
                (k, v)
            })
            .collect();

        url.query_pairs_mut().extend_pairs(query_pairs);

        Ok(Self {
            url: Rc::new(RefCell::new(url)),
        })
    }
}

#[derive(Trace, rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct URLSearchParamsIter {
    params: URLSearchParams,
    index: u32,
}

#[rquickjs::methods]
impl<'js> URLSearchParamsIter {
    pub fn next(&mut self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let obj = Object::new(ctx.clone())?;
        let value = (*self.params.url.borrow())
            .query_pairs()
            .nth(self.index as _)
            .map(|(k, v)| vec![k.to_string(), v.to_string()]);

        if let Some(value) = value {
            obj.set("done", false)?;
            obj.set("value", value)?;
        } else {
            obj.set("done", true)?;
        }

        self.index += 1;

        Ok(obj)
    }
}

impl<'js> IteratorDef<'js> for URLSearchParams {
    fn js_entries(&self, ctx: Ctx<'js>) -> Result<Array<'js>> {
        let array = Array::new(ctx.clone())?;
        for (idx, (key, value)) in self.url.borrow().query_pairs().into_owned().enumerate() {
            let entry = Array::new(ctx.clone())?;
            entry.set(0, key)?;
            entry.set(1, value)?;
            array.set(idx, entry)?;
        }
        Ok(array)
    }
}

fn get_coerced_string_value<'js>(ctx: &Ctx<'js>, value: Opt<Value<'js>>) -> Option<String> {
    if let Some(value) = value.0 {
        if !value.is_undefined() {
            if let Ok(value) = Coerced::<String>::from_js(ctx, value) {
                return Some(value.0);
            }
        }
    };
    None
}

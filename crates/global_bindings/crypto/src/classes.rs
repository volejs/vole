use rquickjs::class::{JsClass, Trace, Tracer, Writable};
use rquickjs::function::This;
use rquickjs::{Array, Class, Constructor, Ctx, Function, IntoJs, Object, Result, Value};

use crate::cipher::{crypto_subtle_decrypt, crypto_subtle_encrypt};
use crate::digest::crypto_subtle_digest;
use crate::generate_key::crypto_subtle_generate_key;
use crate::import_export::{crypto_subtle_export_key, crypto_subtle_import_key};
use crate::random::{crypto_get_random_values, crypto_random_uuid};
use crate::signature::{crypto_subtle_sign, crypto_subtle_verify};
use crate::utils::json_value_to_js_value;

// =============================================================================
// Native classes
// =============================================================================

#[derive(Clone, Debug)]
pub(crate) struct Crypto;

unsafe impl<'js> rquickjs::JsLifetime<'js> for Crypto {
    type Changed<'to> = Crypto;
}

impl<'js> Trace<'js> for Crypto {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for Crypto {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> JsClass<'js> for Crypto {
    const NAME: &'static str = "Crypto";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set(
            "getRandomValues",
            Function::new(ctx.clone(), crypto_get_random_values)?,
        )?;
        proto.set(
            "randomUUID",
            Function::new(ctx.clone(), crypto_random_uuid)?,
        )?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor<'js>(_ctx: Ctx<'js>) -> Result<Crypto> {
            Ok(Crypto)
        }
        let constr = Constructor::new_class::<Crypto, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SubtleCrypto;

unsafe impl<'js> rquickjs::JsLifetime<'js> for SubtleCrypto {
    type Changed<'to> = SubtleCrypto;
}

impl<'js> Trace<'js> for SubtleCrypto {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for SubtleCrypto {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> JsClass<'js> for SubtleCrypto {
    const NAME: &'static str = "SubtleCrypto";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("digest", Function::new(ctx.clone(), crypto_subtle_digest)?)?;
        proto.set(
            "generateKey",
            Function::new(ctx.clone(), crypto_subtle_generate_key)?,
        )?;
        proto.set(
            "encrypt",
            Function::new(ctx.clone(), crypto_subtle_encrypt)?,
        )?;
        proto.set(
            "decrypt",
            Function::new(ctx.clone(), crypto_subtle_decrypt)?,
        )?;
        proto.set("sign", Function::new(ctx.clone(), crypto_subtle_sign)?)?;
        proto.set("verify", Function::new(ctx.clone(), crypto_subtle_verify)?)?;
        proto.set(
            "importKey",
            Function::new(ctx.clone(), crypto_subtle_import_key)?,
        )?;
        proto.set(
            "exportKey",
            Function::new(ctx.clone(), crypto_subtle_export_key)?,
        )?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor<'js>(_ctx: Ctx<'js>) -> Result<SubtleCrypto> {
            Ok(SubtleCrypto)
        }
        let constr = Constructor::new_class::<SubtleCrypto, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CryptoKey {
    pub(crate) id: u64,
    pub(crate) key_type: String,
    pub(crate) extractable: bool,
    pub(crate) algorithm: serde_json::Value,
    pub(crate) usages: Vec<String>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for CryptoKey {
    type Changed<'to> = CryptoKey;
}

impl<'js> Trace<'js> for CryptoKey {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for CryptoKey {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> JsClass<'js> for CryptoKey {
    const NAME: &'static str = "CryptoKey";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set("type", Function::new(ctx.clone(), crypto_key_type)?)?;
        proto.set(
            "extractable",
            Function::new(ctx.clone(), crypto_key_extractable)?,
        )?;
        proto.set(
            "algorithm",
            Function::new(ctx.clone(), crypto_key_algorithm)?,
        )?;
        proto.set("usages", Function::new(ctx.clone(), crypto_key_usages)?)?;
        proto.set("_id", Function::new(ctx.clone(), crypto_key_id)?)?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        fn ctor<'js>(_ctx: Ctx<'js>) -> Result<CryptoKey> {
            Err(rquickjs::Error::Exception)
        }
        let constr = Constructor::new_class::<CryptoKey, _, _>(ctx.clone(), ctor)?;
        Ok(Some(constr))
    }
}

pub(crate) fn crypto_key_type<'js>(this: This<Class<'js, CryptoKey>>) -> Result<String> {
    Ok(this.try_borrow()?.key_type.clone())
}

pub(crate) fn crypto_key_extractable<'js>(this: This<Class<'js, CryptoKey>>) -> Result<bool> {
    Ok(this.try_borrow()?.extractable)
}

pub(crate) fn crypto_key_algorithm<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, CryptoKey>>,
) -> Result<Value<'js>> {
    let alg = this.try_borrow()?.algorithm.clone();
    json_value_to_js_value(&ctx, &alg)
}

pub(crate) fn crypto_key_usages<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, CryptoKey>>,
) -> Result<Array<'js>> {
    let arr = Array::new(ctx.clone())?;
    for (i, u) in this.try_borrow()?.usages.iter().enumerate() {
        arr.set(i, u.clone())?;
    }
    Ok(arr)
}

pub(crate) fn crypto_key_id<'js>(this: This<Class<'js, CryptoKey>>) -> Result<u64> {
    Ok(this.try_borrow()?.id)
}

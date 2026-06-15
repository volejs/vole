use std::{convert::Infallible, io::Cursor, sync::Arc};

use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{client::legacy::{connect::HttpConnector, Client}, rt::TokioExecutor};
use once_cell::sync::Lazy;
use rquickjs::{class::Trace, function::Opt, Class, Ctx, Exception, JsLifetime, Result, Value};
use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime},
    ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme,
};
use vole_core::{bytes::ObjectBytes, object::ObjectExt, result::ResultExt};

pub type HyperClient = Client<HttpsConnector<HttpConnector>, BoxBody<Bytes, Infallible>>;

pub static HTTP_CLIENT: Lazy<std::result::Result<HyperClient, Box<dyn std::error::Error + Send + Sync>>> =
    Lazy::new(|| build_client(None));

#[derive(Debug)]
pub struct ClientTlsConfig {
    reject_unauthorized: bool,
    ca: Option<Vec<Vec<u8>>>,
}

impl Default for ClientTlsConfig {
    fn default() -> Self {
        Self {
            reject_unauthorized: true,
            ca: None,
        }
    }
}

#[derive(Debug)]
struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}

pub fn build_client(
    tls_config: Option<ClientTlsConfig>,
) -> std::result::Result<HyperClient, Box<dyn std::error::Error + Send + Sync>> {
    let connector = match tls_config {
        Some(tls_config) => HttpsConnectorBuilder::new()
            .with_tls_config(build_tls_config(tls_config)?)
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build(),
        None => HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build(),
    };

    Ok(Client::builder(TokioExecutor::new()).build(connector))
}

fn build_tls_config(
    tls_config: ClientTlsConfig,
) -> std::result::Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>> {
    if !tls_config.reject_unauthorized {
        return Ok(ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
            .with_no_client_auth());
    }

    let mut roots = RootCertStore::empty();
    if let Some(ca) = tls_config.ca {
        for certs in ca {
            add_ca_certificates(&mut roots, &certs)?;
        }
    } else {
        roots.roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    Ok(ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth())
}

fn add_ca_certificates(
    roots: &mut RootCertStore,
    bytes: &[u8],
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cursor = Cursor::new(bytes);
    let certs = rustls_pemfile::certs(&mut cursor)
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if certs.is_empty() {
        roots.add(CertificateDer::from(bytes.to_vec()))?;
    } else {
        for cert in certs {
            roots.add(cert)?;
        }
    }

    Ok(())
}

#[rquickjs::class]
#[derive(Clone, JsLifetime, Trace)]
pub struct Agent {
    #[qjs(skip_trace)]
    client: HyperClient,
}

#[rquickjs::methods]
impl Agent {
    #[qjs(constructor)]
    pub fn new<'js>(ctx: Ctx<'js>, options: Opt<Value<'js>>) -> Result<Self> {
        let client = match parse_agent_tls_config(&ctx, options.0)? {
            Some(tls_config) => build_client(Some(tls_config)).or_throw_msg(
                &ctx,
                "Failed to build HTTP client",
            )?,
            None => HTTP_CLIENT
                .as_ref()
                .map_err(|err| Exception::throw_type(&ctx, &err.to_string()))?
                .clone(),
        };
        Ok(Self { client })
    }
}

impl Agent {
    pub(crate) fn client(&self) -> HyperClient {
        self.client.clone()
    }
}

pub fn define(ctx: &Ctx<'_>) -> Result<()> {
    Class::<Agent>::define(&ctx.globals())
}

fn parse_agent_tls_config<'js>(
    ctx: &Ctx<'js>,
    options: Option<Value<'js>>,
) -> Result<Option<ClientTlsConfig>> {
    let Some(options) = options else {
        return Ok(None);
    };
    if options.is_undefined() || options.is_null() {
        return Ok(None);
    }

    let object = options
        .as_object()
        .ok_or_else(|| Exception::throw_type(ctx, "Agent options must be an object"))?;

    let reject_unauthorized = object
        .get_optional::<_, bool>("rejectUnauthorized")?
        .unwrap_or(true);
    let ca = object
        .get_optional::<_, Value>("ca")?
        .filter(|ca| !ca.is_undefined() && !ca.is_null())
        .map(|ca| parse_ca_option(ctx, ca))
        .transpose()?;

    Ok(Some(ClientTlsConfig {
        reject_unauthorized,
        ca,
    }))
}

fn parse_ca_option<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Vec<Vec<u8>>> {
    if value.is_array() {
        let array = unsafe { value.into_array().unwrap_unchecked() };
        let mut entries = Vec::with_capacity(array.len());
        for entry in array.into_iter().flatten() {
            entries.push(ca_value_to_bytes(ctx, entry)?);
        }
        return Ok(entries);
    }

    Ok(vec![ca_value_to_bytes(ctx, value)?])
}

fn ca_value_to_bytes<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Vec<u8>> {
    if value.is_string() {
        let ca: String = value.get()?;
        return Ok(ca.into_bytes());
    }

    ObjectBytes::from(ctx, &value)?.into_bytes(ctx)
}

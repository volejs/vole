use std::net::IpAddr;

use rquickjs::{Ctx, IntoJs, Object, Result, Value};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct DnsIpRecord {
    pub(crate) address: String,
    pub(crate) family: &'static str,
}

impl DnsIpRecord {
    pub(crate) fn from_ip(ip: IpAddr) -> Self {
        let family = match ip {
            IpAddr::V4(_) => "IPv4",
            IpAddr::V6(_) => "IPv6",
        };
        Self {
            address: ip.to_string(),
            family,
        }
    }
}

impl<'js> IntoJs<'js> for DnsIpRecord {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        let object = Object::new(ctx.clone())?;
        object.set("address", self.address)?;
        object.set("family", self.family)?;
        Ok(object.into_value())
    }
}

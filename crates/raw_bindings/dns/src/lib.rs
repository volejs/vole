use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    Class, Ctx, Function, Result,
};

mod record;
mod resolver;

use resolver::{lookup_host, lookup_ip, DnsResolver};

/// Native module exposing low-level DNS resolver operations.
pub struct DnsRawModule;

impl ModuleDef for DnsRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_dns_lookupHost")?;
        decl.declare("__std_dns_lookupIp")?;
        decl.declare("DnsResolver")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let globals = ctx.globals();
        Class::<DnsResolver>::define(&globals)?;

        exports.export(
            "__std_dns_lookupHost",
            Function::new(ctx.clone(), lookup_host)?,
        )?;
        exports.export("__std_dns_lookupIp", Function::new(ctx.clone(), lookup_ip)?)?;

        let resolver_ctor: Function<'js> = globals.get("DnsResolver")?;
        exports.export("DnsResolver", resolver_ctor)?;
        let _ = globals.remove("DnsResolver");

        Ok(())
    }
}

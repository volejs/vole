import {
  __std_dns_lookupHost,
  __std_dns_lookupIp,
  DnsResolver,
} from "vole:raw:dns";

export const lookupHost = __std_dns_lookupHost;
export const lookupIp = __std_dns_lookupIp;

export { DnsResolver };

export const dns = {
  lookupHost,
  lookupIp,
  DnsResolver,
};

export default dns;

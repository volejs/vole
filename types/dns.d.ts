/**
 * DNS resolver helpers exposed by `vole:dns`.
 *
 * The public module is native first: `DnsResolver` is a Rust native class and
 * this declaration describes the native API rather than a JavaScript wrapper.
 */
declare module "vole:dns" {
  /** IP address record returned by DNS host lookup APIs. */
  export interface DnsIpRecord {
    /** Text representation of the resolved IP address. */
    readonly address: string;
    /** IP address family. */
    readonly family: "IPv4" | "IPv6";
  }

  /** Options used when creating a native DNS resolver. */
  export interface DnsResolverOptions {
    /**
     * Enable the resolver-local JavaScript-visible cache.
     *
     * Defaults to `true`. The native Hickory resolver also maintains its own
     * protocol-level cache; `clearCache()` clears both layers.
     */
    cache?: boolean;
  }

  /**
   * Native DNS resolver.
   *
   * The resolver uses the host system DNS configuration through Hickory
   * Resolver. It currently exposes dual-stack host/IP lookup and cache control.
   */
  export class DnsResolver {
    /** Create a native resolver instance. */
    constructor(options?: DnsResolverOptions);

    /** Resolve a hostname to string IP addresses. */
    lookupHost(hostname: string): Promise<string[]>;

    /** Resolve a hostname to structured IP records. */
    lookupIp(hostname: string): Promise<DnsIpRecord[]>;

    /**
     * Clear cached records.
     *
     * When `hostname` is omitted, clears all resolver cache entries. When it is
     * provided, clears the A/AAAA cache for that hostname.
     */
    clearCache(hostname?: string): boolean;

    /** Return whether the resolver-local cache is enabled. */
    cacheEnabled(): boolean;
  }

  /** Resolve a hostname using the shared native resolver. */
  export function lookupHost(hostname: string): Promise<string[]>;

  /** Resolve a hostname to structured IP records using a native resolver. */
  export function lookupIp(hostname: string): Promise<DnsIpRecord[]>;

  /** Default export object. */
  const dns: {
    lookupHost: typeof lookupHost;
    lookupIp: typeof lookupIp;
    DnsResolver: typeof DnsResolver;
  };

  export default dns;
}

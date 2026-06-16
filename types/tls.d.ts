/**
 * TLS capability helpers exposed by `vole:tls`.
 *
 * 当前后端尚为占位实现，后续可在本模块中补齐客户端/服务端 TLS
 * 相关能力。
 */
declare module "vole:tls" {
  /** 当前实现是否开启。 */
  export function isSupported(): boolean;

  /** 兼容命名：同 `isSupported`。 */
  export function isAvailable(): boolean;

  /** 返回运行时 TLS 能力信息。 */
  export function getInfo(): {
    /** 是否已开启 TLS 支持。 */
    enabled: boolean;
  };

  /** 默认导出。 */
  const tls: {
    isSupported: typeof isSupported;
    isAvailable: typeof isAvailable;
    getInfo: typeof getInfo;
  };

  export default tls;
}

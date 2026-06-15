/**
 * Cross-platform path helpers exposed by `vole:path`.
 *
 * The API is close to common Node-style path helpers, but is provided under
 * vole's own module namespace and should be treated as a vole API rather than a
 * Node compatibility promise.
 *
 * @example
 * ```ts
 * import { join, dirname, parse } from "vole:path";
 *
 * const file = join("logs", "app.log");
 * console.log(dirname(file));
 * console.log(parse(file).ext);
 * ```
 *
 * @remarks
 * Results follow the host platform path rules. Avoid persisting `sep`-specific
 * paths in data files when the same project must run on multiple operating
 * systems.
 */
declare module "vole:path" {
  /** Parsed path object returned by `parse`. */
  export interface ParsedPath {
    /** Root segment, such as `/` on Unix or `C:\\` on Windows. */
    root: string;
    /** Directory path without the final base name. */
    dir: string;
    /** Final path component including extension. */
    base: string;
    /** Extension including the leading dot, or an empty string. */
    ext: string;
    /** Final path component without extension. */
    name: string;
  }

  /** Path object accepted by `format`. */
  export interface FormatPathObject {
    /** Root segment, such as `/` on Unix or `C:\\` on Windows. */
    root?: string;
    /** Directory path. Usually takes precedence over `root` when present. */
    dir?: string;
    /** Final path component including extension. Usually takes precedence over `name` and `ext`. */
    base?: string;
    /** Extension including the leading dot. */
    ext?: string;
    /** Final path component without extension. */
    name?: string;
  }

  /** Join path segments using the current platform semantics. */
  export function join(...parts: string[]): string;
  /** Resolve path segments into an absolute path. */
  export function resolve(...parts: string[]): string;
  /** Return the parent directory portion of a path. */
  export function dirname(path: string): string;
  /** Return the last path component, optionally removing `ext`. */
  export function basename(path: string, ext?: string): string;
  /** Return the file extension including the leading dot, or an empty string. */
  export function extname(path: string): string;
  /** Return whether `path` is absolute on the current platform. */
  export function isAbsolute(path: string): boolean;
  /** Return a relative path from `from` to `to`. */
  export function relative(from: string, to: string): string;
  /** Split a path string into structured fields. */
  export function parse(path: string): ParsedPath;
  /** Format a structured path object back into a path string. */
  export function format(pathObject: FormatPathObject): string;
  /** Current platform path separator. */
  export const sep: string;
  /** Current platform path-list delimiter. */
  export const delimiter: string;
}

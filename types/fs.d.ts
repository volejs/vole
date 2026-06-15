/**
 * Filesystem helpers exposed by `vole:fs`.
 *
 * The module provides path-based sync/async helpers and an async file-handle
 * API. Binary reads return `Uint8Array`; text reads return UTF-8 strings.
 *
 * @example
 * ```ts
 * import { mkdir, writeFile, readTextFile, open } from "vole:fs";
 *
 * await mkdir("tmp", { recursive: true });
 * await writeFile("tmp/hello.txt", "hello vole");
 * console.log(await readTextFile("tmp/hello.txt"));
 *
 * const file = await open("tmp/hello.txt");
 * console.log(await file.read(5));
 * await file.close();
 * ```
 *
 * @remarks
 * Sync helpers block the current VM while the filesystem operation runs. Use the
 * async helpers in servers or long-running applications. `FileData` is limited
 * to `string`, `Uint8Array`, and `ArrayBuffer`; other typed-array views are not
 * currently treated as binary data by the shim.
 */
declare module "vole:fs" {
  /** File path accepted by the current implementation. */
  export type PathLike = string;

  /** Data accepted by path-based write helpers. */
  export type FileData = string | Uint8Array | ArrayBuffer | null | undefined;

  /** Origin used by `FileHandle.seek`. */
  export type SeekWhence = "start" | "current" | "end";

  /** Directory creation options. */
  export interface MkdirOptions {
    /** Create parent directories as needed. */
    recursive?: boolean;
  }

  /** File opening options. Read access is enabled by default. */
  export interface OpenOptions {
    /** Enable write access. */
    write?: boolean;
    /** Append writes to the end of the file. */
    append?: boolean;
    /** Create the file if it does not exist. */
    create?: boolean;
    /** Truncate the file when opening. */
    truncate?: boolean;
  }

  /** File metadata returned by `stat` and `statSync`. */
  export class Stats {
    /**
     * Return the file size in bytes.
     *
     * Directories and special files use the value reported by the host OS.
     */
    size(): number;
    /** Return the last modification time, when available from the platform. */
    mtime(): Date | undefined;
    /** Return the creation/change time, when available from the platform. */
    ctime(): Date | undefined;
    /** Return the last modification timestamp in milliseconds. */
    mtimeMs(): number | undefined;
    /** Return the creation/change timestamp in milliseconds. */
    ctimeMs(): number | undefined;
    /** Return whether the path points to a regular file. */
    isFile(): boolean;
    /** Return whether the path points to a directory. */
    isDirectory(): boolean;
    /** Return whether the path points to a symbolic link. */
    isSymbolicLink(): boolean;
  }

  /** Async file handle returned by `open`. */
  export class FileHandle {
    /**
     * Read bytes from the current cursor position.
     *
     * @param size Maximum bytes to read. Defaults to the runtime's chunk size.
     * @returns Bytes read. Empty array can indicate EOF.
     */
    read(size?: number): Promise<Uint8Array>;
    /**
     * Write bytes at the current cursor position.
     *
     * @param data Bytes to write. Only `Uint8Array` is currently accepted.
     * @returns Number of bytes written.
     */
    write(data: Uint8Array): Promise<number>;
    /**
     * Move the file cursor.
     *
     * @param offset Offset relative to `whence`.
     * @param whence Cursor origin. Defaults to `"start"`.
     * @returns New cursor position.
     */
    seek(offset: number, whence?: SeekWhence): Promise<number>;
    /** Close this handle. Closing an already closed handle is a no-op. */
    close(): Promise<void>;
  }

  /** Read a file as bytes synchronously. */
  export function readFileSync(path: PathLike): Uint8Array;
  /** Read a UTF-8 text file synchronously. */
  export function readTextFileSync(path: PathLike): string;
  /** Write data to a file synchronously, replacing existing contents. */
  export function writeFileSync(path: PathLike, data: FileData): void;
  /** Return whether a path exists synchronously. */
  export function existsSync(path: PathLike): boolean;
  /** Create a directory synchronously. */
  export function mkdirSync(path: PathLike, options?: MkdirOptions): void;
  /** Read directory entry names synchronously. */
  export function readdirSync(path: PathLike): string[];
  /** Read file metadata synchronously. */
  export function statSync(path: PathLike): Stats;
  /** Remove a file or directory synchronously. */
  export function removeSync(path: PathLike): void;
  /** Rename or move a path synchronously. */
  export function renameSync(oldPath: PathLike, newPath: PathLike): void;
  /** Copy a file synchronously. */
  export function copyFileSync(src: PathLike, dest: PathLike): void;
  /** Resolve a path to its canonical real path synchronously. */
  export function realPathSync(path: PathLike): string;

  /** Read a file as bytes asynchronously. */
  export function readFile(path: PathLike): Promise<Uint8Array>;
  /** Read a UTF-8 text file asynchronously. */
  export function readTextFile(path: PathLike): Promise<string>;
  /** Write data to a file asynchronously, replacing existing contents. */
  export function writeFile(path: PathLike, data: FileData): Promise<void>;
  /** Return whether a path exists asynchronously. */
  export function exists(path: PathLike): Promise<boolean>;
  /** Create a directory asynchronously. */
  export function mkdir(path: PathLike, options?: MkdirOptions): Promise<void>;
  /** Read directory entry names asynchronously. */
  export function readdir(path: PathLike): Promise<string[]>;
  /** Read file metadata asynchronously. */
  export function stat(path: PathLike): Promise<Stats>;
  /** Remove a file or directory asynchronously. */
  export function remove(path: PathLike): Promise<void>;
  /** Rename or move a path asynchronously. */
  export function rename(oldPath: PathLike, newPath: PathLike): Promise<void>;
  /** Copy a file asynchronously. */
  export function copyFile(src: PathLike, dest: PathLike): Promise<void>;
  /** Resolve a path to its canonical real path asynchronously. */
  export function realPath(path: PathLike): Promise<string>;
  /** Open a file and return an async file handle. */
  export function open(path: PathLike, options?: OpenOptions): Promise<FileHandle>;

  /** Default object export containing all filesystem helpers and classes. */
  const fs: {
    /** See `readFileSync`. */
    readFileSync: typeof readFileSync;
    /** See `readTextFileSync`. */
    readTextFileSync: typeof readTextFileSync;
    /** See `writeFileSync`. */
    writeFileSync: typeof writeFileSync;
    /** See `existsSync`. */
    existsSync: typeof existsSync;
    /** See `mkdirSync`. */
    mkdirSync: typeof mkdirSync;
    /** See `readdirSync`. */
    readdirSync: typeof readdirSync;
    /** See `statSync`. */
    statSync: typeof statSync;
    /** See `removeSync`. */
    removeSync: typeof removeSync;
    /** See `renameSync`. */
    renameSync: typeof renameSync;
    /** See `copyFileSync`. */
    copyFileSync: typeof copyFileSync;
    /** See `realPathSync`. */
    realPathSync: typeof realPathSync;
    /** See `readFile`. */
    readFile: typeof readFile;
    /** See `readTextFile`. */
    readTextFile: typeof readTextFile;
    /** See `writeFile`. */
    writeFile: typeof writeFile;
    /** See `exists`. */
    exists: typeof exists;
    /** See `mkdir`. */
    mkdir: typeof mkdir;
    /** See `readdir`. */
    readdir: typeof readdir;
    /** See `stat`. */
    stat: typeof stat;
    /** See `remove`. */
    remove: typeof remove;
    /** See `rename`. */
    rename: typeof rename;
    /** See `copyFile`. */
    copyFile: typeof copyFile;
    /** See `realPath`. */
    realPath: typeof realPath;
    /** See `open`. */
    open: typeof open;
    /** File handle constructor. */
    FileHandle: typeof FileHandle;
    /** File metadata constructor. */
    Stats: typeof Stats;
  };
  export default fs;
}

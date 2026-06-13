import {
  __fs_readFileSync,
  __fs_readTextFileSync,
  __fs_writeFileSync,
  __fs_writeFileSyncBuf,
  __fs_existsSync,
  __fs_mkdirSync,
  __fs_readdirSync,
  __fs_statSync,
  __fs_removeSync,
  __fs_renameSync,
  __fs_copyFileSync,
  __fs_realPathSync,
  __fs_readFile,
  __fs_readTextFile,
  __fs_writeFile,
  __fs_writeFileBuf,
  __fs_exists,
  __fs_mkdir,
  __fs_readdir,
  __fs_stat,
  __fs_remove,
  __fs_rename,
  __fs_copyFile,
  __fs_realPath,
  __fs_open,
  FileHandle,
  Stats,
} from "vole:raw:fs";

function encodeData(data) {
  if (data instanceof Uint8Array) {
    return data;
  }
  if (data instanceof ArrayBuffer) {
    return new Uint8Array(data);
  }
  if (typeof data === "string") {
    return new TextEncoder().encode(data);
  }
  if (data != null) {
    return new TextEncoder().encode(String(data));
  }
  return new Uint8Array(0);
}

// --- Synchronous path-based API ---

export function readFileSync(path) {
  return __fs_readFileSync(path);
}

export function readTextFileSync(path) {
  return __fs_readTextFileSync(path);
}

export function writeFileSync(path, data) {
  const buf = encodeData(data);
  if (buf instanceof Uint8Array) {
    return __fs_writeFileSyncBuf(path, buf);
  }
  return __fs_writeFileSync(path, String(data));
}

export function existsSync(path) {
  return __fs_existsSync(path);
}

export function mkdirSync(path, options) {
  return __fs_mkdirSync(path, options?.recursive ?? false);
}

export function readdirSync(path) {
  return __fs_readdirSync(path);
}

export function statSync(path) {
  return __fs_statSync(path);
}

export function removeSync(path) {
  return __fs_removeSync(path);
}

export function renameSync(oldPath, newPath) {
  return __fs_renameSync(oldPath, newPath);
}

export function copyFileSync(src, dest) {
  return __fs_copyFileSync(src, dest);
}

export function realPathSync(path) {
  return __fs_realPathSync(path);
}

// --- Asynchronous path-based API (real async tokio::fs implementations) ---

export async function readFile(path) {
  return __fs_readFile(path);
}

export async function readTextFile(path) {
  return __fs_readTextFile(path);
}

export async function writeFile(path, data) {
  const buf = encodeData(data);
  if (buf instanceof Uint8Array) {
    return __fs_writeFileBuf(path, buf);
  }
  return __fs_writeFile(path, String(data));
}

export async function exists(path) {
  return __fs_exists(path);
}

export async function mkdir(path, options) {
  return __fs_mkdir(path, options?.recursive ?? false);
}

export async function readdir(path) {
  return __fs_readdir(path);
}

export async function stat(path) {
  return __fs_stat(path);
}

export async function remove(path) {
  return __fs_remove(path);
}

export async function rename(oldPath, newPath) {
  return __fs_rename(oldPath, newPath);
}

export async function copyFile(src, dest) {
  return __fs_copyFile(src, dest);
}

export async function realPath(path) {
  return __fs_realPath(path);
}

// --- File-handle API (merged from the old vole:file module) ---

export async function open(path, options = {}) {
  return __fs_open(path, options);
}

export { FileHandle, Stats };

export default {
  readFileSync,
  readTextFileSync,
  writeFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  statSync,
  removeSync,
  renameSync,
  copyFileSync,
  realPathSync,
  readFile,
  readTextFile,
  writeFile,
  exists,
  mkdir,
  readdir,
  stat,
  remove,
  rename,
  copyFile,
  realPath,
  open,
  FileHandle,
  Stats,
};

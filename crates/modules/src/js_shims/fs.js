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
} from "std:raw:fs";

// --- 同步 API ---

export function readFileSync(path) {
  return __fs_readFileSync(path);
}

export function readTextFileSync(path) {
  return __fs_readTextFileSync(path);
}

export function writeFileSync(path, data) {
  if (data instanceof Uint8Array) {
    return __fs_writeFileSyncBuf(path, data);
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
  const s = __fs_statSync(path);
  return {
    size: s.size,
    isFile: s.isFile,
    isDirectory: s.isDirectory,
    mtime: new Date(s.mtime),
    ctime: new Date(s.ctime),
  };
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

// --- 异步 API（基于同步实现的简化版本）---

export async function readFile(path) {
  return readFileSync(path);
}

export async function readTextFile(path) {
  return readTextFileSync(path);
}

export async function writeFile(path, data) {
  return writeFileSync(path, data);
}

export async function mkdir(path, options) {
  return mkdirSync(path, options);
}

export async function readdir(path) {
  return readdirSync(path);
}

export async function stat(path) {
  return statSync(path);
}

export async function remove(path) {
  return removeSync(path);
}

export async function rename(oldPath, newPath) {
  return renameSync(oldPath, newPath);
}

export async function copyFile(src, dest) {
  return copyFileSync(src, dest);
}

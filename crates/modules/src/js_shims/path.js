import {
  __path_join,
  __path_resolve,
  __path_dirname,
  __path_basename,
  __path_extname,
  __path_isAbsolute,
  __path_relative,
  __path_parse,
  __path_format,
  __path_sep,
  __path_delimiter,
} from "vole:raw:path";

export function join(...parts) {
  return __path_join(parts);
}

export function resolve(...parts) {
  return __path_resolve(parts);
}

export function dirname(path) {
  return __path_dirname(path);
}

export function basename(path, ext) {
  return __path_basename(path, ext);
}

export function extname(path) {
  return __path_extname(path);
}

export function isAbsolute(path) {
  return __path_isAbsolute(path);
}

export function relative(from, to) {
  return __path_relative(from, to);
}

export function parse(path) {
  return __path_parse(path);
}

export function format(pathObject) {
  return __path_format(pathObject);
}

export const sep = __path_sep;
export const delimiter = __path_delimiter;

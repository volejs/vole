import {
  __std_env_get,
  __std_env_set,
  __std_env_has,
  __std_env_remove,
  __std_env_entries,
  __std_env_load,
} from "vole:raw:env";

function assertEnvName(name) {
  if (typeof name !== "string") {
    throw new TypeError("env variable name must be a string");
  }
  if (name.length === 0 || name.includes("\0") || name.includes("=")) {
    throw new TypeError(
      "env variable name must be a non-empty string without '\\0' or '='",
    );
  }
}

function assertEnvValue(value) {
  if (typeof value !== "string") {
    throw new TypeError("env variable value must be a string");
  }
  if (value.includes("\0")) {
    throw new TypeError("env variable value must not contain NUL");
  }
}

function assertEnvFilePath(path) {
  if (typeof path !== "string" || path.length === 0) {
    throw new TypeError("env.load requires a file path");
  }
}

export function get(name) {
  assertEnvName(name);
  return __std_env_get(name);
}

export function set(name, value) {
  assertEnvName(name);
  assertEnvValue(value);
  return __std_env_set(name, value);
}

export function has(name) {
  assertEnvName(name);
  return __std_env_has(name);
}

export function remove(name) {
  assertEnvName(name);
  return __std_env_remove(name);
}

export function entries() {
  return __std_env_entries();
}

export function load(path) {
  assertEnvFilePath(path);
  return __std_env_load(path);
}

export const env = {
  get,
  set,
  has,
  remove,
  entries,
  load,
};

export default {
  get,
  set,
  has,
  remove,
  entries,
  load,
  env,
};

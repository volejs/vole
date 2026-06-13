import {
  __std_env_get,
  __std_env_set,
  __std_env_has,
  __std_env_remove,
  __std_env_entries,
} from "vole:raw:env";

export function get(name) {
  return __std_env_get(name);
}
export function set(name, value) {
  return __std_env_set(name, value);
}
export function has(name) {
  return __std_env_has(name);
}
export function remove(name) {
  return __std_env_remove(name);
}
export function entries() {
  return __std_env_entries();
}

export const env = {
  get(key) { return __std_env_get(key); },
  set(key, value) { return __std_env_set(key, value); },
  has(key) { return __std_env_has(key); },
  remove(key) { return __std_env_remove(key); },
  entries() { return __std_env_entries(); },
};

/**
 * Environment variable helpers exposed by `vole:env`.
 *
 * This module provides a small key/value API over the host process environment.
 * Values are always strings, matching platform environment variable semantics.
 *
 * @example
 * ```ts
 * import { get, set, has, entries } from "vole:env";
 *
 * set("VOLE_MODE", "dev");
 * if (has("VOLE_MODE")) {
 *   console.log(get("VOLE_MODE"));
 * }
 *
 * console.log(entries());
 * ```
 *
 * @remarks
 * Mutations affect the current vole process and children spawned after the
 * mutation. They do not persist to the user's shell or operating system profile.
 */
declare module "vole:env" {
  /**
   * Return the value for an environment variable.
   *
   * @param name Environment variable name.
   * @returns The string value, or `undefined` when the variable is absent.
   */
  export function get(name: string): string | undefined;

  /**
   * Set or replace an environment variable in the current process.
   *
   * @param name Environment variable name.
   * @param value Environment variable value. Values are stored as strings.
   */
  export function set(name: string, value: string): void;

  /**
   * Test whether an environment variable exists.
   *
   * @param name Environment variable name.
   * @returns `true` when the variable exists, even if its value is an empty string.
   */
  export function has(name: string): boolean;

  /**
   * Remove an environment variable from the current process.
   *
   * @param name Environment variable name.
   */
  export function remove(name: string): void;

  /**
   * Return a snapshot of all current environment variables.
   *
   * @returns A plain object mapping variable names to string values.
   */
  export function entries(): Record<string, string>;

  /**
   * Object-style facade for the same environment helpers.
   *
   * This is useful when passing the environment API around as a single value.
   */
  export const env: {
    /** See `get`. */
    get(name: string): string | undefined;
    /** See `set`. */
    set(name: string, value: string): void;
    /** See `has`. */
    has(name: string): boolean;
    /** See `remove`. */
    remove(name: string): void;
    /** See `entries`. */
    entries(): Record<string, string>;
  };
}

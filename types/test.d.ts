/**
 * Built-in test registration API exposed by `vole:test`.
 *
 * Test files register cases with `test` / `describe`, then the CLI calls
 * `run()` after all matching test files have been evaluated.
 *
 * @example
 * ```ts
 * import { test, describe, beforeEach } from "vole:test";
 *
 * describe("math", () => {
 *   let value = 0;
 *   beforeEach(() => { value = 1; });
 *
 *   test("increments", () => {
 *     if (value + 1 !== 2) throw new Error("bad math");
 *   });
 * });
 * ```
 *
 * @remarks
 * The runner is small and intentionally not Jest-compatible. Use explicit
 * assertions by throwing errors, or build a small assertion helper in your test
 * file.
 */
declare module "vole:test" {
  /** Test body callback. Async callbacks are awaited by the runner. */
  export type TestFn = () => void | Promise<void>;

  /** Object-form test case accepted by `test`. */
  export interface TestCase {
    /** Human-readable test name printed by the runner. */
    name: string;
    /** Test body. Throw or reject to fail the test. */
    fn: TestFn;
  }

  /** Register a test case by name and callback. */
  export function test(name: string, fn: TestFn): void;
  /** Register a test case using object form. */
  export function test(testCase: TestCase): void;
  /** Register a named test suite. */
  export function describe(name: string, fn: () => void): void;
  /** Alias for `test`. */
  export function it(name: string, fn: TestFn): void;
  /** Register a callback that runs once before tests in the current suite. */
  export function beforeAll(fn: TestFn): void;
  /** Register a callback that runs once after tests in the current suite. */
  export function afterAll(fn: TestFn): void;
  /** Register a callback that runs before each test in the current suite. */
  export function beforeEach(fn: TestFn): void;
  /** Register a callback that runs after each test in the current suite. */
  export function afterEach(fn: TestFn): void;
  /** Run all registered tests in the current VM. */
  export function run(): Promise<void>;

  /** Default object export containing all test registration helpers. */
  const testModule: {
    /** See `test`. */
    test: typeof test;
    /** See `describe`. */
    describe: typeof describe;
    /** See `it`. */
    it: typeof it;
    /** See `beforeAll`. */
    beforeAll: typeof beforeAll;
    /** See `afterAll`. */
    afterAll: typeof afterAll;
    /** See `beforeEach`. */
    beforeEach: typeof beforeEach;
    /** See `afterEach`. */
    afterEach: typeof afterEach;
    /** See `run`. */
    run: typeof run;
  };
  export default testModule;
}

// Adapted from AWS LLRT tests/unit/clone.test.ts
// Original license: Apache-2.0
// https://github.com/awslabs/llrt

import { test } from "std:test";

function assertEqual(actual, expected, message) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      message || `expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`
    );
  }
}

function assertNotSame(a, b, message) {
  if (a === b) {
    throw new Error(message || "expected values to not be the same reference");
  }
}

test("structuredClone clones a simple object", () => {
  const original = { foo: "bar", num: 42 };
  const cloned = structuredClone(original);
  assertEqual(cloned, original);
  original.foo += "extra";
  if (cloned.foo === original.foo) {
    throw new Error("clone was not deep");
  }
});

test("structuredClone clones an array", () => {
  const original = [1, 2, 3, 4, 5];
  const cloned = structuredClone(original);
  assertEqual(cloned, original);
});

test("structuredClone clones an array of objects", () => {
  const obj = { foo: "bar" };
  const original = [obj, obj, obj, obj, obj];
  const cloned = structuredClone(original);
  assertEqual(cloned, original);
  assertNotSame(cloned[0], original[0]);
});

test("structuredClone clones nested objects", () => {
  const original = { foo: { bar: { baz: "qux" } } };
  const cloned = structuredClone(original);
  assertEqual(cloned, original);
});

// The following cases are not yet supported by this runtime's structuredClone
// (current implementation is a serde_json round-trip without transfer list):
// - circular references
// - Map / Set
// - Date objects

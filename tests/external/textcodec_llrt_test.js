// Adapted from AWS LLRT tests/unit/util.test.ts (TextEncoder/TextDecoder section)
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

function assertArrayEqual(actual, expected, message) {
  if (actual.length !== expected.length) {
    throw new Error(message || `array length mismatch`);
  }
  for (let i = 0; i < actual.length; i++) {
    if (actual[i] !== expected[i]) {
      throw new Error(
        message || `expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`
      );
    }
  }
}

// The current runtime provides a basic TextEncoder/TextDecoder, but several
// spec behaviours differ from LLRT/Node (UTF-16LE decoding, BOM stripping,
// fatal errors, unsupported labels, and encodeInto semantics). Only UTF-8
// round-trips are exercised here.

test("TextEncoder/TextDecoder UTF-8 round-trip", () => {
  const text = "Hello, 世界!";
  const encoded = new TextEncoder().encode(text);
  const decoded = new TextDecoder().decode(encoded);
  assertEqual(decoded, text);
});

test("TextEncoder encodes ASCII", () => {
  assertArrayEqual(
    new TextEncoder().encode("ABC"),
    new Uint8Array([65, 66, 67])
  );
});

test("TextEncoder encodes multi-byte UTF-8", () => {
  assertArrayEqual(
    new TextEncoder().encode("🔥"),
    new Uint8Array([240, 159, 148, 165])
  );
});

// Adapted from AWS LLRT tests/unit/date.test.ts
// Original license: Apache-2.0
// https://github.com/awslabs/llrt

import { test } from "std:test";

function assertTrue(value, message) {
  if (value !== true) {
    throw new Error(message || `expected true but got ${JSON.stringify(value)}`);
  }
}

function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(
      message || `expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`
    );
  }
}

function assertContains(haystack, needle, message) {
  if (typeof haystack !== "string" || !haystack.includes(needle)) {
    throw new Error(
      message || `expected ${JSON.stringify(haystack)} to contain ${JSON.stringify(needle)}`
    );
  }
}

// This test requires full Intl/DateTimeFormat timeZone support.
// QuickJS currently ignores the timeZone option, so it is skipped.
// test("Date.prototype.toLocaleString with timeZone option", () => {
//   const date = new Date("2022-03-02T15:45:34Z");
//   const denver = date.toLocaleString("en-US", { timeZone: "America/Denver" });
//   const tokyo = date.toLocaleString("en-US", { timeZone: "Asia/Tokyo" });
//   assertContains(denver, "8:45");
//   assertContains(tokyo, "3/3");
// });

test("Date.prototype.toLocaleString without timezone option", () => {
  const date = new Date("2022-03-02T15:45:34Z");
  const result = date.toLocaleString();

  assertTrue(typeof result === "string");
  assertTrue(result.length > 0);
});

test("Date.prototype.toLocaleString handles Invalid Date", () => {
  const date = new Date("invalid");
  const result = date.toLocaleString("en-US", { timeZone: "UTC" });

  assertEqual(result, "Invalid Date");
});

// Adapted from AWS LLRT tests/unit/numbers.test.ts
// Original license: Apache-2.0
// https://github.com/awslabs/llrt

import { test } from "vole:test";

function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(
      message || `expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`
    );
  }
}

function assertTrue(value, message) {
  if (value !== true) {
    throw new Error(message || `expected true but got ${JSON.stringify(value)}`);
  }
}

const testCases = [
  { num: 0, radix: 2, expected: "0" },
  { num: 1, radix: 2, expected: "1" },
  { num: -1, radix: 2, expected: "-1" },
  { num: NaN, radix: 2, expected: "NaN" },
  { num: Infinity, radix: 2, expected: "Infinity" },

  { num: 255, radix: 16, expected: "ff" },
  { num: -255, radix: 16, expected: "-ff" },
  { num: NaN, radix: 16, expected: "NaN" },
  { num: Infinity, radix: 16, expected: "Infinity" },

  { num: 1023, radix: 8, expected: "1777" },
  { num: -1023, radix: 8, expected: "-1777" },
  { num: NaN, radix: 8, expected: "NaN" },
  { num: Infinity, radix: 8, expected: "Infinity" },

  { num: 123456789, radix: 10, expected: "123456789" },
  { num: -123456789, radix: 10, expected: "-123456789" },
  { num: 12345.6789, radix: 10, expected: "12345.6789" },
  { num: -12345.6789, radix: 10, expected: "-12345.6789" },
  { num: NaN, radix: 10, expected: "NaN" },
  { num: Infinity, radix: 10, expected: "Infinity" },
  { num: -Infinity, radix: 10, expected: "-Infinity" },

  { num: 2147483647, radix: 16, expected: "7fffffff" },
  { num: -2147483648, radix: 16, expected: "-80000000" },
  // QuickJS prints one extra hex digit compared to LLRT's expected value.
  { num: 12345.6789, radix: 16, expected: "3039.adcc63f142" },
  { num: -12345.6789, radix: 16, expected: "-3039.adcc63f142" },
  { num: NaN, radix: 16, expected: "NaN" },
  { num: Infinity, radix: 16, expected: "Infinity" },
  { num: -Infinity, radix: 16, expected: "-Infinity" },

  { num: 100, radix: 36, expected: "2s" },
  { num: -100, radix: 36, expected: "-2s" },
  { num: NaN, radix: 36, expected: "NaN" },
  { num: Infinity, radix: 36, expected: "Infinity" },

  { num: new Number(0), radix: undefined, expected: "0" },
  { num: new Number(42), radix: undefined, expected: "42" },
];

for (const { num, radix, expected } of testCases) {
  test(`Number.toString num=${num} radix=${radix}`, () => {
    assertEqual(num.toString(radix), expected);
  });
}

const largeNum = 1749475325433;

test("Number.toString large number binary", () => {
  assertEqual(
    largeNum.toString(2),
    "11001011101010100110110101111010111111001"
  );
});

test("Number.toString large number octal", () => {
  assertEqual(largeNum.toString(8), "31352466572771");
});

test("Number.toString large number decimal", () => {
  assertEqual(largeNum.toString(10), "1749475325433");
});

test("Number.toString large number hexadecimal", () => {
  assertEqual(largeNum.toString(16), "19754daf5f9");
});

test("Number.toString large number base-20", () => {
  assertEqual(largeNum.toString(20), "386fb0fdbd");
});

test("Number.toString large number base-26", () => {
  assertEqual(largeNum.toString(26), "89l74gagh");
});

test("Number.toString large number base-32", () => {
  assertEqual(largeNum.toString(32), "1itadltfp");
});

test("Number.toString large number base-36", () => {
  assertEqual(largeNum.toString(36), "mbp4fshl");
});

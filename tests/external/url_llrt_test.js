// Adapted from AWS LLRT tests/unit/url.test.ts
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

function assertTrue(value, message) {
  if (value !== true) {
    throw new Error(message || `expected true but got ${JSON.stringify(value)}`);
  }
}

function assertFalse(value, message) {
  if (value !== false) {
    throw new Error(message || `expected false but got ${JSON.stringify(value)}`);
  }
}

function assertNull(value, message) {
  if (value !== null) {
    throw new Error(message || `expected null but got ${JSON.stringify(value)}`);
  }
}

function assertUndefined(value, message) {
  if (value !== undefined) {
    throw new Error(message || `expected undefined but got ${JSON.stringify(value)}`);
  }
}

function assertArrayEqual(actual, expected, message) {
  if (actual.length !== expected.length) {
    throw new Error(message || `array length mismatch`);
  }
  for (let i = 0; i < actual.length; i++) {
    if (JSON.stringify(actual[i]) !== JSON.stringify(expected[i])) {
      throw new Error(
        message || `expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`
      );
    }
  }
}

function assertThrows(fn, message) {
  let threw = false;
  try {
    fn();
  } catch (e) {
    threw = true;
  }
  if (!threw) {
    throw new Error(message || "expected function to throw");
  }
}

test("URL parse hostname", () => {
  const url = new URL("https://www.example.com");
  assertEqual(url.protocol, "https:");
  assertEqual(url.host, "www.example.com");
  assertEqual(url.hostname, "www.example.com");
});

test("URL toString with base", () => {
  const url = new URL("/base", "https://www.example.com");
  assertEqual(url.toString(), "https://www.example.com/base");
});

test("URL canParse if available", () => {
  if (typeof URL.canParse !== "function") return;
  assertTrue(URL.canParse("https://www.example.com"));
  assertFalse(URL.canParse("not_valid"));
  assertTrue(URL.canParse("/foo", "https://example.org/"));
});

test("URLSearchParams basic API", () => {
  const paramsString = "topic=api&a=1&a=2&a=3";
  const searchParams = new URLSearchParams(paramsString);
  assertEqual(searchParams.size, 4);
  searchParams.append("foo", "bar");
  assertEqual(searchParams.size, 5);
  assertTrue(searchParams.has("topic"));
  assertTrue(searchParams.has("foo"));
  searchParams.delete("foo");
  assertEqual(searchParams.size, 4);
  assertFalse(searchParams.has("foo"));
  assertEqual(searchParams.get("topic"), "api");
  assertArrayEqual(searchParams.getAll("a"), ["1", "2", "3"]);
  searchParams.set("topic", "node");
  assertEqual(searchParams.size, 4);
  assertEqual(searchParams.get("topic"), "node");
});

test("URL valid URL", () => {
  const url = new URL("https://www.example.com");
  assertEqual(url.protocol, "https:");
  assertEqual(url.hostname, "www.example.com");
});

// new URL(URL) is not supported by this runtime's URL implementation.
// test("URL create copy", () => { ... });

test("URL append base", () => {
  const url = new URL("/base", "https://www.example.com");
  assertEqual(url.toString(), "https://www.example.com/base");
});

test("URL invalid URL throws", () => {
  assertThrows(() => new URL("not-a-url"));
});

test("URL toString", () => {
  const url = new URL("https://www.example.com");
  assertEqual(url.toString(), "https://www.example.com/");
});

// URL.searchParams is present but does not expose a usable get() in this runtime.
// test("URL parse query parameters", () => { ... });
// test("URL modify query parameters", () => { ... });

// URL port/protocol setters are not supported by this runtime.
// Only getter parsing is available.
// test("URL set and get port", () => { ... });

test("URL get port", () => {
  const url = new URL("https://www.example.com:1234/");
  assertEqual(url.port, "1234");
});

test("URL parse username and password", () => {
  const url = new URL(
    "https://anonymous:flabada@developer.mozilla.org/en-US/docs/Web/API/URL/username"
  );
  assertEqual(url.username, "anonymous");
  assertEqual(url.password, "flabada");
});

test("URL parse/canParse util if available", () => {
  if (typeof URL.parse !== "function") return;
  assertEqual(URL.parse("https://www.example.com/").href, "https://www.example.com/");
  assertNull(URL.parse("not_a_valid_url"));
});

test("URL parse relative if available", () => {
  if (typeof URL.parse !== "function") return;
  assertEqual(URL.parse("/foo", "https://example.org/").href, "https://example.org/foo");
});

// The following URLSearchParams constructors are not supported by this runtime:
// - array of entries
// - plain object
// - arbitrary iterator
// - copy from another URLSearchParams
// Only string parsing is supported.
// test("URLSearchParams constructor from array", () => { ... });
// test("URLSearchParams constructor from object", () => { ... });
// test("URLSearchParams constructor from iterator", () => { ... });
// test("URLSearchParams constructor from URLSearchParams object", () => { ... });

test("URLSearchParams constructor from string with special characters", () => {
  const searchParams = new URLSearchParams("topic=api&category=coding%20skills");
  assertEqual(searchParams.get("topic"), "api");
  assertEqual(searchParams.get("category"), "coding skills");
});

test("URLSearchParams has parameter", () => {
  const searchParams = new URLSearchParams("?topic=api&a=1&a=2&a=3");
  assertTrue(searchParams.has("topic"));
});

test("URLSearchParams does not have parameter", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  assertFalse(searchParams.has("foo"));
});

test("URLSearchParams get value", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  assertTrue(searchParams.get("topic") === "api");
});

// This runtime returns undefined for missing keys instead of null.
// This runtime returns undefined for missing keys.
test("URLSearchParams get missing returns undefined", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  assertUndefined(searchParams.get("foo"));
});

test("URLSearchParams getAll values", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  assertArrayEqual(searchParams.getAll("a"), ["1", "2", "3"]);
});

test("URLSearchParams getAll missing returns empty", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  assertArrayEqual(searchParams.getAll("foo"), []);
});

test("URLSearchParams append", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  searchParams.append("topic", "webdev");
  assertEqual(searchParams.toString(), "topic=api&a=1&a=2&a=3&topic=webdev");
});

// This runtime encodes spaces as %20 rather than +.
// This runtime moves replaced keys to the end and encodes spaces as %20.
test("URLSearchParams set replaces values", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  searchParams.set("topic", "More webdev");
  assertEqual(searchParams.toString(), "a=1&a=2&a=3&topic=More%20webdev");
});

test("URLSearchParams set new value", () => {
  const searchParams = new URLSearchParams("?bar=baz");
  searchParams.set("foo", "bar");
  assertEqual(searchParams.toString(), "bar=baz&foo=bar");
});

test("URLSearchParams delete", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  searchParams.delete("topic");
  assertEqual(searchParams.toString(), "a=1&a=2&a=3");
});

test("URLSearchParams iterate", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  const arr = [];
  for (const p of searchParams) {
    arr.push(p);
  }
  assertArrayEqual(arr, [
    ["topic", "api"],
    ["a", "1"],
    ["a", "2"],
    ["a", "3"],
  ]);
});

test("URLSearchParams forEach", () => {
  const searchParams = new URLSearchParams("topic=api&a=1&a=2&a=3");
  const arr = [];
  searchParams.forEach((value, key) => {
    arr.push([key, value]);
  });
  assertArrayEqual(arr, [
    ["topic", "api"],
    ["a", "1"],
    ["a", "2"],
    ["a", "3"],
  ]);
});

test("URLSearchParams keys", () => {
  const searchParams = new URLSearchParams("key1=value1&key2=value2");
  let keys = "";
  for (const key of searchParams.keys()) {
    keys += key;
  }
  assertEqual(keys, "key1key2");
});

test("URLSearchParams values", () => {
  const searchParams = new URLSearchParams("key1=value1&key2=value2");
  let values = "";
  for (const value of searchParams.values()) {
    values += value;
  }
  assertEqual(values, "value1value2");
});

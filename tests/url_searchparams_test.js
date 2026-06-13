import { test } from "vole:test";

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

test("URL.searchParams returns a URLSearchParams object", () => {
  const url = new URL("https://example.com/?foo=bar&baz=qux");
  const params = url.searchParams;
  assertTrue(params instanceof URLSearchParams, "searchParams should be a URLSearchParams instance");
  assertEqual(params.get("foo"), "bar");
  assertEqual(params.get("baz"), "qux");
});

test("URL.searchParams for URL without query", () => {
  const url = new URL("https://example.com/path");
  const params = url.searchParams;
  assertTrue(params instanceof URLSearchParams);
  assertEqual(params.toString(), "");
  assertEqual(params.size, 0);
});

test("URL.searchParams reflects encoded query", () => {
  const url = new URL("https://example.com/?name=hello%20world&a=1&a=2");
  const params = url.searchParams;
  assertEqual(params.get("name"), "hello world");
  assertEqual(params.getAll("a"), ["1", "2"]);
});

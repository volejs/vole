import { test } from "vole:test";

const assert = {
  equal(actual, expected) {
    if (actual !== expected) {
      throw new Error(`expected ${expected}, got ${actual}`);
    }
  },
};

test("URL.canParse", () => {
  assert.equal(URL.canParse("http://example.com"), true);
  assert.equal(URL.canParse("not a url"), false);
  assert.equal(URL.canParse("/path", "http://example.com"), true);
  assert.equal(URL.canParse("/path"), false);
});

test("URL.parse", () => {
  const absolute = URL.parse("http://example.com");
  assert.equal(absolute instanceof URL, true);
  assert.equal(absolute.href, "http://example.com/");

  const relative = URL.parse("/path", "http://example.com");
  assert.equal(relative instanceof URL, true);
  assert.equal(relative.href, "http://example.com/path");

  const invalid = URL.parse("not a url");
  assert.equal(invalid, null);
});

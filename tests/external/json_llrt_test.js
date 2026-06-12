// Adapted from AWS LLRT tests/unit/json.test.ts
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

test("JSON.parse valid JSON", () => {
  assertEqual(JSON.parse('{"key": "value"}'), { key: "value" });
});

test("JSON.parse invalid JSON throws", () => {
  assertThrows(() => JSON.parse('{key: "value"}'));
  assertThrows(() => JSON.parse(""));
});

test("JSON.parse nested structures", () => {
  assertEqual(
    JSON.parse('{"name": "John", "age": 25, "address": {"city": "New York", "zip": "10001"}}'),
    { name: "John", age: 25, address: { city: "New York", zip: "10001" } }
  );
});

test("JSON.parse arrays", () => {
  assertEqual(JSON.parse('[1, 2, 3, {"key": "value"}]'), [1, 2, 3, { key: "value" }]);
});

test("JSON.parse booleans", () => {
  assertEqual(JSON.parse('{"isTrue": true, "isFalse": false}'), { isTrue: true, isFalse: false });
});

test("JSON.parse null", () => {
  assertEqual(JSON.parse('{"nullableValue": null}'), { nullableValue: null });
});

test("JSON.parse large int", () => {
  // QuickJS stores this as a double, so the parsed value is rounded.
  assertEqual(JSON.parse('{"bigInt": 888888888888888888}'), { bigInt: 888888888888888800 });
});

test("JSON.parse special characters", () => {
  const specialChars = "!@#$%^&*()_+-={}[]|;:,.<>?/";
  assertEqual(
    JSON.parse(`{"specialChars": "${specialChars}"}`),
    { specialChars }
  );
});

test("JSON.stringify round-trip", () => {
  const data = { key: "value", age: 25 };
  assertEqual(JSON.parse(JSON.stringify(data)), data);
});

test("JSON.stringify toJSON method", () => {
  const obj = {
    key: "value",
    age: 25,
    toJSON() {
      return { customKey: this.key.toUpperCase(), customAge: this.age * 2 };
    },
  };
  assertEqual(JSON.parse(JSON.stringify(obj)), { customKey: "VALUE", customAge: 50 });
});

test("JSON.stringify integer floats", () => {
  assertEqual(JSON.stringify({ value: 1.0 }), '{"value":1}');
});

test("JSON.stringify very large numbers scientific", () => {
  // QuickJS uses "1e+30" for scientific notation.
  assertEqual(
    JSON.stringify({ value: 999999999999999999999999999999 }),
    '{"value":1e+30}'
  );
});

test("JSON.stringify self-reference throws", () => {
  const recursiveData = { key: "value", nested: { age: 25, inner: null } };
  recursiveData.nested.inner = recursiveData;
  assertThrows(() => JSON.stringify(recursiveData));
});

test("JSON.stringify default spacing", () => {
  const data = {
    key: "value",
    bool: true,
    num: 42,
    arr: [1, 2, 3],
    nested: { level1: { level2: { level3: "nestedValue" } } },
  };
  const expected = `{
    "key": "value",
    "bool": true,
    "num": 42,
    "arr": [
        1,
        2,
        3
    ],
    "nested": {
        "level1": {
            "level2": {
                "level3": "nestedValue"
            }
        }
    }
}`;
  assertEqual(JSON.stringify(data, null, 4), expected);
});

test("JSON.stringify custom string spacing", () => {
  const data = {
    key: "value",
    bool: false,
    num: 3.14,
    arr: ["apple", "banana", "cherry"],
    nested: { level1: { level2: { level3: "nestedValue" } } },
  };
  const expected = `{
   "key": "value",
   "bool": false,
   "num": 3.14,
   "arr": [
      "apple",
      "banana",
      "cherry"
   ],
   "nested": {
      "level1": {
         "level2": {
            "level3": "nestedValue"
         }
      }
   }
}`;
  assertEqual(JSON.stringify(data, null, "   "), expected);
});

test("JSON.stringify replacer function", () => {
  const data = { key: "value", secret: "hidden" };
  const replacer = (key, value) => (key === "secret" ? undefined : value);
  assertEqual(JSON.stringify(data, replacer, 2), '{\n  "key": "value"\n}');
});

test("JSON.stringify complex with replacer", () => {
  const date = new Date();
  class Foo {
    constructor() {
      this.prop = 1;
    }
  }
  const complexData = {
    key: "value",
    date,
    nested: {
      array: [1, 2, 3],
      obj: { a: "apple", b: "banana" },
      foo: new Foo(),
      arrowFn: () => { },
      fn: function () { },
      namedFn: function namedFn() { },
    },
  };
  const replacer = (key, value) =>
    typeof value === "string" ? value.toUpperCase() : value;
  const expected = `{
    "key": "VALUE",
    "date": "${date.toJSON()}",
    "nested": {
        "array": [
            1,
            2,
            3
        ],
        "obj": {
            "a": "APPLE",
            "b": "BANANA"
        },
        "foo": {
            "prop": 1
        }
    }
}`;
  assertEqual(JSON.stringify(complexData, replacer, 4), expected);
});

test("JSON.stringify object undefined values", () => {
  assertEqual(JSON.stringify({ a: undefined, b: "123", c: "123" }), '{"b":"123","c":"123"}');
  const expectedStart = `{
   "b": "123",
   "c": "123"
}`;
  assertEqual(JSON.stringify({ a: undefined, b: "123", c: "123" }, null, "   "), expectedStart);

  assertEqual(JSON.stringify({ a: "123", b: undefined, c: "123" }), '{"a":"123","c":"123"}');
  assertEqual(JSON.stringify({ a: "123", b: "123", c: undefined }), '{"a":"123","b":"123"}');

  const valueMixed = JSON.stringify({
    a: "123",
    b: undefined,
    c: undefined,
    d: "123",
    e: undefined,
    f: "123",
    g: "123",
    h: undefined,
    i: undefined,
  });
  assertEqual(valueMixed, '{"a":"123","d":"123","f":"123","g":"123"}');
});

test("JSON.stringify array undefined values", () => {
  assertEqual(JSON.stringify([undefined, "123", "123"]), '[null,"123","123"]');
  assertEqual(JSON.stringify(["123", undefined, "123"]), '["123",null,"123"]');
  assertEqual(JSON.stringify(["123", "123", undefined]), '["123","123",null]');
});

test("JSON.stringify removes non-JSON values", () => {
  const now = new Date();
  const dateString = now.toJSON();
  const data = {
    a: "123",
    b: undefined,
    c: () => "123",
    d: RegExp("apa"),
    e: now,
  };
  assertEqual(JSON.stringify(data), `{"a":"123","d":{},"e":"${dateString}"}`);
});

test("JSON.stringify Error", () => {
  assertEqual(JSON.stringify(new Error("error")), '{}');
});

test("JSON.stringify BigInt throws", () => {
  assertThrows(() => JSON.stringify({ v: 1n }));
});

test("JSON.stringify replacer returns new non-primitive objects", () => {
  const json = JSON.stringify({ key: "value" });
  const obj = { simple: "text", nested: json };
  const replacer = (key, value) => {
    try {
      return typeof value === "string" ? JSON.parse(value) : value;
    } catch {
      return value;
    }
  };
  assertEqual(JSON.stringify(obj, replacer), '{"simple":"text","nested":{"key":"value"}}');
});

test("JSON.stringify escapes broken surrogate pairs", () => {
  const text =
    "[A-Za-z\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u02b8\u0300-\u0590\u0900-\u1fff\u200e\u2c00-\ud801\ud804-\ud839\ud83c-\udbff\uf900-\ufb1c\ufe00-\ufe6f\ufefd-\uffff]";
  assertEqual(JSON.parse(JSON.stringify(text)), text);
});

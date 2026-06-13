import { test } from "vole:test";
import { open, close, call, define } from "vole:ffi";

const LIB_PATH = "tests/fixtures/libffi_test.dylib";

function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(
      message || `expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`
    );
  }
}

test("vole:ffi open and close", () => {
  const handle = open(LIB_PATH);
  if (typeof handle !== "bigint") {
    throw new Error("expected handle to be bigint");
  }
  close(handle);
});

test("vole:ffi call integer function", () => {
  const handle = open(LIB_PATH);
  try {
    const result = call(
      handle,
      "add_ints",
      [2, 3],
      { parameters: ["i32", "i32"], result: "i32" }
    );
    assertEqual(result, 5);
  } finally {
    close(handle);
  }
});

test("vole:ffi call double function", () => {
  const handle = open(LIB_PATH);
  try {
    const result = call(
      handle,
      "add_doubles",
      [2.5, 3.5],
      { parameters: ["f64", "f64"], result: "f64" }
    );
    assertEqual(result, 6.0);
  } finally {
    close(handle);
  }
});

test("vole:ffi call string length function", () => {
  const handle = open(LIB_PATH);
  try {
    const result = call(
      handle,
      "strlen_c",
      ["hello"],
      { parameters: ["string"], result: "i32" }
    );
    assertEqual(result, 5);
  } finally {
    close(handle);
  }
});

test("vole:ffi define reusable function", () => {
  const handle = open(LIB_PATH);
  try {
    const add = define(handle, "add_ints", {
      parameters: ["i32", "i32"],
      result: "i32",
    });
    assertEqual(add(10, 20), 30);
    assertEqual(add(-5, 5), 0);
  } finally {
    close(handle);
  }
});

test("vole:ffi call returns string", () => {
  const handle = open(LIB_PATH);
  try {
    const result = call(
      handle,
      "greet",
      ["FFI"],
      { parameters: ["string"], result: "string" }
    );
    assertEqual(result, "Hello, FFI!");
  } finally {
    close(handle);
  }
});

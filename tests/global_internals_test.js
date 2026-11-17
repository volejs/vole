import { test } from "vole:test";

test("runtime does not expose internal bindings on globalThis", () => {
  const internals = Object.getOwnPropertyNames(globalThis)
    .filter((name) =>
      name.startsWith("__") || name.startsWith("_std") || name.startsWith("_ffi")
    )
    .sort();

  if (internals.length !== 0) {
    throw new Error("Unexpected internal global keys: " + JSON.stringify(internals));
  }
});

# External / Vendor Tests

This directory contains compatibility tests adapted from other JavaScript
runtimes and standard test suites.

## Sources

| Source                                                                | License      | Notes                                        |
| --------------------------------------------------------------------- | ------------ | -------------------------------------------- |
| [AWS LLRT](https://github.com/awslabs/llrt)                           | Apache-2.0   | Rust + QuickJS runtime, closest to our stack |
| [Web Platform Tests (WPT)](https://github.com/web-platform-tests/wpt) | BSD-3-Clause | Browser / WinterTC Web API conformance       |
| [Deno std](https://github.com/denoland/std)                           | MIT          | Deno standard library tests                  |
| [Node.js](https://github.com/nodejs/node)                             | MIT          | Node.js built-in tests                       |

## Adapting Tests

Most upstream tests use framework-specific APIs (`expect`, `node:assert`,
`Deno.test`, etc.). When bringing them in, convert them to the project's
`std:test` API:

```js
import { test } from "std:test";

test("description", () => {
  if (actual !== expected) {
    throw new Error("assertion failed");
  }
});
```

## Current Adapted Tests

| File                           | Source                            | Status |
| ------------------------------ | --------------------------------- | ------ |
| `structuredClone_llrt_test.js` | LLRT `tests/unit/clone.test.ts`   | ✅     |
| `numbers_llrt_test.js`         | LLRT `tests/unit/numbers.test.ts` | ✅     |
| `date_llrt_test.js`            | LLRT `tests/unit/date.test.ts`    | ✅     |
| `json_llrt_test.js`            | LLRT `tests/unit/json.test.ts`    | ✅     |
| `url_llrt_test.js`             | LLRT `tests/unit/url.test.ts`     | ✅     |
| `timers_llrt_test.js`          | LLRT `tests/unit/timers.test.ts`  | ✅     |
| `textcodec_llrt_test.js`       | LLRT `tests/unit/util.test.ts`    | ✅     |

Run them with:

```sh
./target/debug/vole test tests/external
```

## Known Runtime Differences

The following upstream behaviours are not supported by the current runtime and
are therefore skipped or adjusted in the adapted files:

- **structuredClone**: only JSON-serializable objects are cloned; circular
  references, `Map`, `Set`, and `Date` objects are not preserved.
- **Date.prototype.toLocaleString**: the `timeZone` option is ignored.
- **Number.prototype.toString(radix)**: floating-point hexadecimal output uses
  one extra digit than LLRT expects (`3039.adcc63f142` vs `3039.adcc63f14`).
- **JSON.parse/stringify**: very large integers are rounded to double precision;
  scientific notation is emitted as `1e+30` instead of `1e30`.
- **URL**: `URL` objects cannot be used as constructor arguments; `port` and
  `protocol` setters are unavailable; `url.searchParams` does not expose a
  `get()` method.
- **URLSearchParams**: only string constructors are supported; missing keys
  return `undefined`; `set()` moves replaced keys to the end and encodes spaces
  as `%20`.
- **Timers**: `clearTimeout`/`clearInterval` require numeric handles; negative
  delays are rejected.
- **TextEncoder/TextDecoder**: only basic UTF-8 round-trips are reliable;
  UTF-16LE decoding, BOM stripping, fatal mode, unsupported-label errors, and
  `encodeInto` semantics differ from the spec.

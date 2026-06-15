# WPT adapter

`third_party/wpt` is a git submodule pointing at Web Platform Tests. The files in this directory are vole-specific adapters and suite definitions; do not edit upstream WPT files in place.

## Scope

This adapter intentionally starts with self-contained tests only. Browser/server-backed tests that require the WPT HTTP server, DOM, navigation, XMLHttpRequest, service workers, or browser security state should stay out of the allowlist until vole has a dedicated adapter for that area.

Current suites:

```sh
vole test tests/wpt/suites/fetch-headers_test.js
vole test tests/wpt/suites/fetch-request-response_test.js
vole test tests/wpt/suites/url_test.js
```

## Files

- `harness.js` provides a small `testharness.js` compatibility layer on top of `vole:test`.
- `runner.js` loads WPT files listed by a suite and registers their tests with `vole:test`.
- `suites/*.txt` are allowlists of upstream WPT files, relative to `third_party/wpt`.
- `expected-failures.txt` lists known failures without modifying upstream WPT files.

## Excluded tests

Do not add browser-only tests to stable suites. This includes tests that need
`document`, `iframe`, navigation, service workers, XMLHttpRequest, or the WPT
HTTP server.

Some fetch stream tests currently pass their JS assertions but trigger a
QuickJS GC assertion during runtime shutdown. Keep those commented in the suite
with a reason until the underlying lifetime/trace issue is fixed. These are not
represented as expected failures because the process abort happens after the
test has already passed.

## Expected failures

Use one exact test per line:

```txt
fetch/api/headers/headers-basic.any.js: Create headers with null should throw
```

Or mark a whole file as expected failure:

```txt
fetch/api/headers/headers-record.any.js: *
```

An expected failure passes only if it still fails. If it unexpectedly passes, the adapter reports that so the xfail entry can be removed.

## Submodule update

```sh
git submodule update --init --depth 1 third_party/wpt
```

When updating WPT, update the suite allowlists and expected failures in the same change.

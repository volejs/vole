# vole-types

TypeScript declaration files for the `vole` JavaScript runtime.

Install the package in projects that run on `vole`:

```sh
npm install -D vole-types
```

Then reference it from `tsconfig.json`:

```json
{
  "compilerOptions": {
    "types": ["vole-types"],
    "lib": ["ES2022", "DOM", "DOM.Iterable"]
  }
}
```

These declarations describe public `vole:*` modules and small vole-specific
global additions. Standard Web APIs are provided through TypeScript's DOM libs.

# Benchmarks

Pure JavaScript micro-benchmarks used to measure runtime performance.
These benchmarks do not depend on Web APIs or `vole:*` modules and can be
compared across JavaScript runtimes (Node.js, Deno, Bun, LLRT, etc.).

## Running

```sh
# Run a single benchmark
vole run benchmarks/fibonacci.js

# Run all benchmarks
for f in benchmarks/*.js; do echo "=== $f ==="; vole run "$f"; done
```

## Benchmarks

| File              | Description                          |
| ----------------- | ------------------------------------ |
| `fibonacci.js`    | Recursive Fibonacci                  |
| `nqueens.js`      | N-Queens backtracking solver         |
| `primes.js`       | Sieve of Eratosthenes                |
| `json.js`         | JSON parse/stringify round-trip      |
| `arraysort.js`    | Numeric array sort                   |
| `strings.js`      | String split/reverse/join/replace    |
| `regex.js`        | Regular expression match/replace     |
| `objects.js`      | Object creation and property access  |
| `mapset.js`       | Map/Set insertion and lookup         |
| `classes.js`      | Class instantiation and method calls |
| `arrayiterate.js` | Array iteration styles               |

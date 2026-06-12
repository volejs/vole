// Object creation and property access benchmark
const ITERATIONS = 5_000_000;

let total = 0;
const start = performance.now();
for (let i = 0; i < ITERATIONS; i++) {
  const obj = { a: i, b: i + 1, c: i + 2 };
  obj.d = obj.a + obj.b + obj.c;
  total += obj.d;
}
const end = performance.now();

console.log(`object create/access x${ITERATIONS}: ${(end - start).toFixed(2)}ms`);
console.log(`checksum: ${total}`);

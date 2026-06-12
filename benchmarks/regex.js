// Regular expression benchmark
const ITERATIONS = 500_000;
const text = "The quick brown fox jumps over the lazy dog. ";
const pattern = /\b\w{4}\b/g;

let count = 0;
const start = performance.now();
for (let i = 0; i < ITERATIONS; i++) {
  const matches = text.match(pattern);
  count += matches ? matches.length : 0;
  const replaced = text.replace(/fox/g, "cat").replace(/dog/g, "wolf");
}
const end = performance.now();

console.log(`regex match/replace x${ITERATIONS}: ${(end - start).toFixed(2)}ms`);
console.log(`matches found: ${count}`);

// String manipulation benchmark
const ITERATIONS = 100_000;
let result = "";

const start = performance.now();
for (let i = 0; i < ITERATIONS; i++) {
  result = "hello".repeat(10)
    .split("")
    .reverse()
    .join("")
    .replace(/l/g, "L")
    .toUpperCase();
}
const end = performance.now();

console.log(`string ops x${ITERATIONS}: ${(end - start).toFixed(2)}ms`);
console.log(`result: ${result.slice(0, 20)}...`);

// Classic recursive Fibonacci benchmark
function fib(n) {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2);
}

const N = 35;
const start = performance.now();
const result = fib(N);
const end = performance.now();

console.log(`fib(${N}) = ${result}`);
console.log(`time: ${(end - start).toFixed(2)}ms`);

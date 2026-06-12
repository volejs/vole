// Array sort benchmark
function randomArray(size) {
  const arr = new Array(size);
  for (let i = 0; i < size; i++) {
    arr[i] = Math.random();
  }
  return arr;
}

const SIZE = 100_000;
const ITERATIONS = 100;
let total = 0;

for (let i = 0; i < ITERATIONS; i++) {
  const arr = randomArray(SIZE);
  const start = performance.now();
  arr.sort((a, b) => a - b);
  const end = performance.now();
  total += end - start;
}

console.log(`array sort ${SIZE} elements x${ITERATIONS}: avg ${(total / ITERATIONS).toFixed(2)}ms`);

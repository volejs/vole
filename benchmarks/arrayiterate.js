// Array iteration benchmark
const SIZE = 1_000_000;
const arr = new Array(SIZE);
for (let i = 0; i < SIZE; i++) {
  arr[i] = i;
}

let total = 0;
let start = performance.now();
for (let i = 0; i < SIZE; i++) {
  total += arr[i];
}
let end = performance.now();
console.log(`for loop x${SIZE}: ${(end - start).toFixed(2)}ms`);

total = 0;
start = performance.now();
arr.forEach((v) => {
  total += v;
});
end = performance.now();
console.log(`forEach x${SIZE}: ${(end - start).toFixed(2)}ms`);

total = 0;
start = performance.now();
total = arr.reduce((acc, v) => acc + v, 0);
end = performance.now();
console.log(`reduce x${SIZE}: ${(end - start).toFixed(2)}ms`);

console.log(`sum: ${total}`);

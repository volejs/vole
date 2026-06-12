// Map and Set operations benchmark
const SIZE = 100_000;

const map = new Map();
const set = new Set();

let start = performance.now();
for (let i = 0; i < SIZE; i++) {
  map.set(`key-${i}`, i);
  set.add(i);
}
let end = performance.now();
console.log(`Map/Set insert x${SIZE}: ${(end - start).toFixed(2)}ms`);

let found = 0;
start = performance.now();
for (let i = 0; i < SIZE; i++) {
  if (map.has(`key-${i}`) && set.has(i)) {
    found++;
  }
}
end = performance.now();
console.log(`Map/Set lookup x${SIZE}: ${(end - start).toFixed(2)}ms`);
console.log(`found: ${found}`);

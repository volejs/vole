// JSON parse/stringify benchmark
const payload = {
  users: Array.from({ length: 1000 }, (_, i) => ({
    id: i,
    name: `user-${i}`,
    email: `user${i}@example.com`,
    active: i % 2 === 0,
    tags: ["a", "b", "c"],
  })),
};

const text = JSON.stringify(payload);
const iterations = 1000;

let start = performance.now();
for (let i = 0; i < iterations; i++) {
  JSON.parse(text);
}
let end = performance.now();
console.log(`JSON.parse x${iterations}: ${(end - start).toFixed(2)}ms`);

const obj = JSON.parse(text);
start = performance.now();
for (let i = 0; i < iterations; i++) {
  JSON.stringify(obj);
}
end = performance.now();
console.log(`JSON.stringify x${iterations}: ${(end - start).toFixed(2)}ms`);

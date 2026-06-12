// Class instantiation and method call benchmark
const ITERATIONS = 5_000_000;

class Point {
  constructor(x, y) {
    this.x = x;
    this.y = y;
  }

  distance(other) {
    const dx = this.x - other.x;
    const dy = this.y - other.y;
    return Math.sqrt(dx * dx + dy * dy);
  }
}

let total = 0;
const start = performance.now();
for (let i = 0; i < ITERATIONS; i++) {
  const a = new Point(i, i + 1);
  const b = new Point(i + 2, i + 3);
  total += a.distance(b);
}
const end = performance.now();

console.log(`class instantiate/call x${ITERATIONS}: ${(end - start).toFixed(2)}ms`);
console.log(`distance sum: ${total.toFixed(2)}`);

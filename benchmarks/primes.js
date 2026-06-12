// Sieve of Eratosthenes benchmark
function sieve(limit) {
  const isPrime = new Array(limit + 1).fill(true);
  isPrime[0] = isPrime[1] = false;
  for (let i = 2; i * i <= limit; i++) {
    if (isPrime[i]) {
      for (let j = i * i; j <= limit; j += i) {
        isPrime[j] = false;
      }
    }
  }
  let count = 0;
  for (let i = 2; i <= limit; i++) {
    if (isPrime[i]) count++;
  }
  return count;
}

const LIMIT = 10_000_000;
const start = performance.now();
const result = sieve(LIMIT);
const end = performance.now();

console.log(`primes up to ${LIMIT} = ${result}`);
console.log(`time: ${(end - start).toFixed(2)}ms`);

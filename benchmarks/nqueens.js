// N-Queens solver benchmark
function solveNQueens(n) {
  const cols = new Array(n).fill(false);
  const diag1 = new Array(2 * n - 1).fill(false);
  const diag2 = new Array(2 * n - 1).fill(false);
  let count = 0;

  function place(row) {
    if (row === n) {
      count++;
      return;
    }
    for (let col = 0; col < n; col++) {
      const d1 = row + col;
      const d2 = row - col + n - 1;
      if (cols[col] || diag1[d1] || diag2[d2]) continue;
      cols[col] = diag1[d1] = diag2[d2] = true;
      place(row + 1);
      cols[col] = diag1[d1] = diag2[d2] = false;
    }
  }

  place(0);
  return count;
}

const N = 12;
const start = performance.now();
const result = solveNQueens(N);
const end = performance.now();

console.log(`nqueens(${N}) = ${result}`);
console.log(`time: ${(end - start).toFixed(2)}ms`);

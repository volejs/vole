let count = 0;
const id = setInterval(() => {
  count++;
  console.log("interval", count);
  if (count >= 3) {
    clearInterval(id);
    console.log("done");
  }
}, 30);

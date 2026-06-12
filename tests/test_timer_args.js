// Test setTimeout with args
setTimeout((a, b) => {
    console.log("timeout args:", a, b);
}, 10, "hello", 42);

// Test setInterval with args
let count = 0;
const id = setInterval((msg) => {
    count++;
    console.log("interval arg:", msg, count);
    if (count >= 2) {
        clearInterval(id);
        console.log("done");
    }
}, 20, "tick");

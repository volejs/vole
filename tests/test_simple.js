console.log("=== console ===");
console.log("hello from console.log");
console.info("info test");
console.warn("warn test");
console.error("error test");
console.debug("debug test");

console.log("=== performance ===");
const t0 = performance.now();
for (let i = 0; i < 1000000; i++) { }
const t1 = performance.now();
console.log("elapsed:", (t1 - t0).toFixed(3), "ms");

console.log("=== timers ===");
const id = setTimeout(() => console.log("timeout fired"), 100);
console.log("timeout id:", id);
clearTimeout(id);

const intervalId = setInterval(() => console.log("interval fired"), 100);
console.log("interval id:", intervalId);
clearInterval(intervalId);

queueMicrotask(() => console.log("microtask fired"));

console.log("=== base64 ===");
const encoded = btoa("hello");
console.log("btoa:", encoded);
const decoded = atob(encoded);
console.log("atob:", decoded);

console.log("=== DOMException ===");
const err = new DOMException("something went wrong", "NetworkError");
console.log("DOMException name:", err.name);
console.log("DOMException message:", err.message);

console.log("=== reportError ===");
reportError(new Error("reported error"));

console.log("=== done ===");

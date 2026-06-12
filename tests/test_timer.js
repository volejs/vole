console.log("start");
setTimeout(() => console.log("timeout 50ms"), 50);
setTimeout(() => console.log("timeout 100ms"), 100);
setTimeout(() => console.log("timeout 0ms"), 0);
queueMicrotask(() => console.log("microtask"));
console.log("end");

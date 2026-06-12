performance.mark("start");
for (let i = 0; i < 1000000; i++) {}
performance.mark("end");
performance.measure("loop", "start", "end");
console.log("performance.now:", performance.now());

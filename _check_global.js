const internals = Object.keys(globalThis).filter(k => k.startsWith("__") || k.startsWith("_std") || k.startsWith("_ffi"));
console.log("internal global keys:", JSON.stringify(internals));

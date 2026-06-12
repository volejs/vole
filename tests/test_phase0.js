// Phase 0 基础功能测试

// 1. console
console.log("log test", 1, true, null);
console.info("info test");
console.warn("warn test");
console.error("error test");
console.debug("debug test");

console.time("timer1");
for (let i = 0; i < 1000000; i++) { }
console.timeEnd("timer1");

console.assert(true, "this should not print");
console.assert(false, "assertion failed", 42);

// 2. performance
console.log("=== performance ===");
const t0 = performance.now();
for (let i = 0; i < 1000000; i++) { }
const t1 = performance.now();
console.log("elapsed:", (t1 - t0).toFixed(3), "ms");

// 3. timers
console.log("=== timers ===");
const id = setTimeout(() => console.log("timeout fired"), 100);
console.log("timeout id:", id);
clearTimeout(id);

const intervalId = setInterval(() => console.log("interval fired"), 100);
console.log("interval id:", intervalId);
clearInterval(intervalId);

queueMicrotask(() => console.log("microtask fired"));

// 4. atob / btoa
console.log("=== base64 ===");
const encoded = btoa("hello");
console.log("btoa:", encoded);
const decoded = atob(encoded);
console.log("atob:", decoded);

// 5. DOMException
console.log("=== DOMException ===");
const err = new DOMException("something went wrong", "NetworkError");
console.log("DOMException name:", err.name);
console.log("DOMException message:", err.message);

// 6. reportError
console.log("=== reportError ===");
reportError(new Error("reported error"));

// 7. std:env
console.log("=== std:env ===");
import { get, set, has, remove, entries, env } from "std:env";
console.log("HOME:", get("HOME"));
console.log("PATH exists:", has("PATH"));
console.log("env.entries keys count:", Object.keys(entries()).length);
console.log("env.HOME:", env.get("HOME"));

// 8. std:process
console.log("=== std:process ===");
import { pid, platform, arch, version, args, cwd, stdout, stderr } from "std:process";
console.log("pid:", pid);
console.log("platform:", platform);
console.log("arch:", arch);
console.log("version:", version);
console.log("args:", args);
console.log("cwd:", cwd());
stdout.write("stdout write test\n");
stderr.write("stderr write test\n");

// 9. std:fs
console.log("=== std:fs ===");
import { readFileSync, readTextFileSync, writeFileSync, existsSync } from "std:fs";
writeFileSync("/tmp/test_phase0.txt", "hello from phase 0");
console.log("file exists:", existsSync("/tmp/test_phase0.txt"));
const text = readTextFileSync("/tmp/test_phase0.txt");
console.log("file content:", text);

// 10. std:path
console.log("=== std:path ===");
import { join, resolve, dirname, basename, extname } from "std:path";
console.log("join:", join("a", "b", "c"));
console.log("dirname:", dirname("/a/b/c.txt"));
console.log("basename:", basename("/a/b/c.txt"));
console.log("extname:", extname("/a/b/c.txt"));

console.log("=== Phase 0 test complete ===");

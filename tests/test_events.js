// Test Event
const ev = new Event("test", { cancelable: true });
console.log("Event type:", ev.type);
console.log("Event cancelable:", ev.cancelable);
ev.preventDefault();
console.log("Event defaultPrevented:", ev.defaultPrevented);

// Test CustomEvent
const ce = new CustomEvent("custom", { detail: { foo: 42 } });
console.log("CustomEvent detail:", ce.detail.foo);

// Test EventTarget
const target = new EventTarget();
let called = false;
target.addEventListener("hello", (e) => {
    called = true;
    console.log("EventTarget received:", e.type);
});
target.dispatchEvent(new Event("hello"));
console.log("Listener called:", called);

// Test once listener
let onceCount = 0;
target.addEventListener("once", () => { onceCount++; }, { once: true });
target.dispatchEvent(new Event("once"));
target.dispatchEvent(new Event("once"));
console.log("Once count:", onceCount);

// Test removeEventListener
let removedCount = 0;
function handler() { removedCount++; }
target.addEventListener("remove", handler);
target.removeEventListener("remove", handler);
target.dispatchEvent(new Event("remove"));
console.log("Removed count:", removedCount);

// Test AbortController
const ac = new AbortController();
let abortCalled = false;
ac.signal.addEventListener("abort", () => {
    abortCalled = true;
    console.log("Abort signal fired");
});
ac.abort();
console.log("Signal aborted:", ac.signal.aborted);
console.log("Abort reason:", ac.signal.reason?.name);
console.log("Abort listener called:", abortCalled);

// Test AbortSignal.timeout
const timeoutSignal = AbortSignal.timeout(50);
let timeoutFired = false;
timeoutSignal.addEventListener("abort", () => {
    timeoutFired = true;
    console.log("Timeout signal fired");
});
setTimeout(() => {
    console.log("Timeout signal aborted:", timeoutSignal.aborted);
    console.log("Timeout listener called:", timeoutFired);
    console.log("=== events test done ===");
}, 100);

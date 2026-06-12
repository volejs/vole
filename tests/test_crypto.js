// Test getRandomValues
const arr = new Uint8Array(16);
crypto.getRandomValues(arr);
console.log("random length:", arr.length);
console.log("random not all zero:", arr.some(v => v !== 0));

// Test subtle.digest SHA-256
async function testDigest() {
    const data = new TextEncoder().encode("hello");
    const hash = await crypto.subtle.digest("SHA-256", data);
    const hex = Array.from(new Uint8Array(hash))
        .map(b => b.toString(16).padStart(2, "0"))
        .join("");
    console.log("SHA-256:", hex);
    console.log("=== crypto test done ===");
}
testDigest();

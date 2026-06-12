// Test Headers
const h = new Headers();
h.set("Content-Type", "application/json");
h.append("X-Custom", "value1");
h.append("X-Custom", "value2");
console.log("Headers get:", h.get("Content-Type"));
console.log("Headers get case-insensitive:", h.get("content-type"));
console.log("Headers append:", h.get("X-Custom"));
console.log("Headers has:", h.has("X-Custom"));
h.delete("X-Custom");
console.log("Headers after delete:", h.has("X-Custom"));

// Test Request
const req = new Request("https://example.com/api", {
    method: "POST",
    headers: { "Authorization": "Bearer token" },
    body: "{\"foo\":42}"
});
console.log("Request url:", req.url);
console.log("Request method:", req.method);
console.log("Request headers:", req.headers.get("Authorization"));

// Test fetch with Request
async function testFetchRequest() {
    const resp = await fetch(new Request("https://httpbin.org/get"));
    console.log("fetch Request status:", resp.status);
    console.log("=== headers/request test done ===");
}
testFetchRequest();

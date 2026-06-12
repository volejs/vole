async function main() {
    try {
        const resp = await fetch("https://httpbin.org/get");
        console.log("status:", resp.status);
        console.log("ok:", resp.ok);
        console.log("statusText:", resp.statusText);
        const text = await resp.text();
        console.log("text length:", text.length);
        console.log("text startsWith:", text.startsWith("{"));
        
        // Test json()
        const resp2 = await fetch("https://httpbin.org/get");
        const json = await resp2.json();
        console.log("json url:", json.url);
        
        console.log("=== fetch test done ===");
    } catch (e) {
        console.log("fetch error:", e.message || e);
    }
}
main();

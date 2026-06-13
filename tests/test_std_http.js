import { request } from "vole:net/http";

async function main() {
    const resp = await request("https://httpbin.org/get");
    console.log("vole:net/http status:", resp.status);
    console.log("vole:net/http ok:", resp.ok);
    const text = await resp.text();
    console.log("vole:net/http text length:", text.length);
    console.log("=== vole:net/http test done ===");
}
main();

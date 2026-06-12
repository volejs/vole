import { request } from "std:http";

async function main() {
    const resp = await request("https://httpbin.org/get");
    console.log("std:http status:", resp.status);
    console.log("std:http ok:", resp.ok);
    const text = await resp.text();
    console.log("std:http text length:", text.length);
    console.log("=== std:http test done ===");
}
main();

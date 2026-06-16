import { request } from "vole:http";

async function main() {
  const resp = await request("https://httpbin.org/get");
  console.log("vole:http status:", resp.status);
  console.log("vole:http ok:", resp.ok);
  const text = await resp.text();
  console.log("vole:http text length:", text.length);
  console.log("=== vole:http test done ===");
}
main();

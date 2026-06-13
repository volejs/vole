import { serve } from "vole:net/http";

const server = serve((req) => {
  console.log("request:", req.method, req.url);
  if (req.url === "/hello") {
    return new Response("Hello from vole:net/http server", { status: 200 });
  }
  return new Response("Not found", { status: 404 });
}, { port: 8765, hostname: "127.0.0.1" });

async function test() {
  await server.listen();
  console.log("server listening");

  // Give the server a moment to start accepting connections
  await new Promise(r => setTimeout(r, 100));

  const resp = await fetch("http://127.0.0.1:8765/hello");
  console.log("client status:", resp.status);
  const text = await resp.text();
  console.log("client body:", text);

  if (resp.status !== 200 || text !== "Hello from vole:net/http server") {
    throw new Error("HTTP server test failed");
  }

  await server.shutdown();
  console.log("server shutdown");
  console.log("=== vole:net/http server test passed ===");
}

test();

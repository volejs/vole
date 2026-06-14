console.log("file loaded");
import { test } from "vole:test";

test("closed resolves", async () => {
  const stream = new ReadableStream({
    pull(controller) { controller.close(); }
  });
  const reader = stream.getReader();
  const closedPromise = reader.closed;
  const { done } = await reader.read();
  if (!done) throw new Error("expected done");
  await closedPromise;
  console.log("closed ok");
});

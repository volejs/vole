import { test } from "std:test";

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

test("ReadableStreamDefaultController.desiredSize respects highWaterMark", () => {
  const stream = new ReadableStream(
    {
      start(controller) {
        controller.enqueue("a");
        controller.enqueue("b");
      },
    },
    { highWaterMark: 3 }
  );

  const controller = stream._controller;
  if (controller.desiredSize !== 1) {
    throw new Error("expected desiredSize 1, got " + controller.desiredSize);
  }
});

test("ReadableStreamDefaultController.desiredSize uses size function", () => {
  const stream = new ReadableStream(
    {
      start(controller) {
        controller.enqueue("ab");
        controller.enqueue("c");
      },
    },
    {
      highWaterMark: 10,
      size: (chunk) => chunk.length,
    }
  );

  const controller = stream._controller;
  if (controller.desiredSize !== 7) {
    throw new Error("expected desiredSize 7, got " + controller.desiredSize);
  }
});

test("ReadableStream backpressure stops pulling when full", async () => {
  let pullCount = 0;
  const stream = new ReadableStream(
    {
      pull(controller) {
        pullCount++;
        controller.enqueue(pullCount);
      },
    },
    { highWaterMark: 2 }
  );

  await delay(10);

  // Pull should have been called until desiredSize <= 0 (i.e. 2 chunks queued).
  if (pullCount !== 2) {
    throw new Error("expected 2 pulls, got " + pullCount);
  }

  const reader = stream.getReader();
  await reader.read();
  await delay(0);

  // After consuming one chunk, desiredSize becomes 1, so pull is called once.
  if (pullCount !== 3) {
    throw new Error("expected 3 pulls after read, got " + pullCount);
  }

  reader.releaseLock();
});

test("fetch Response.body streams chunks instead of enqueueing all at once", async () => {
  // Use a local echo server or HTTP server if available; otherwise skip.
  if (typeof globalThis.__test_server_url__ !== "string") {
    return;
  }

  const resp = await fetch(globalThis.__test_server_url__ + "/large");
  const reader = resp.body.getReader();

  let chunkCount = 0;
  let totalBytes = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunkCount++;
    totalBytes += value.length;
  }

  if (chunkCount < 2) {
    throw new Error("expected body to be split into multiple chunks, got " + chunkCount);
  }
  if (totalBytes === 0) {
    throw new Error("expected non-empty body");
  }
});

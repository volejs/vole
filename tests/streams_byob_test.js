import { test } from "vole:test";

test("BYOB reader reads bytes from a byte stream", async () => {
  const stream = new ReadableStream({
    type: "bytes",
    pull(controller) {
      controller.enqueue(new Uint8Array([1, 2, 3]));
      controller.close();
    },
  });

  const reader = stream.getReader({ mode: "byob" });
  const view = new Uint8Array(10);
  const { done, value } = await reader.read(view);
  if (done) throw new Error("expected not done");
  if (value !== view) throw new Error("expected same view object");
  if (value[0] !== 1 || value[1] !== 2 || value[2] !== 3) {
    throw new Error("unexpected bytes: " + Array.from(value).join(","));
  }

  const second = await reader.read(new Uint8Array(4));
  if (!second.done) throw new Error("expected done after close");
});

test("BYOB reader buffers remainder when chunk exceeds view", async () => {
  const stream = new ReadableStream({
    type: "bytes",
    start(controller) {
      controller.enqueue(new Uint8Array([1, 2, 3, 4, 5]));
      controller.close();
    },
  });

  const reader = stream.getReader({ mode: "byob" });
  const v1 = new Uint8Array(2);
  const r1 = await reader.read(v1);
  if (r1.done) throw new Error("expected not done");
  if (r1.value[0] !== 1 || r1.value[1] !== 2) throw new Error("bad first chunk");

  const v2 = new Uint8Array(3);
  const r2 = await reader.read(v2);
  if (r2.done) throw new Error("expected not done");
  if (r2.value[0] !== 3 || r2.value[1] !== 4 || r2.value[2] !== 5) {
    throw new Error("bad second chunk");
  }

  const r3 = await reader.read(new Uint8Array(1));
  if (!r3.done) throw new Error("expected done");
});

test("ReadableByteStreamController.respond resolves pending BYOB read", async () => {
  let controller;
  const stream = new ReadableStream({
    type: "bytes",
    start(c) {
      controller = c;
    },
  });

  const reader = stream.getReader({ mode: "byob" });
  const view = new Uint8Array(8);
  const p = reader.read(view);
  controller.respond(3);

  const { done, value } = await p;
  if (done) throw new Error("expected not done");
  if (value.byteLength !== 3) throw new Error("expected byteLength 3, got " + value.byteLength);
});

test("ReadableByteStreamController.respondWithNewView resolves read", async () => {
  let controller;
  const stream = new ReadableStream({
    type: "bytes",
    start(c) {
      controller = c;
    },
  });

  const reader = stream.getReader({ mode: "byob" });
  const original = new Uint8Array(8);
  const p = reader.read(original);
  const newView = new Uint8Array(original.buffer, 0, 4);
  controller.respondWithNewView(newView);

  const { done, value } = await p;
  if (done) throw new Error("expected not done");
  if (value !== newView) throw new Error("expected new view");
  if (value.byteLength !== 4) throw new Error("expected byteLength 4");
});

test("BYOB reader cannot be acquired for non-byte stream", () => {
  const stream = new ReadableStream();
  try {
    stream.getReader({ mode: "byob" });
    throw new Error("expected TypeError");
  } catch (err) {
    if (!(err instanceof TypeError)) throw err;
  }
});

test("Byte stream supports default reader", async () => {
  const stream = new ReadableStream({
    type: "bytes",
    start(controller) {
      controller.enqueue(new Uint8Array([10, 20, 30]));
      controller.close();
    },
  });

  const reader = stream.getReader();
  const { done, value } = await reader.read();
  if (done) throw new Error("expected not done");
  if (!(value instanceof Uint8Array)) throw new Error("expected Uint8Array");
  if (value.length !== 3 || value[2] !== 30) throw new Error("bad value");

  const second = await reader.read();
  if (!second.done) throw new Error("expected done");
});

test("Byte stream default highWaterMark is 0", () => {
  let controller;
  new ReadableStream({
    type: "bytes",
    start(c) {
      controller = c;
    },
  });
  if (controller.desiredSize !== 0) {
    throw new Error("expected desiredSize 0, got " + controller.desiredSize);
  }
});

test("Releasing BYOB reader rejects pending read", async () => {
  let controller;
  const stream = new ReadableStream({
    type: "bytes",
    start(c) {
      controller = c;
    },
  });

  const reader = stream.getReader({ mode: "byob" });
  const p = reader.read(new Uint8Array(4));
  reader.releaseLock();

  try {
    await p;
    throw new Error("expected rejection");
  } catch (err) {
    if (!(err instanceof TypeError)) throw err;
  }
});

test("Canceling byte stream invokes source cancel with reason", async () => {
  let cancelReason;
  const stream = new ReadableStream({
    type: "bytes",
    cancel(reason) {
      cancelReason = reason;
    },
  });

  const reader = stream.getReader({ mode: "byob" });
  await stream.cancel("bye");
  if (cancelReason !== "bye") throw new Error("bad cancel reason: " + cancelReason);
});

test("Controller error rejects pending BYOB read", async () => {
  let controller;
  const stream = new ReadableStream({
    type: "bytes",
    start(c) {
      controller = c;
    },
  });

  const reader = stream.getReader({ mode: "byob" });
  const p = reader.read(new Uint8Array(4));
  controller.error(new Error("boom"));

  try {
    await p;
    throw new Error("expected rejection");
  } catch (err) {
    if (err.message !== "boom") throw err;
  }
});

test("respond without pending BYOB read throws", () => {
  let controller;
  new ReadableStream({
    type: "bytes",
    start(c) {
      controller = c;
    },
  });

  try {
    controller.respond(1);
    throw new Error("expected TypeError");
  } catch (err) {
    if (!(err instanceof TypeError)) throw err;
  }
});

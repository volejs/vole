import { test } from "vole:test";

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

test("ReadableStream pull is async, not polled", async () => {
  let pullCount = 0;
  const stream = new ReadableStream({
    pull(controller) {
      pullCount++;
      if (pullCount === 1) {
        controller.enqueue("a");
      } else if (pullCount === 2) {
        controller.enqueue("b");
      } else {
        controller.close();
      }
    },
  });

  const reader = stream.getReader();
  const values = [];
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    values.push(value);
  }

  if (values.join(",") !== "a,b") {
    throw new Error("unexpected values: " + values.join(","));
  }
  if (pullCount !== 3) {
    throw new Error("expected 3 pulls, got " + pullCount);
  }
});

test("ReadableStream async pull resolves read", async () => {
  const stream = new ReadableStream({
    async pull(controller) {
      await delay(20);
      controller.enqueue(42);
      controller.close();
    },
  });

  const reader = stream.getReader();
  const { done, value } = await reader.read();
  if (done) throw new Error("unexpected done");
  if (value !== 42) throw new Error("unexpected value: " + value);
  const second = await reader.read();
  if (!second.done) throw new Error("expected done");
});

test("ReadableStream closed resolves reader.closed", async () => {
  const stream = new ReadableStream({
    pull(controller) {
      controller.close();
    },
  });

  const reader = stream.getReader();
  const closedPromise = reader.closed;
  const { done } = await reader.read();
  if (!done) throw new Error("expected done after close");
  await closedPromise;
});

test("TransformStream pipes data", async () => {
  const transform = new TransformStream({
    transform(chunk, controller) {
      controller.enqueue(chunk.toUpperCase());
    },
  });

  const writer = transform.writable.getWriter();
  await writer.write("hello");
  await writer.close();

  const reader = transform.readable.getReader();
  const { done, value } = await reader.read();
  if (done) throw new Error("unexpected done");
  if (value !== "HELLO") throw new Error("unexpected value: " + value);
  const end = await reader.read();
  if (!end.done) throw new Error("expected done");
});

test("WritableStream closed resolves after writer.close", async () => {
  let closedCalled = false;
  const stream = new WritableStream({
    close() {
      closedCalled = true;
    },
  });

  const writer = stream.getWriter();
  const closedPromise = writer.closed;
  await writer.close();
  await closedPromise;
  if (!closedCalled) throw new Error("sink close was not called");
});

test("TextEncoderStream encodes chunks", async () => {
  const encoder = new TextEncoderStream();
  const writer = encoder.writable.getWriter();
  await writer.write("hello");
  await writer.close();

  const reader = encoder.readable.getReader();
  const chunks = [];
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
  }

  const decoded = new TextDecoder().decode(
    chunks.length === 1 ? chunks[0] : new Uint8Array(0)
  );
  if (decoded !== "hello") {
    throw new Error("unexpected decoded: " + decoded);
  }
});

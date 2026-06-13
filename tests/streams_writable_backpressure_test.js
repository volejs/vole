import { test } from "vole:test";

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

test("WritableStream write resolves after sink finishes", async () => {
  let writeStarted = false;
  let writeFinished = false;
  const stream = new WritableStream({
    async write(chunk) {
      writeStarted = true;
      await delay(10);
      writeFinished = true;
    },
  });

  const writer = stream.getWriter();
  const writePromise = writer.write("x");
  if (writeFinished) throw new Error("write should not be synchronous");
  await writePromise;
  if (!writeFinished) throw new Error("write should have finished");
});

test("WritableStream writer.ready reflects backpressure", async () => {
  const resolvers = new Map();
  const stream = new WritableStream(
    {
      write(chunk) {
        return new Promise((resolve) => {
          resolvers.set(chunk, resolve);
        });
      },
    },
    { highWaterMark: 2 }
  );

  const writer = stream.getWriter();
  await writer.ready;

  const promiseA = writer.write("a");
  await delay(0);
  await writer.ready;

  const promiseB = writer.write("b");
  await delay(0);
  let readyResolved = false;
  writer.ready.then(() => {
    readyResolved = true;
  });
  await delay(5);
  if (readyResolved) throw new Error("ready should be pending while full");

  resolvers.get("a")();
  await promiseA;
  resolvers.get("b")();
  await promiseB;
  await writer.ready;
});

test("WritableStream writes are queued and processed in order", async () => {
  const written = [];
  const stream = new WritableStream({
    async write(chunk) {
      await delay(1);
      written.push(chunk);
    },
  });

  const writer = stream.getWriter();
  await writer.write("a");
  await writer.write("b");
  await writer.write("c");

  if (written.join(",") !== "a,b,c") {
    throw new Error("unexpected order: " + written.join(","));
  }
});

test("WritableStream close waits for queued writes", async () => {
  const written = [];
  const stream = new WritableStream({
    async write(chunk) {
      await delay(5);
      written.push(chunk);
    },
  });

  const writer = stream.getWriter();
  const writePromise = writer.write("a");
  const closePromise = writer.close();
  await writePromise;
  if (written.length !== 1) throw new Error("write should have completed before close");
  await closePromise;
  if (written.length !== 1) throw new Error("close should not add writes");
});

test("WritableStream abort rejects pending writes", async () => {
  const stream = new WritableStream({
    write(chunk) {
      return new Promise(() => { });
    },
  });

  const writer = stream.getWriter();
  const writePromise = writer.write("a");
  await writer.abort("stop");

  let rejected = false;
  try {
    await writePromise;
  } catch (e) {
    rejected = true;
  }
  if (!rejected) throw new Error("pending write should be rejected on abort");
});

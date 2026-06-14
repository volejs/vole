import { test } from "vole:test";

function silence(writer, reader) {
  writer.ready.catch(() => {});
  writer.closed.catch(() => {});
  reader.closed.catch(() => {});
}

test("TransformStream writable abort errors readable side", async () => {
  const t = new TransformStream({
    transform(c, ctrl) {
      ctrl.enqueue(c);
    },
  });
  const writer = t.writable.getWriter();
  const reader = t.readable.getReader();
  silence(writer, reader);
  await writer.write("a");
  await writer.abort("bye");
  try {
    await reader.read();
    throw new Error("expected rejection");
  } catch (e) {
    if (e !== "bye") throw e;
  }
});

test("TransformStream readable cancel errors writable side", async () => {
  const t = new TransformStream({
    transform(c, ctrl) {
      ctrl.enqueue(c);
    },
  });
  const writer = t.writable.getWriter();
  const reader = t.readable.getReader();
  silence(writer, reader);
  await reader.cancel("cancelled");
  try {
    await writer.write("a");
    throw new Error("expected rejection");
  } catch (e) {
    if (e !== "cancelled") throw e;
  }
});

test("TransformStream transform throw errors both sides", async () => {
  const t = new TransformStream({
    transform() {
      throw new Error("boom");
    },
  });
  const writer = t.writable.getWriter();
  const reader = t.readable.getReader();
  silence(writer, reader);
  try {
    await writer.write("a");
    throw new Error("expected write rejection");
  } catch (e) {
    if (e.message !== "boom") throw e;
  }
  try {
    await reader.read();
    throw new Error("expected read rejection");
  } catch (e) {
    if (e.message !== "boom") throw e;
  }
});

test("TransformStream transform rejection errors both sides", async () => {
  const t = new TransformStream({
    transform() {
      return Promise.reject(new Error("boom"));
    },
  });
  const writer = t.writable.getWriter();
  const reader = t.readable.getReader();
  silence(writer, reader);
  try {
    await writer.write("a");
    throw new Error("expected write rejection");
  } catch (e) {
    if (e.message !== "boom") throw e;
  }
  try {
    await reader.read();
    throw new Error("expected read rejection");
  } catch (e) {
    if (e.message !== "boom") throw e;
  }
});

test("TransformStream flush error rejects close and readable", async () => {
  const t = new TransformStream({
    transform(c, ctrl) {
      ctrl.enqueue(c);
    },
    flush() {
      throw new Error("flush err");
    },
  });
  const writer = t.writable.getWriter();
  const reader = t.readable.getReader();
  silence(writer, reader);
  await writer.write("a");
  try {
    await writer.close();
    throw new Error("expected close rejection");
  } catch (e) {
    if (e.message !== "flush err") throw e;
  }
  try {
    await reader.read();
    throw new Error("expected read rejection");
  } catch (e) {
    if (e.message !== "flush err") throw e;
  }
});

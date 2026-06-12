import { test } from "std:test";

const assert = {
  equal(actual, expected) {
    if (actual !== expected) {
      throw new Error(`expected ${expected}, got ${actual}`);
    }
  },
};

async function readStream(stream) {
  const reader = stream.getReader();
  const chunks = [];
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
  }
  const total = chunks.reduce((sum, c) => sum + c.length, 0);
  const result = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }
  return result;
}

test("Blob stream returns a real ReadableStream", async () => {
  const blob = new Blob(["hello stream"]);
  const stream = blob.stream();
  assert.equal(stream instanceof ReadableStream, true);

  const bytes = await readStream(stream);
  const text = new TextDecoder().decode(bytes);
  assert.equal(text, "hello stream");
});

test("Blob bytes returns Uint8Array", async () => {
  const blob = new Blob(["hello bytes"]);
  const bytes = await blob.bytes();
  assert.equal(bytes instanceof Uint8Array, true);
  assert.equal(bytes.length, 11);
  assert.equal(new TextDecoder().decode(bytes), "hello bytes");
});

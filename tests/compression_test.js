import { test } from "vole:test";

const assert = {
  equal(actual, expected) {
    if (actual !== expected) {
      throw new Error(`expected ${expected}, got ${actual}`);
    }
  },
};

async function concatChunks(stream) {
  const reader = stream.getReader();
  const chunks = [];
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
  }
  return chunks;
}

function concatenateBytes(chunks) {
  const total = chunks.reduce((sum, c) => sum + c.length, 0);
  const result = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }
  return result;
}

test("CompressionStream / DecompressionStream round-trip", async () => {
  assert.equal(typeof CompressionStream, "function");
  assert.equal(typeof DecompressionStream, "function");

  const input = new TextEncoder().encode("hello compression");

  const cs = new CompressionStream("gzip");
  const writer = cs.writable.getWriter();
  writer.write(input);
  writer.close();
  const compressed = concatenateBytes(await concatChunks(cs.readable));
  assert.equal(compressed.length > 0, true);

  const ds = new DecompressionStream("gzip");
  const writer2 = ds.writable.getWriter();
  writer2.write(compressed);
  writer2.close();
  const decompressed = concatenateBytes(await concatChunks(ds.readable));
  const output = new TextDecoder().decode(decompressed);
  assert.equal(output, "hello compression");
});

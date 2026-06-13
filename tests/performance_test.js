import { test } from "vole:test";

const assert = {
  equal(actual, expected) {
    if (actual !== expected) {
      throw new Error(`expected ${expected}, got ${actual}`);
    }
  },
};

test("Performance constructor is exposed", () => {
  assert.equal(typeof Performance, "function");
  assert.equal(performance instanceof Performance, true);
});

test("Performance instance methods work", () => {
  const p = new Performance();
  assert.equal(typeof p.now, "function");
  assert.equal(typeof p.timeOrigin, "number");
  assert.equal(p.timeOrigin, performance.timeOrigin);
  assert.equal(typeof p.now(), "number");
  assert.equal(p.now() >= 0, true);
});

test("performance.mark and performance.measure", () => {
  const mark = performance.mark("start");
  assert.equal(typeof mark, "object");
  assert.equal(mark.name, "start");
  assert.equal(mark.entryType, "mark");

  const measure = performance.measure("dur", "start");
  assert.equal(typeof measure, "object");
  assert.equal(measure.name, "dur");
  assert.equal(measure.entryType, "measure");
  assert.equal(measure.duration >= 0, true);

  performance.clearMarks("start");
  performance.clearMeasures("dur");
});

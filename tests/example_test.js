import { test, describe, it, beforeAll, afterAll, beforeEach, afterEach } from "std:test";

test("simple addition", () => {
  if (1 + 1 !== 2) {
    throw new Error("math is broken");
  }
});

describe("suite A", () => {
  let value = 0;

  beforeAll(() => {
    value = 10;
  });

  beforeEach(() => {
    value += 1;
  });

  it("should have incremented value", () => {
    if (value !== 11) {
      throw new Error("expected 11, got " + value);
    }
  });

  it("should increment again", () => {
    if (value !== 12) {
      throw new Error("expected 12, got " + value);
    }
  });

  afterAll(() => {
    console.log("suite A done");
  });
});

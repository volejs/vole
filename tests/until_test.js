import { test } from "vole:test";
import {
  WaitUntil,
  drainManaged,
  managedPendingCount,
  waitUntil,
} from "vole:until";

const assert = {
  equal(actual, expected) {
    if (actual !== expected) {
      throw new Error(`expected ${expected}, got ${actual}`);
    }
  },
};

test("WaitUntil defaults to a managed scope", async () => {
  const before = managedPendingCount();
  const until = new WaitUntil();

  assert.equal(until.managed, true);
  waitUntil(until, Promise.resolve());
  assert.equal(until.pendingCount, 1);
  assert.equal(managedPendingCount(), before + 1);

  await drainManaged(1000);
  assert.equal(until.pendingCount, 0);
  assert.equal(managedPendingCount(), before);
});

test("WaitUntil can create detached scopes", async () => {
  const before = managedPendingCount();
  const until = new WaitUntil({ managed: false });

  assert.equal(until.managed, false);
  until.waitUntil(Promise.resolve());
  assert.equal(until.pendingCount, 1);
  assert.equal(managedPendingCount(), before);

  await until.drain(1000);
  assert.equal(until.pendingCount, 0);
  assert.equal(managedPendingCount(), before);
});

test("WaitUntil drain waits for promise work", async () => {
  const until = new WaitUntil({ managed: false });
  let completed = false;

  until.waitUntil(
    Promise.resolve().then(() => {
      completed = true;
    }),
  );

  assert.equal(completed, false);
  await until.drain(1000);
  assert.equal(completed, true);
  assert.equal(until.pendingCount, 0);
});

test("WaitUntil rejects invalid options", () => {
  let threw = false;

  try {
    new WaitUntil(null);
  } catch (error) {
    threw = error instanceof TypeError;
  }

  assert.equal(threw, true);
});

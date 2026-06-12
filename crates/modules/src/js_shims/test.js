const tests = [];
const suites = [];

let currentSuite = null;

export function test(name, fn) {
  if (typeof name === "object" && name !== null) {
    fn = name.fn;
    name = name.name;
  }
  if (typeof fn !== "function") {
    throw new TypeError("test() second argument must be a function");
  }
  const entry = { name: String(name), fn, suite: currentSuite };
  if (currentSuite) {
    currentSuite.tests.push(entry);
  } else {
    tests.push(entry);
  }
}

export function describe(name, fn) {
  const suite = {
    name: String(name),
    tests: [],
    beforeAll: [],
    afterAll: [],
    beforeEach: [],
    afterEach: [],
  };
  const prev = currentSuite;
  currentSuite = suite;
  suites.push(suite);
  try {
    fn();
  } finally {
    currentSuite = prev;
  }
}

export function it(name, fn) {
  test(name, fn);
}

export function beforeAll(fn) {
  if (currentSuite) currentSuite.beforeAll.push(fn);
}

export function afterAll(fn) {
  if (currentSuite) currentSuite.afterAll.push(fn);
}

export function beforeEach(fn) {
  if (currentSuite) currentSuite.beforeEach.push(fn);
}

export function afterEach(fn) {
  if (currentSuite) currentSuite.afterEach.push(fn);
}

export async function run() {
  let passed = 0;
  let failed = 0;

  async function runOne(entry) {
    const { name, fn, suite } = entry;
    try {
      if (suite) {
        for (const be of suite.beforeEach) await be();
      }
      await fn();
      if (suite) {
        for (const ae of suite.afterEach) await ae();
      }
      passed++;
      console.log(`  ✓ ${name}`);
    } catch (e) {
      failed++;
      console.log(`  ✗ ${name}`);
      console.log(`    ${e.stack || e.message || e}`);
    }
  }

  for (const entry of tests) {
    await runOne(entry);
  }

  for (const suite of suites) {
    if (suite.tests.length === 0) continue;
    console.log(`\n${suite.name}`);
    for (const ba of suite.beforeAll) await ba();
    for (const entry of suite.tests) {
      await runOne(entry);
    }
    for (const aa of suite.afterAll) await aa();
  }

  console.log(`\n${passed} passed, ${failed} failed`);
  if (failed > 0) {
    throw new Error(`${failed} test(s) failed`);
  }
}

export default { test, describe, it, beforeAll, afterAll, beforeEach, afterEach, run };

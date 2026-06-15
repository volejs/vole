import { readTextFileSync } from "vole:fs";
import { setWptCurrentFile, setWptExpectedFailures } from "./harness.js";

const WPT_ROOT = "third_party/wpt";
const SUITE_ROOT = "tests/wpt/suites";
const EXPECTED_FAILURES = "tests/wpt/expected-failures.txt";

function readList(path) {
  return readTextFileSync(path)
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"));
}

function stripMeta(code) {
  return code
    .split(/\r?\n/)
    .filter((line) => !line.startsWith("// META:"))
    .join("\n");
}

export function registerWptSuite(name) {
  setWptExpectedFailures(readList(EXPECTED_FAILURES));
  for (const relativePath of readList(`${SUITE_ROOT}/${name}.txt`)) {
    setWptCurrentFile(relativePath);
    const code = stripMeta(readTextFileSync(`${WPT_ROOT}/${relativePath}`));
    (0, eval)(`${code}\n//# sourceURL=wpt://${relativePath}`);
  }
}

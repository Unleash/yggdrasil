import test from "ava";
import fs from "fs";
import { UnleashEngine } from "node-sdk";

const SPEC_PATH = `../client-specification/specifications/`;

test("Runs client spec", async (t) => {
  const testIndex = JSON.parse(fs.readFileSync(`${SPEC_PATH}/index.json`));

  for (const testFile of testIndex) {
    const testData = JSON.parse(
      fs.readFileSync(`${SPEC_PATH}/${testFile}`, "utf-8")
    );

    const { name, state, tests, variantTests } = testData;
    const engine = new UnleashEngine();
    engine.takeState(state);
    for (const test of tests || []) {
      const engineResult = engine.isEnabled(test.toggleName, test.context);
      t.is(engineResult, test.expectedResult);
    }

    for (const variantTest of variantTests || []) {
      const engineResult = engine.getVariant(variantTest.toggleName, variantTest.context);
      t.deepEqual(engineResult, variantTest.expectedResult);
    }

    t.truthy(true);
  }
});

test("Runs the same evaluation twice", t => {
  const engine = new UnleashEngine();
  const firstCheck = engine.isEnabled("test", {});
  const secondCheck = engine.isEnabled("test", {});
  // engine.isEnabled("test")
  t.falsy(firstCheck || secondCheck);
})

const yggdrasil = require("../pkg/yggdrasil_engine.js");

test("Rule evaluates correctly", () => {
  const context = {
    userId: "7",
  };

  const result = yggdrasil.evaluate("user_id > 6", context);
  expect(result).toBe(true);
});

test("Unknown base properties are ignored", () => {
  const context = {
    thisPropDoesNotExist: "7",
  };

  const result = yggdrasil.evaluate("user_id > 6", context);
  expect(result).toBe(false);
});

test("Properties correctly propagate", () => {
  const context = {
    properties: {
      customProperty: "7",
    },
  };

  const result = yggdrasil.evaluate('context["customProperty"] > 6', context);
  expect(result).toBe(true);
});

test("Invalid rules raise an error", () => {
  expect(() => {
    yggdrasil.evaluate("This is not a valid rule", {});
  }).toThrow();
});

test("Context can be empty but not null or undefined", () => {
  const rule = "user_id > 6";

  yggdrasil.evaluate(rule, {}); // should not throw

  expect(() => {
    yggdrasil.evaluate(rule, null);
  }).toThrow();

  expect(() => {
    yggdrasil.evaluate(rule, undefined);
  }).toThrow();

  expect(() => {
    yggdrasil.evaluate(rule);
  }).toThrow();
});

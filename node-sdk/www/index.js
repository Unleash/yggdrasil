import { UnleashEngine, Context } from "node-sdk";

let engine = new UnleashEngine();
let context = new Context("dev");

console.log(engine);
console.log(engine.isEnabled("AlwaysOn", context));
// engine.isEnabled();
// unleash.greet();

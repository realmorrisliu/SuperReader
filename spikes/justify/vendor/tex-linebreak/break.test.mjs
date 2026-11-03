import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

const source = readFileSync(new URL("./layout.js", import.meta.url), "utf8");
const createTexLayout = new Function(`${source}\nreturn createTexLayout;`)();
const tex = createTexLayout();

function box(width) {
  return { type: "box", width };
}

function glue(width, stretch, shrink) {
  return { type: "glue", width, stretch, shrink };
}

const items = [];
for (let i = 0; i < 8; i++) {
  if (items.length) items.push(glue(4, 4, 2));
  items.push(box(12));
}
items.push(glue(0, 1000, 0));
items.push(tex.forcedBreak());

const breaks = tex.breakLines(items, 30, {
  maxAdjustmentRatio: 1,
  initialMaxAdjustmentRatio: 0.5,
});
assert.ok(breaks.length > 1, `breakpoints ${breaks.join(",")}`);
assert.equal(breaks[0], 0);
assert.equal(breaks[breaks.length - 1], items.length - 1);

const over = [box(10), glue(5, 10, 10), box(10), tex.forcedBreak()];
assert.throws(
  () => tex.breakLines(over, 100, { maxAdjustmentRatio: 1 }),
  (error) => error instanceof tex.MaxAdjustmentExceededError,
);

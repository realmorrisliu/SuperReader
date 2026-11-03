import { expect, open, test } from "nsh/test";

test("spike title is on the chrome", async () => {
  const app = await open(".");
  expect(app.texts().join("\n")).toContain("SuperReader spike");
});

test("no night mode", async () => {
  const app = await open(".");
  expect(app.texts().join("\n").toLowerCase()).not.toContain("night");
});

test("no theme switch", async () => {
  const app = await open(".");
  expect(app.texts().join("\n").toLowerCase()).not.toContain("theme");
});

test("no spread", async () => {
  const app = await open(".");
  expect(app.texts().join("\n").toLowerCase()).not.toContain("spread");
});

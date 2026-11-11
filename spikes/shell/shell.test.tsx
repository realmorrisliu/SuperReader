import { expect, open, test } from "nsh/test";

test("no spike title", async () => {
  const app = await open(".");
  expect(app.texts().join("\n")).not.toContain("SuperReader spike");
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

test("full form lists the sidebar groups and the chat field", async () => {
  const app = await open(".");
  const blob = `\n${app.texts().join("\n")}\n`;
  const at = (label: string) => blob.indexOf(`\n${label}\n`);
  const library = at("书库");
  const notes = at("笔记");
  const lists = at("书单");
  const sources = at("来源");
  const settings = at("设置");
  expect(library).toBeGreaterThanOrEqual(0);
  expect(notes).toBeGreaterThan(library);
  expect(lists).toBeGreaterThan(notes);
  expect(sources).toBeGreaterThan(lists);
  expect(settings).toBeGreaterThan(sources);
  expect(blob).not.toContain("Drop a book to open it");
  expect(blob).not.toContain("Import from Apple Books");
});

test("sidebar toggle hides and shows the sidebar", async () => {
  const app = await open(".");
  expect(app.texts().join("\n")).toContain("设置");
  await app.press("侧边栏");
  expect(app.texts().join("\n")).not.toContain("设置");
  await app.press("侧边栏");
  expect(app.texts().join("\n")).toContain("设置");
});

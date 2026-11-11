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
  const blob = app.texts().join("\n");
  const library = blob.indexOf("书库");
  const notes = blob.indexOf("笔记");
  const lists = blob.indexOf("书单");
  const sources = blob.indexOf("来源");
  const settings = blob.indexOf("设置");
  expect(library).toBeGreaterThanOrEqual(0);
  expect(notes).toBeGreaterThan(library);
  expect(lists).toBeGreaterThan(notes);
  expect(sources).toBeGreaterThan(lists);
  expect(settings).toBeGreaterThan(sources);
  expect(blob).toContain("问问 Slowpage…");
  expect(blob).toContain("你的阅读助手");
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

test("panel toggle hides and shows the chat field", async () => {
  const app = await open(".");
  expect(app.texts().join("\n")).toContain("问问 Slowpage…");
  await app.press("对话");
  expect(app.texts().join("\n")).not.toContain("问问 Slowpage…");
  await app.press("对话");
  expect(app.texts().join("\n")).toContain("问问 Slowpage…");
});

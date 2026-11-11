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

test("an imported source appears under 来源", async () => {
  const app = await open(".");
  const between = (blob: string) => {
    const start = blob.indexOf("\n来源\n");
    const end = blob.indexOf("\n添加来源\n");
    return blob.slice(start, end);
  };
  const before = `\n${app.texts().join("\n")}\n`;
  expect(between(before)).not.toContain("Apple Books");
  await app.press("Apple Books");
  const after = `\n${app.texts().join("\n")}\n`;
  expect(between(after)).toContain("Apple Books");
  expect(after).toContain("书架");
  expect(after).toContain("Plain Essay");
  expect(after).toContain("Field Guide");
  expect(after).toContain("Folder Book");
});

test("添加来源 adds a second source and PDF filters the grid", async () => {
  const app = await open(".");
  await app.press("Apple Books");
  await app.press("PDF");
  const pdf = `\n${app.texts().join("\n")}\n`;
  expect(pdf).toContain("Field Guide");
  expect(pdf).not.toContain("Plain Essay");
  await app.press("添加来源");
  await app.press("calibre");
  await app.press("全部书籍");
  const both = `\n${app.texts().join("\n")}\n`;
  const start = both.indexOf("\n来源\n");
  const end = both.indexOf("\n添加来源\n");
  const sources = both.slice(start, end);
  expect(sources).toContain("Apple Books");
  expect(sources).toContain("calibre");
  expect(both).toContain("Harbor Notes");
  expect(both).toContain("Plain Essay");
});

test("继续阅读 lists opened books with the latest first", async () => {
  const app = await open(".");
  await app.press("Apple Books");
  await app.press("Plain Essay");
  await app.wait(50);
  await app.press("Field Guide");
  const blob = `\n${app.texts().join("\n")}\n`;
  const start = blob.indexOf("\n继续阅读\n");
  const end = blob.indexOf("\n书架\n");
  const row = blob.slice(start, end);
  const guide = row.indexOf("Field Guide");
  const essay = row.indexOf("Plain Essay");
  expect(start).toBeGreaterThanOrEqual(0);
  expect(guide).toBeGreaterThanOrEqual(0);
  expect(essay).toBeGreaterThan(guide);
});

test("sidebar toggle hides and shows the sidebar", async () => {
  const app = await open(".");
  expect(app.texts().join("\n")).toContain("设置");
  await app.press("侧边栏");
  expect(app.texts().join("\n")).not.toContain("设置");
  await app.press("侧边栏");
  expect(app.texts().join("\n")).toContain("设置");
});

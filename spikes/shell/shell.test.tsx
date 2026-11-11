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

test("empty shelf offers Apple Books import", async () => {
  const app = await open(".");
  const blob = app.texts().join("\n");
  expect(blob).toContain("Drop a book to open it");
  expect(blob).toContain("Import from Apple Books");
  expect(blob).toContain("calibre library");
  expect(blob).toContain("Folder");
  expect(blob).not.toContain("Plain Essay");
  expect(blob).not.toContain("Harbor Notes");
});

test("importing the apple fixture lists the books and a result line", async () => {
  const app = await open(".");
  await app.press("Import from Apple Books");
  const blob = app.texts().join("\n");
  expect(blob).toContain("Plain Essay");
  expect(blob).toContain("Field Guide");
  expect(blob).toContain("Folder Book");
  expect(blob).toContain("Store");
  expect(blob).toContain("Not downloaded");
  expect(blob).toContain(
    "Imported 3, 1 store-protected unavailable, 1 not downloaded from iCloud",
  );
});

test("pressing Plain Essay leaves the other books", async () => {
  const app = await open(".");
  await app.press("Import from Apple Books");
  await app.press("Plain Essay");
  const blob = app.texts().join("\n");
  expect(blob).toContain("Plain Essay");
  expect(blob).not.toContain("Field Guide");
  expect(blob).not.toContain("Folder Book");
});

test("opening Field Guide from the shelf", async () => {
  const app = await open(".");
  await app.press("Import from Apple Books");
  await app.press("Field Guide");
  const blob = app.texts().join("\n");
  expect(blob).toContain("Field Guide");
  expect(blob).not.toContain("Plain Essay");
  expect(blob).not.toContain("Folder Book");
});

test("switching an epub and a pdf from the shelf", async () => {
  const app = await open(".");
  await app.press("Import from Apple Books");
  await app.press("Plain Essay");
  let blob = app.texts().join("\n");
  expect(blob).toContain("Plain Essay");
  expect(blob).not.toContain("Field Guide");
  await app.press("Library");
  await app.press("Field Guide");
  blob = app.texts().join("\n");
  expect(blob).toContain("Field Guide");
  expect(blob).not.toContain("Plain Essay");
  await app.press("Library");
  await app.press("Plain Essay");
  blob = app.texts().join("\n");
  expect(blob).toContain("Plain Essay");
  expect(blob).not.toContain("Field Guide");
});

test("shelf grows across frames while importing", async () => {
  const app = await open(".", {
    clock: { time: new Date("2026-01-01T00:00:00Z") },
  });
  const titles = [
    "Plain Essay",
    "Field Guide",
    "Folder Book",
    "Cloud Copy",
    "Store Sample",
    "Spoken Sample",
  ];
  const count = (blob: string) => titles.filter((title) => blob.includes(title)).length;
  await app.press("Import from Apple Books");
  const early = app.texts().join("\n");
  const earlyCount = count(early);
  expect(earlyCount).toBeLessThan(titles.length);
  expect(early).not.toContain("Imported 3");
  let grew = false;
  await app.waitUntil(() => {
    const blob = app.texts().join("\n");
    if (count(blob) > earlyCount) {
      grew = true;
    }
    return (
      grew &&
      blob.includes(
        "Imported 3, 1 store-protected unavailable, 1 not downloaded from iCloud",
      )
    );
  });
});

test("calibre fixture opens from its button", async () => {
  const app = await open(".");
  await app.press("calibre library");
  const blob = app.texts().join("\n");
  expect(blob).toContain("Harbor Notes");
  expect(blob).not.toContain("Plain Essay");
});

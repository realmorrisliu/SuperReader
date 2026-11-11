import { expect, open, test } from "nsh/test";

test("denied apple import turns the button into a folder picker", async () => {
  const app = await open(".");
  expect(app.texts().join("\n")).toContain("Import from Apple Books");
  await app.press("Import from Apple Books");
  const blob = app.texts().join("\n");
  expect(blob).toContain("Choose Folder");
  expect(blob).not.toContain("Import from Apple Books");
  expect(blob).not.toContain("Plain Essay");
});

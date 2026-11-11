import { expect, open, test } from "nsh/test";

test("the shell does not offer the bare shelf picker", async () => {
  const app = await open(".");
  const blob = app.texts().join("\n");
  expect(blob).toContain("书库");
  expect(blob).not.toContain("Choose Folder");
  expect(blob).not.toContain("Import from Apple Books");
});

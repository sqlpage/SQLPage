import { expect, test } from "../../fixture.ts";

test("a JSON menu item with a link is announced as a link", async ({
  page,
}) => {
  await expect(page.getByRole("link", { name: "Reports" })).toHaveAttribute(
    "href",
    "reports.sql",
  );
  await expect(page.getByRole("button", { name: "Reports" })).toHaveCount(0);
});

test("a JSON menu item with a submenu stays a button", async ({ page }) => {
  await expect(page.getByRole("button", { name: "More" })).toHaveAttribute(
    "data-bs-toggle",
    "dropdown",
  );
});

test("a plain string menu item is still a link", async ({ page }) => {
  await expect(page.getByRole("link", { name: "about" })).toHaveAttribute(
    "href",
    "about.sql",
  );
});

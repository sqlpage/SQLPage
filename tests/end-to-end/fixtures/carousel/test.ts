import { expect, test } from "../../fixture.ts";

test("slide indicators have distinct accessible names", async ({ page }) => {
  await expect(page.getByRole("button", { name: "Slide 1" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Slide 2" })).toBeVisible();
});

test("keyboard users can switch slides using the carousel arrows", async ({
  page,
}) => {
  await expect(page.getByRole("img", { name: "First image" })).toBeVisible();
  await page.getByRole("button", { name: "Next" }).press("Enter");
  await expect(page.getByRole("img", { name: "Second image" })).toBeVisible();
  await page.getByRole("button", { name: "Previous" }).press("Space");
  await expect(page.getByRole("img", { name: "First image" })).toBeVisible();
});

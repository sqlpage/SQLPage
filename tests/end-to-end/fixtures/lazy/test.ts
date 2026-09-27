import { expect, test } from "../../fixture";

test("loads lazy content and initializes its chart", async ({ page }) => {
  await expect(
    page.getByRole("heading", { name: "Loaded lazy content" }),
  ).toBeVisible();
  await expect(page.locator("#lazy-chart .apexcharts-canvas")).toBeVisible();
  await expect(page.locator("[aria-busy=true]")).toHaveCount(0);
});

test("keeps embedded cards and iframe cards working", async ({ page }) => {
  await expect(
    page.getByRole("heading", { name: "Loaded card content" }),
  ).toBeVisible();
  await expect(
    page
      .frameLocator("iframe")
      .getByRole("heading", { name: "Loaded iframe content" }),
  ).toBeVisible();
  await expect(page.locator(".card-loading-placeholder")).toHaveCount(0);
});

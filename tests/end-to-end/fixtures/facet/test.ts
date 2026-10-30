import { expect, test } from "../../fixture.ts";

test("fragment dropdown keeps one Bootstrap instance after repeated events", async ({
  page,
}) => {
  await page
    .locator("main")
    .dispatchEvent("fragment-loaded", { bubbles: true });
  await page.evaluate(() =>
    document.dispatchEvent(new CustomEvent("fragment-loaded")),
  );
  const toggle = page
    .locator('[data-bs-toggle="dropdown"]')
    .filter({ hasText: "Constitution" });
  await toggle.click();
  await expect(page.locator(".dropdown-menu.show")).toBeVisible();
  await toggle.click();
  await expect(page.locator(".dropdown-menu.show")).toHaveCount(0);
});

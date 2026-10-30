import { expect, test } from "../../fixture.ts";

test("tracking block with a link opens it", async ({ page }) => {
  await page.locator(".tracking-block").first().click();
  await expect(page).toHaveURL(/\/tracking\/day\.sql\?day=Monday$/);
  await expect(page.getByText("Details for Monday")).toBeVisible();
});

test("tracking block without a link is not a link", async ({ page }) => {
  await expect(page.locator(".tracking-block").nth(1)).not.toHaveAttribute(
    "href",
  );
});

test("tracking block with a link looks like one without", async ({ page }) => {
  const blocks = page.locator(".tracking-block");
  const linked = await blocks.nth(0).boundingBox();
  const plain = await blocks.nth(1).boundingBox();
  expect(linked?.width).toBe(plain?.width);
  expect(linked?.height).toBe(plain?.height);
});

test("tracking block with a link keeps its tooltip", async ({ page }) => {
  await page.locator(".tracking-block").first().hover();
  await expect(page.getByRole("tooltip")).toHaveText("Monday: up");
});

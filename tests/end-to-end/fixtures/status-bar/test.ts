import { expect, loadFragment, test } from "../../fixture.ts";

test("rows render in order with icons, tooltips and links", async ({
  page,
}) => {
  const bar = page.getByRole("status", { name: "Repository status" });
  await expect(bar).toHaveAttribute("id", "repository");
  await expect(bar).toContainClass("project-status");
  await expect(bar).toHaveText(
    /Build passed\s*Branch: feature\/improve-search-index\s*12 pending changes\s*View history/,
  );

  const build = bar.getByRole("link", { name: "Build passed", exact: true });
  const history = bar.getByRole("link", { name: "View history" });
  await expect(bar.locator(".build-result")).toContainClass("text-green");
  await expect(build.locator("svg")).toBeVisible();
  const barRight = await bar.evaluate((el) => el.getBoundingClientRect().right);
  const textRight = await history.evaluate((el) => {
    const text = document.createRange();
    text.selectNodeContents(el);
    return text.getBoundingClientRect().right;
  });
  expect(barRight - textRight).toBeLessThan(32);
  await build.hover();
  await expect(page.getByRole("tooltip")).toHaveText("All checks passed");
  await history.click();
  await expect(page).toHaveURL(/#history$/);
  await expect(
    page.getByRole("heading", { name: "Build history" }),
  ).toBeInViewport();
});

test("only the fixed bar stays at the viewport bottom while scrolling", async ({
  page,
}) => {
  const fixed = page.getByRole("status", { name: "Status bar", exact: true });
  const inline = page.getByRole("status", { name: "Repository status" });
  const bottomGap = () =>
    fixed.evaluate((bar) => innerHeight - bar.getBoundingClientRect().bottom);

  await expect(fixed.getByText("Connected", { exact: true })).toBeVisible();
  await expect(
    fixed.getByRole("link", { name: "Open build history" }),
  ).toBeVisible();
  await expect(inline).toBeInViewport();
  expect(await bottomGap()).toBeCloseTo(0);
  await page
    .getByRole("heading", { name: "Build history" })
    .scrollIntoViewIfNeeded();
  await expect(inline).not.toBeInViewport();
  expect(await bottomGap()).toBeCloseTo(0);
});

test("a narrow bar scrolls to the next focused link without widening the page", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 640 });
  const bar = page.getByRole("status", { name: "Repository status" });
  const history = bar.getByRole("link", { name: "View history" });
  await expect(history).not.toBeInViewport();
  await bar.getByRole("link", { name: "Build passed" }).focus();
  await page.keyboard.press("Tab");
  await expect(history).toBeFocused();
  await expect(history).toBeInViewport({ ratio: 0.99 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(
    320,
  );
});

test("a fixed bar disappears with its embedded fragment", async ({ page }) => {
  await loadFragment(
    page,
    "/status-bar/fragment.sql",
    "#activity",
    "#activity",
  );
  const bar = page.getByRole("status", { name: "Sync status" });
  await expect(bar).toHaveText("Synced");
  await page.locator("#activity").evaluate((el) => el.remove());
  await expect(bar).toHaveCount(0);
});

import { AxeBuilder } from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const wcagTags = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];

// Existing failures in the documentation search shortcut and inline prose links.
const sharedRulesToFix = ["label-content-name-mismatch", "link-in-text-block"];

// Re-enable each rule for the affected component once its examples pass.
const componentRulesToFix: Partial<Record<string, string[]>> = {
  button: ["color-contrast", "image-alt"],
  card: ["color-contrast", "frame-title", "image-alt"],
  carousel: ["button-name"],
  code: ["scrollable-region-focusable"],
  login: ["image-alt"],
  map: ["color-contrast"],
  pagination: ["link-name"],
  table: ["scrollable-region-focusable"],
  tracking: ["aria-prohibited-attr"],
};

test("component documentation passes axe WCAG A and AA checks", {
  tag: "@accessibility",
}, async ({ page }) => {
  test.setTimeout(180_000);

  // Keep the video iframe in the audit, without loading YouTube's UI.
  await page.route("https://www.youtube.com/embed/**", (route) =>
    route.fulfill({ status: 204 }),
  );

  const catalog = await page.goto("/components.sql");
  expect(catalog?.ok()).toBe(true);
  const links = page.locator('#catalog a[href^="/component.sql?component="]');
  await expect(links.first()).toBeVisible();
  const urls = await links.evaluateAll((anchors) =>
    anchors.map((anchor) => (anchor as HTMLAnchorElement).href),
  );

  for (const url of new Set(urls)) {
    const component = new URL(url).searchParams.get("component") ?? "";
    await test.step(component, async () => {
      const response = await page.goto(url);
      expect(response?.ok()).toBe(true);
      await expect(
        page.getByRole("heading", {
          name: `The ${component} component`,
          level: 1,
          exact: true,
        }),
      ).toBeVisible();

      const { violations } = await new AxeBuilder({ page })
        .withTags(wcagTags)
        .disableRules([
          ...sharedRulesToFix,
          ...(componentRulesToFix[component] ?? []),
        ])
        .analyze();

      expect
        .soft(
          violations.map(({ id, helpUrl, nodes }) => ({
            id,
            helpUrl,
            nodes: nodes.map(({ target, failureSummary }) => ({
              target,
              failureSummary,
            })),
          })),
          `${component} documentation accessibility violations`,
        )
        .toEqual([]);
    });
  }
});

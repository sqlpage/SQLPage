import { AxeBuilder } from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const wcagTags = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];

// Existing failures in the documentation search shortcut and inline prose links.
const sharedRulesToFix = ["label-content-name-mismatch", "link-in-text-block"];

// Re-enable each rule for the affected component once its examples pass.
const componentRulesToFix: Partial<Record<string, string[]>> = {
  button: ["color-contrast", "image-alt"],
  card: ["color-contrast", "frame-title", "image-alt"],
  code: ["scrollable-region-focusable"],
  login: ["image-alt"],
  map: ["color-contrast"],
  table: ["scrollable-region-focusable"],
};

const stockThemes = ["light", "dark"] as const;
type StockTheme = (typeof stockThemes)[number];

// Keep the rules active so additional failing elements exceed the baseline.
const stockBaseline: Partial<
  Record<string, Record<string, Partial<Record<StockTheme, number>>>>
> = {
  alert: {
    "color-contrast": { light: 7 },
    "link-in-text-block": { light: 1, dark: 1 },
  },
  big_number: { "color-contrast": { light: 6, dark: 4 } },
  button: {
    "color-contrast": { light: 4, dark: 5 },
    "image-alt": { light: 1, dark: 1 },
  },
  card: {
    "color-contrast": { light: 6, dark: 7 },
    "frame-title": { light: 1, dark: 1 },
    "image-alt": { light: 3, dark: 3 },
  },
  datagrid: { "color-contrast": { light: 3, dark: 2 } },
  divider: { "color-contrast": { dark: 1 } },
  foldable: { "link-in-text-block": { light: 1, dark: 1 } },
  list: { "color-contrast": { light: 1 } },
  login: {
    "image-alt": { light: 1, dark: 1 },
    "link-in-text-block": { dark: 1 },
  },
  map: { "color-contrast": { dark: 3 } },
  modal: { "color-contrast": { light: 2, dark: 1 } },
  status_bar: { "color-contrast": { light: 1 } },
  steps: { "color-contrast": { light: 2, dark: 2 } },
  tab: { "color-contrast": { dark: 2 } },
  table: {
    "color-contrast": { light: 13, dark: 1 },
    "scrollable-region-focusable": { light: 1, dark: 1 },
  },
  text: { "link-in-text-block": { light: 4, dark: 4 } },
  timeline: { "link-in-text-block": { light: 1, dark: 1 } },
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

  const liveExamples = ".docs-example-output";
  for (const href of new Set(urls)) {
    const url = new URL(href);
    const component = url.searchParams.get("component") ?? "";
    let previewCount = 0;

    for (const theme of ["custom-docsite", ...stockThemes] as const) {
      const stockTheme = theme === "custom-docsite" ? undefined : theme;
      if (stockTheme) url.searchParams.set("theme", stockTheme);
      await test.step(`${component} (${theme})`, async () => {
        const response = await page.goto(url.href);
        expect(response?.ok()).toBe(true);
        await expect(
          page.getByRole("heading", {
            name: `The ${component} component`,
            level: 1,
            exact: true,
          }),
        ).toBeVisible();

        const axe = new AxeBuilder({ page }).withTags(wcagTags);
        const previews = page.locator(liveExamples);
        if (stockTheme) {
          const html = page.locator("html");
          await expect(html).toHaveAttribute("data-bs-theme", stockTheme);
          await expect(html).not.toHaveAttribute(
            "data-site-theme",
            "custom-docsite",
          );
          await expect(
            page.locator(
              'link[href="/assets/highlightjs-and-tabler-theme.css"]',
            ),
          ).toHaveCount(0);
          await expect(previews).toHaveCount(previewCount);
          if (previewCount === 0) return;
          axe.include(liveExamples);
        } else {
          previewCount = await previews.count();
          axe.disableRules([
            ...sharedRulesToFix,
            ...(componentRulesToFix[component] ?? []),
          ]);
        }

        const { violations } = await axe.analyze();
        const regressions = violations
          .map(({ id, helpUrl, nodes }) => ({
            id,
            helpUrl,
            baseline: stockTheme
              ? (stockBaseline[component]?.[id]?.[stockTheme] ?? 0)
              : 0,
            nodes: nodes.map(({ target, failureSummary }) => ({
              target,
              failureSummary,
            })),
          }))
          .filter(({ nodes, baseline }) => nodes.length > baseline);
        expect
          .soft(regressions, `${component} (${theme}) accessibility violations`)
          .toEqual([]);
      });
    }
  }
});

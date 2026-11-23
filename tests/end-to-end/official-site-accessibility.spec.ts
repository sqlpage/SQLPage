import { AxeBuilder } from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const wcagTags = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];

const stockThemes = ["light", "dark"] as const;
const themes = ["custom-docsite", ...stockThemes] as const;
type Theme = (typeof themes)[number];

// Keep color-contrast active and allow only the observed counts per component
// and theme. All other rules still require zero violations.
const upstreamContrast: Record<string, Partial<Record<Theme, number>>> = {
  // Tabler's stock text, status, alert, step and table colors.
  // https://github.com/tabler/tabler/issues/3630
  alert: { light: 7 },
  big_number: { light: 6, dark: 4 },
  datagrid: { light: 3, dark: 2 },
  list: { light: 1 },
  modal: { light: 2, dark: 1 },
  status_bar: { light: 1 },
  steps: { light: 2, dark: 2 },
  table: { light: 13, dark: 1 },
  // Tabler's outline labels and accent text on dark surfaces.
  // https://github.com/tabler/tabler/issues/3359
  button: { light: 3, dark: 4 },
  divider: { dark: 1 },
  tab: { dark: 2 },
  // axe uses Tabler's masked card stripe as the background for card text.
  // https://github.com/dequelabs/axe-core/issues/5573
  card: { "custom-docsite": 7, light: 6, dark: 7 },
  // Leaflet's attribution links over its translucent attribution background.
  // https://github.com/Leaflet/Leaflet/issues/7538
  map: { "custom-docsite": 3, dark: 3 },
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

    for (const theme of themes) {
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
        }

        const { violations } = await axe.analyze();
        const failures = violations
          .filter(
            ({ id, nodes }) =>
              id !== "color-contrast" ||
              nodes.length > (upstreamContrast[component]?.[theme] ?? 0),
          )
          .map(({ id, helpUrl, nodes }) => ({
            id,
            helpUrl,
            nodes: nodes.map(({ target, failureSummary }) => ({
              target,
              failureSummary,
            })),
          }));
        expect
          .soft(failures, `${component} (${theme}) accessibility violations`)
          .toEqual([]);
      });
    }
  }
});

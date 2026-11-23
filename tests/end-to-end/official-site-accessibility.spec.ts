import { AxeBuilder } from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const wcagTags = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];

const stockThemes = ["light", "dark"] as const;

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
        }

        const { violations } = await axe.analyze();
        const failures = violations.map(({ id, helpUrl, nodes }) => ({
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

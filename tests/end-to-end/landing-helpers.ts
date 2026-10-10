import { expect, type Page } from "@playwright/test";

/** Load the lazy example through normal scrolling before recording page geometry. */
export async function settleLandingPreview(page: Page) {
  const preview = page.locator(
    '.demo-preview iframe[title="Live SQLPage component demo"]',
  );
  await preview.scrollIntoViewIfNeeded();
  await expect(
    page
      .frameLocator('iframe[title="Live SQLPage component demo"]')
      .getByRole("cell", { name: "Acme Corp" }),
  ).toBeVisible();
  await preview.evaluate(async (frame: HTMLIFrameElement) => {
    await frame.contentDocument!.fonts.ready;
  });
  await expect
    .poll(() =>
      preview.evaluate((frame: HTMLIFrameElement) =>
        Math.abs(
          frame.parentElement!.getBoundingClientRect().height -
            frame
              .contentDocument!.querySelector("#sqlpage_main_wrapper")!
              .getBoundingClientRect().height -
            2,
        ),
      ),
    )
    .toBeLessThan(1);
  await page.evaluate(() => window.scrollTo(0, 0));
}

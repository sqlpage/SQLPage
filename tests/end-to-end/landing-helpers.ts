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

/** Wait for the real scroll event before advancing controlled animation frames. */
export async function scrollLanding(page: Page, top: number) {
  await page.evaluate(
    (top) =>
      new Promise<void>((resolve) => {
        const done = () => {
          window.removeEventListener("scroll", done);
          resolve();
        };
        window.addEventListener("scroll", done, { once: true });
        const before = scrollY;
        window.scrollTo({ top, behavior: "instant" });
        if (scrollY === before) done();
      }),
    top,
  );
  await page.clock.runFor(32);
}

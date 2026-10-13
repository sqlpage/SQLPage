import { expect, test } from "@playwright/test";

for (const settings of [
  { width: 1440, height: 1000, reducedMotion: "no-preference", visible: 2 },
  { width: 768, height: 1024, reducedMotion: "reduce", visible: 1 },
  { width: 390, height: 844, reducedMotion: "reduce", visible: 1 },
] as const) {
  test(`landing comparison browses all alternatives at ${settings.width}px`, async ({
    page,
  }) => {
    await page.setViewportSize(settings);
    await page.emulateMedia({ reducedMotion: settings.reducedMotion });
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto("/");
    expect(
      await page.locator(".landing-section").evaluateAll((sections) => {
        const comparison = sections.findIndex(
          (section) => section.id === "compare",
        );
        const databases = sections.findIndex((section) =>
          section.classList.contains("databases-section"),
        );
        return comparison + 1 === databases;
      }),
    ).toBe(true);
    const section = page.locator("#compare");
    const track = section.getByRole("region");
    const previous = section.getByRole("button", {
      name: "Previous alternative",
    });
    const next = section.getByRole("button", { name: "Next alternative" });
    const status = section.getByRole("status");
    await next.scrollIntoViewIfNeeded();
    await expect(previous).toBeDisabled();
    await expect(status).toHaveText(
      settings.visible === 2 ? "1–2 of 5 alternatives" : "1 of 5 alternatives",
    );
    const sqlpage = section.locator(".comparison-sqlpage");
    const initialHeight = await section.evaluate(
      (section) => section.clientHeight,
    );
    for (let index = 1; index <= 5 - settings.visible; index++) {
      await next.click();
      await expect(status).toHaveText(
        settings.visible === 2
          ? `${index + 1}–${index + 2} of 5 alternatives`
          : `${index + 1} of 5 alternatives`,
      );
      await expect(
        sqlpage.getByRole("heading", { name: "SQLPage", exact: true }),
      ).toBeVisible();
    }
    await expect(next).toBeDisabled();
    await expect(previous).toBeEnabled();
    // The final card is wholly readable, and cycling never changes section geometry.
    const last = track.locator(".comparison-card").last();
    expect(
      await last.evaluate((card) => {
        const bounds = card.getBoundingClientRect();
        const track = card.parentElement?.getBoundingClientRect();
        if (!track) throw new Error("Missing alternatives track");
        return bounds.left >= track.left - 1 && bounds.right <= track.right + 1;
      }),
    ).toBe(true);
    expect(await section.evaluate((section) => section.clientHeight)).toBe(
      initialHeight,
    );
    await track.focus();
    await track.press("Home");
    await expect(previous).toBeDisabled();
    await track.press("ArrowLeft");
    await expect(previous).toBeDisabled();
    await track.press("ArrowRight");
    await expect(previous).toBeEnabled();
    await track.press("End");
    await expect(next).toBeDisabled();
    await track.press("ArrowRight");
    await expect(next).toBeDisabled();
    await track.press("ArrowLeft");
    await expect(next).toBeEnabled();
    await previous.click();
    await expect(next).toBeEnabled();
    // Resize the same page after browsing; scroll bounds and controls update together.
    await page.setViewportSize({ width: 1280, height: 900 });
    await track.focus();
    await track.press("End");
    await expect(status).toHaveText("4–5 of 5 alternatives");
    await expect(next).toBeDisabled();
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    expect(errors).toEqual([]);
  });
}

test("landing comparison remains browsable without JavaScript", async ({
  browser,
  baseURL,
}) => {
  const context = await browser.newContext({
    baseURL,
    javaScriptEnabled: false,
    viewport: { width: 390, height: 844 },
  });
  const page = await context.newPage();
  await page.goto("/");
  const section = page.locator("#compare");
  await section.scrollIntoViewIfNeeded();
  await expect(
    section.getByRole("group", { name: "Browse alternatives" }),
  ).toBeHidden();
  const track = section.getByRole("region");
  await track.scrollIntoViewIfNeeded();
  await track.hover();
  await page.mouse.wheel(3000, 0);
  await expect
    .poll(() => track.evaluate((track) => track.scrollLeft))
    .toBeGreaterThan(0);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await expect(
    section.getByRole("heading", { name: "Low-code platform", exact: true }),
  ).toBeVisible();
  await context.close();
});

import { expect, type Locator, test } from "@playwright/test";
import { scrollLanding, settleLandingPreview } from "./landing-helpers.ts";

async function bounds(element: Locator) {
  const rect = await element.boundingBox();
  if (!rect) throw new Error("Expected visible mobile content");
  return rect;
}

async function expectContentFits(pageContent: Locator) {
  const overflow = await pageContent.evaluate((root) => {
    const selectors = [
      ".section-inner h2",
      ".purpose-intro",
      ".purpose-grid",
      ".stack-list li",
      ".benefit",
      ".frontend-tabs button",
      ".frontend-panels",
      ".feature-node",
      ".database-grid a",
      ".primary-button",
      ".landing-footer nav a",
    ].join(",");
    return [...root.querySelectorAll<HTMLElement>(selectors)]
      .filter((element) => !element.closest("[hidden]"))
      .filter((element) => {
        const rect = element.getBoundingClientRect();
        return (
          rect.left < -1 ||
          rect.right > innerWidth + 1 ||
          element.scrollWidth > element.clientWidth + 1
        );
      })
      .map((element) => element.textContent?.trim().replace(/\s+/g, " "));
  });
  expect(
    overflow,
    "visible content fits without clipping or horizontal scrolling",
  ).toEqual([]);
}

test("phone layouts have balanced gutters, readable cards, and usable controls", async ({
  page,
}) => {
  test.setTimeout(180_000);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-enhanced",
    "true",
  );
  const content = page.locator(".sqlpage-world");
  for (const width of [320, 375, 390, 430, 600]) {
    await page.setViewportSize({ width, height: 844 });
    await expect
      .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
      .toBe(width);
    for (const selector of [
      ".purpose-intro",
      ".purpose-grid",
      ".database-grid",
    ]) {
      const element = page.locator(selector);
      const inner = await element.locator("..").boundingBox();
      if (!inner) throw new Error("Expected a section with mobile gutters");
      const rect = await bounds(element);
      expect(rect.x, selector).toBeCloseTo(inner.x, 0);
      expect(rect.x + rect.width, selector).toBeCloseTo(
        inner.x + inner.width,
        0,
      );
      expect(rect.x, selector).toBeCloseTo(width - rect.x - rect.width, 0);
    }
    const benefits = page.locator(".benefit");
    for (const benefit of await benefits.all()) {
      expect((await bounds(benefit)).width).toBeCloseTo(
        (await bounds(page.locator(".benefit-grid"))).width,
        0,
      );
    }
    await expectContentFits(content);
    const panels = page.locator(".frontend-panels");
    for (const name of [
      "Make it yours",
      "Ship it anywhere",
      "Safe by default",
      "Instant",
    ]) {
      await page.getByRole("tab", { name }).click();
      if (name === "Make it yours") {
        const diagram = page.locator(".customize-diagram");
        const result = diagram.locator(".feature-result");
        const sources = diagram.locator(".feature-node:not(.feature-result)");
        const sourceBounds = await Promise.all(
          (await sources.all()).map(bounds),
        );
        const resultBounds = await bounds(result);
        expect(resultBounds.y).toBeGreaterThan(
          Math.max(...sourceBounds.map((rect) => rect.y + rect.height)),
        );
        expect(resultBounds.width).toBeCloseTo(
          (await bounds(diagram)).width,
          0,
        );
        await expect(result.locator("br")).toHaveCSS("display", "none");
      }
      if (name === "Ship it anywhere") {
        const diagram = page.locator(".ship-diagram");
        const source = await bounds(diagram.locator(":scope > .feature-node"));
        const options = await bounds(diagram.locator(".ship-options"));
        expect(options.y).toBeGreaterThan(source.y + source.height);
        await expect(diagram.locator(":scope > b")).toHaveCSS(
          "rotate",
          "90deg",
        );
      }
      const whitespace = await panels.evaluate((element) => {
        const panel = element.querySelector(".frontend-panel:not([hidden])")!;
        const bottom = Math.max(
          ...[...panel.children].map(
            (child) => child.getBoundingClientRect().bottom,
          ),
        );
        return (
          element.getBoundingClientRect().bottom -
          bottom -
          parseFloat(getComputedStyle(element).paddingBottom)
        );
      });
      expect(
        whitespace,
        `content-sized card at ${width}px`,
      ).toBeLessThanOrEqual(1);
      await expectContentFits(content);
    }
    await page
      .getByRole("button", { name: "A SQLPage app", exact: true })
      .click();
    await expectContentFits(content);
    await page
      .getByRole("button", { name: "A typical webapp", exact: true })
      .click();
    for (const control of await page
      .locator(
        ".segmented button, .sql-editor-header button:not([hidden]), .primary-button, .landing-footer nav a",
      )
      .all()) {
      expect((await bounds(control)).height).toBeGreaterThanOrEqual(44);
    }
    const demo = page.frameLocator(
      'iframe[title="Live SQLPage component demo"]',
    );
    await expect(demo.getByRole("searchbox")).toHaveCSS("font-size", "16px");
    await page.getByRole("tab", { name: "form Fields from rows" }).click();
    await expect(demo.getByLabel("Your name")).toHaveCSS("font-size", "16px");
    await expect(demo.getByLabel("Your team")).toHaveCSS("font-size", "16px");
    await page
      .getByRole("tab", { name: "table Sortable and searchable" })
      .click();
  }
});

test("tablet and landscape layouts keep every frontend view inside the page", async ({
  page,
}) => {
  test.setTimeout(120_000);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  for (const viewport of [
    { width: 768, height: 1024 },
    { width: 844, height: 390 },
  ]) {
    await page.setViewportSize(viewport);
    for (const name of [
      "Make it yours",
      "Ship it anywhere",
      "Safe by default",
      "Instant",
    ]) {
      await page.getByRole("tab", { name }).click();
      await expectContentFits(page.locator(".sqlpage-world"));
    }
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth),
    ).toBe(viewport.width);
  }
});

test("small phones can use navigation and every live component preview", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 568 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await page.getByRole("button", { name: "Open navigation" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  const rect = await bounds(dialog);
  expect(rect.x).toBeGreaterThanOrEqual(0);
  expect(rect.x + rect.width).toBeLessThanOrEqual(320);
  for (const link of await dialog.getByRole("link").all()) {
    expect((await bounds(link)).height).toBeGreaterThanOrEqual(44);
  }
  await page.getByRole("button", { name: "Close navigation" }).click();
  await expect(dialog).not.toBeVisible();
  const preview = page.frameLocator(
    'iframe[title="Live SQLPage component demo"]',
  );
  for (const component of ["chart", "big_number", "catalog", "form", "table"]) {
    await page.locator(`#tab-${component}`).click();
    await expect(page.locator("#demo-panel")).toHaveAttribute(
      "data-demo",
      component,
    );
    await expect(page.locator("#demo-panel")).not.toHaveAttribute(
      "aria-busy",
      "true",
    );
    await expect(preview.locator("#sqlpage_main_wrapper")).toBeVisible();
    await expect
      .poll(() =>
        preview
          .locator("html")
          .evaluate((root) => root.scrollWidth - root.clientWidth),
      )
      .toBeLessThanOrEqual(0);
  }
});

test("purpose copy clears the rotating sculpture on phones and tablets", async ({
  page,
}) => {
  test.setTimeout(90_000);
  await page.setViewportSize({ width: 390, height: 1024 });
  await page.clock.install();
  await page.goto("/");
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-scene",
    "ready",
    { timeout: 30_000 },
  );
  await settleLandingPreview(page);
  await page.clock.pauseAt(await page.evaluate(() => Date.now() + 60_000));

  for (const width of [390, 507, 600, 768, 900]) {
    await page.setViewportSize({ width, height: 1024 });
    await page.clock.runFor(64);
    await scrollLanding(page, 0);
    const top = await page.locator(".purpose-section").evaluate((section) => {
      let top = 0;
      for (
        let element: HTMLElement | null = section as HTMLElement;
        element;
        element = element.offsetParent as HTMLElement | null
      )
        top += element.offsetTop;
      return top;
    });
    await scrollLanding(page, top);
    await expect(page.locator(".scene-layer")).toHaveCSS("opacity", "1");

    // The complete canvas contains every rotated pose, so text below its
    // bottom stays readable throughout rotation rather than at one angle.
    await expect
      .poll(() =>
        page.evaluate(() => {
          const paragraph = document.querySelector(
            ".purpose-intro > p:first-child",
          )!;
          const canvas = document.querySelector("canvas")!;
          return (
            paragraph.getBoundingClientRect().top -
            canvas.getBoundingClientRect().bottom
          );
        }),
      )
      .toBeGreaterThanOrEqual(16);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth),
      `page fits at ${width}px`,
    ).toBe(width);
  }
});

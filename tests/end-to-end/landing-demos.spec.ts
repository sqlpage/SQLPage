import { expect, test } from "@playwright/test";

for (const settings of [
  { width: 1280, height: 720, reducedMotion: "no-preference" },
  { width: 390, height: 844, reducedMotion: "no-preference" },
  { width: 390, height: 844, reducedMotion: "reduce" },
] as const) {
  test(`landing examples stay in sync at ${settings.width}px with ${settings.reducedMotion} motion`, async ({
    page,
  }) => {
    test.setTimeout(60_000);
    await page.setViewportSize(settings);
    await page.emulateMedia({ reducedMotion: settings.reducedMotion });
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    // Sample UI motion without software WebGL competing for animation frames.
    // The real sculpture and its integration are covered in landing.spec.ts.
    await page.route("**/assets/landing/js/scene/database-scene.js", (route) =>
      route.abort(),
    );
    await page.goto("/");
    await page.locator("#components").scrollIntoViewIfNeeded();
    const panel = page.locator("#demo-panel");
    const source = page.locator("#demo-source");
    const demo = page.frameLocator(
      'iframe[title="Live SQLPage component demo"]',
    );
    await expect(demo.getByRole("cell", { name: "Acme Corp" })).toBeVisible();
    const tableSource = await source.textContent();

    // Hold the source request: the current working example must stay visible.
    let release = () => {};
    const held = new Promise<void>((resolve) => {
      release = resolve;
    });
    await page.route(
      "**/landing-demos/source.sql?component=form",
      async (route) => {
        await held;
        await route.continue();
      },
    );
    await page.getByRole("tab", { name: "form Fields from rows" }).click();
    await expect(panel).toHaveAttribute("aria-busy", "true");
    await expect(demo.getByRole("cell", { name: "Acme Corp" })).toBeVisible();
    await expect(source).toHaveText(tableSource!);
    await expect(
      page.getByRole("tab", { name: "table.sql", exact: true }),
    ).toBeVisible();

    // Sample actual layout while the loaded preview and its full SQL replace it.
    const samples = panel.evaluate(
      (element) =>
        new Promise<{ height: number; scroll: number; loading: boolean }[]>(
          (resolve) => {
            const frames: {
              height: number;
              scroll: number;
              loading: boolean;
            }[] = [];
            const start = performance.now();
            const sample = () => {
              frames.push({
                height: element.getBoundingClientRect().height,
                scroll: scrollY,
                loading: element
                  .querySelector("#demo-source")!
                  .textContent!.includes("Loading source"),
              });
              if (performance.now() - start < 1000)
                requestAnimationFrame(sample);
              else resolve(frames);
            };
            sample();
          },
        ),
    );
    release();
    await expect(demo.getByLabel("Your name")).toBeVisible();
    await expect(source).toContainText("/landing-demos/save.sql");
    await expect(panel).not.toHaveAttribute("aria-busy", "true");
    const frames = await samples;
    expect(frames.some((frame) => frame.loading)).toBe(false);
    const heights = frames.map((frame) => frame.height);
    expect(Math.min(...heights)).toBeGreaterThanOrEqual(
      Math.min(heights[0], heights.at(-1)!) - 2,
    );
    const scrolls = frames.map((frame) => frame.scroll);
    expect(Math.max(...scrolls) - Math.min(...scrolls)).toBeLessThan(2);
    if (settings.reducedMotion === "no-preference") {
      const distance = Math.abs(heights.at(-1)! - heights[0]);
      const steps = heights
        .slice(1)
        .map((height, index) => Math.abs(height - heights[index]));
      expect(Math.max(...steps)).toBeLessThan(Math.max(12, distance * 0.75));
    }

    // A slow, superseded request must never overwrite the latest selection.
    let releaseChart = () => {};
    const heldChart = new Promise<void>((resolve) => {
      releaseChart = resolve;
    });
    await page.route(
      "**/landing-demos/source.sql?component=chart",
      async (route) => {
        await heldChart;
        await route.continue();
      },
    );
    await page
      .getByRole("tab", { name: "chart A timeline from 3 rows", exact: true })
      .click();
    await expect(panel).toHaveAttribute("aria-busy", "true");
    const numbers = page.getByRole("tab", {
      name: "big_number KPIs at a glance",
      exact: true,
    });
    await numbers.evaluate((button) => {
      button.addEventListener(
        "pointerdown",
        () => {
          (button as HTMLElement).dataset.pointerScroll = String(scrollY);
        },
        { once: true },
      );
      document.addEventListener(
        "pointerup",
        () => {
          (button as HTMLElement).dataset.pointerShift = String(
            scrollY - Number((button as HTMLElement).dataset.pointerScroll),
          );
        },
        { once: true },
      );
    });
    await numbers.click();
    await expect(numbers).toHaveAttribute("data-pointer-shift", "0");
    await expect(demo.getByText("98%", { exact: true })).toBeVisible();
    releaseChart();
    await expect(source).toContainText("'big_number' as component");
    await expect(panel).toHaveAttribute("data-demo", "big_number");
    await expect(page.locator("#demo-panel iframe")).toHaveCount(1);
    await numbers.press("Home");
    await expect(page.locator("#tab-chart")).toBeFocused();
    await expect(demo.locator(".apexcharts-canvas")).toBeVisible();
    await expect(source).toContainText("from tickets");
    expect(errors).toEqual([]);
  });
}

test("a failed landing example keeps the current preview and SQL", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await page.locator("#components").scrollIntoViewIfNeeded();
  const demo = page.frameLocator('iframe[title="Live SQLPage component demo"]');
  await expect(demo.getByRole("cell", { name: "Acme Corp" })).toBeVisible();
  await page.route("**/landing-demos/source.sql?component=chart", (route) =>
    route.fulfill({ status: 503 }),
  );
  await page
    .getByRole("tab", { name: "chart A timeline from 3 rows", exact: true })
    .click();
  await expect(page.getByRole("status")).toHaveText(
    "The demo could not load. Please try again.",
  );
  await expect(demo.getByRole("cell", { name: "Acme Corp" })).toBeVisible();
  await expect(page.locator("#demo-source")).toContainText("from customers");
  await expect(page.locator("#tab-table")).toHaveAttribute(
    "aria-selected",
    "true",
  );
  await expect(page.locator("#demo-panel iframe")).toHaveCount(1);
});

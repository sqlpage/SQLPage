import { expect, test } from "@playwright/test";

test("landing page: live components, deployment, scrolling sculpture and mobile navigation", async ({
  page,
}) => {
  test.setTimeout(90_000);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("SELECT");
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-scene",
    "ready",
    { timeout: 30_000 },
  );
  await expect(page.locator("canvas")).toHaveCount(1);

  // The pinned introduction fades around a full-screen sculpture before docking.
  const initialSize = (await page.locator("canvas").boundingBox())!.height;
  await page.evaluate(() => {
    const hero = document.querySelector<HTMLElement>(".experience")!;
    const viewport = hero.querySelector<HTMLElement>(".viewport")!;
    window.scrollTo(0, (hero.offsetHeight - viewport.offsetHeight) * 0.76);
  });
  await expect
    .poll(async () => (await page.locator("canvas").boundingBox())!.height)
    .toBeGreaterThan(initialSize * 1.2);
  await expect
    .poll(() =>
      page
        .locator(".hero-copy span")
        .first()
        .evaluate((element) => Number(getComputedStyle(element).opacity)),
    )
    .toBeLessThan(0.05);

  const demo = page.frameLocator('iframe[title="Live SQLPage component demo"]');
  await page.locator("#components").scrollIntoViewIfNeeded();
  await expect(demo.getByRole("cell", { name: "Acme Corp" })).toBeVisible();
  await demo.getByRole("searchbox").fill("Globex");
  await expect(demo.getByRole("cell", { name: "Globex" })).toBeVisible();
  await expect(demo.getByRole("cell", { name: "Acme Corp" })).toBeHidden();
  await demo.getByRole("searchbox").clear();
  await demo.getByRole("button", { name: "seats", exact: true }).click();
  await expect(demo.locator("tbody tr").first()).toContainText("Initech");
  await expect(page.locator("#demo-source")).toContainText("from customers");
  await expect(
    page.locator("#demo-source .hljs-keyword").first(),
  ).toBeVisible();

  await page.getByRole("tab", { name: "chart", exact: false }).click();
  await expect(demo.locator(".apexcharts-canvas")).toBeVisible();
  await expect(page.locator("#demo-source")).toContainText("from tickets");
  // Arrow navigation selects the next component and keeps the active tab focusable.
  await page
    .getByRole("tab", { name: "chart", exact: false })
    .press("ArrowDown");
  await page.getByRole("tab", { name: "form Fields from rows" }).click();
  await demo.getByLabel("Your name").fill("Ada");
  await demo.getByLabel("Your team").fill("SQL builders");
  await demo.getByRole("button", { name: "Submit" }).click();
  await expect(demo.getByRole("alert")).toContainText("Hello, Ada!");
  await page.getByRole("tab", { name: "big_number" }).click();
  await expect(demo.getByText("98%", { exact: true })).toBeVisible();
  await expect
    .poll(async () => (await page.locator("iframe").boundingBox())!.height)
    .toBeLessThan(220);

  await page
    .getByRole("button", { name: "A SQLPage app", exact: true })
    .click();
  await expect(page.locator(".stack-comparison")).toHaveAttribute(
    "data-stack",
    "sqlpage",
  );
  await page.getByRole("tab", { name: "Managed hosting" }).click();
  await expect(
    page.getByRole("link", { name: "Explore managed hosting" }),
  ).toBeVisible();
  await page.getByRole("tab", { name: "Run it on your server" }).click();
  await expect(page.locator("#server-panel")).toContainText("sqlpage");

  // Each measured anchor owns the same canvas, including after responsive reflow.
  for (const stop of await page.locator("[data-scene-stop]").all()) {
    await stop.evaluate((element) =>
      window.scrollTo(
        0,
        element.closest("section")!.offsetTop - innerHeight * 0.12,
      ),
    );
    await expect
      .poll(async () => {
        const target = await stop.boundingBox();
        const canvas = await page.locator("canvas").boundingBox();
        return target && canvas
          ? Math.max(
              Math.abs(canvas.x + canvas.width * 0.291366 - target.x),
              Math.abs(canvas.y + canvas.height * 0.224284 - target.y),
              Math.abs(canvas.width * 0.393678 - target.width),
            )
          : Infinity;
      })
      .toBeLessThan(2);
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.evaluate(() => window.scrollTo(0, 0));
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-motion",
    "off",
  );
  await page.getByRole("button", { name: "Open navigation" }).click();
  const navigation = page.getByRole("dialog", { name: "SQLPage" });
  await expect(
    navigation.getByRole("link", { name: "Documentation" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(navigation).toBeHidden();
  await expect(
    page.getByRole("button", { name: "Open navigation" }),
  ).toBeFocused();
  await page.locator("#components").scrollIntoViewIfNeeded();
  await expect(demo.getByText("98%", { exact: true })).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  expect(errors).toEqual([]);
});

import { expect, test } from "@playwright/test";
import { settleLandingPreview } from "./landing-helpers.ts";

for (const viewport of [
  { width: 1280, height: 720 },
  { width: 390, height: 844 },
]) {
  test(`landing sculpture: page anchors, full-body visibility and spin at ${viewport.width}px`, async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.setViewportSize(viewport);
    await page.clock.install();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1 })).toContainText(
      "SELECT",
    );
    await expect(page.locator(".sqlpage-world")).toHaveAttribute(
      "data-scene",
      "ready",
      { timeout: 30_000 },
    );
    await expect(page.locator("canvas")).toHaveCount(1);
    await settleLandingPreview(page);
    // Render each sampled pose without spending software GPU time on idle frames.
    await page.clock.pauseAt(await page.evaluate(() => Date.now() + 60_000));
    await page.clock.runFor(64);

    // The pinned introduction fades around a full-screen sculpture before docking.
    const initialSize = (await page.locator("canvas").boundingBox())!.height;
    await page.evaluate(() => {
      const hero = document.querySelector<HTMLElement>(".experience")!;
      const viewport = hero.querySelector<HTMLElement>(".viewport")!;
      window.scrollTo(0, (hero.offsetHeight - viewport.offsetHeight) * 0.76);
    });
    await page.clock.runFor(64);
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

    // After the opening flight, each section holds a fixed page position and size while
    // scrolling rotates the sculpture. Handoffs hide relocation between anchors.
    async function sample(scroll: number) {
      await page.evaluate((target) => {
        window.scrollTo(0, Math.round(target));
      }, scroll);
      await page.clock.runFor(64);
      await expect
        .poll(() =>
          page
            .locator("canvas")
            .evaluate((canvas) =>
              Math.abs(
                Number(canvas.dataset.scrollTurn) -
                  (scrollY / innerHeight) * Math.PI * 2,
              ),
            ),
        )
        .toBeLessThan(0.005);
      return page.locator("canvas").evaluate((canvas) => {
        const rect = canvas.getBoundingClientRect();
        return {
          x: rect.x + scrollX,
          y: rect.y + scrollY,
          size: rect.width,
          turn: Number(canvas.dataset.scrollTurn),
          opacity: Number(
            getComputedStyle(canvas.closest(".scene-layer")!).opacity,
          ),
        };
      });
    }
    function expectSamePageFrame(
      pose: { x: number; y: number; size: number },
      anchor: { x: number; y: number; size: number },
    ) {
      expect(Math.abs(pose.x - anchor.x)).toBeLessThan(2);
      expect(Math.abs(pose.y - anchor.y)).toBeLessThan(2);
      expect(Math.abs(pose.size - anchor.size)).toBeLessThan(2);
    }
    async function expectAtPageAnchor(
      pose: { x: number; y: number; size: number },
      index: number,
    ) {
      const target = await page
        .locator("[data-scene-stop]")
        .nth(index)
        .evaluate((stop) => {
          const rect = stop.getBoundingClientRect();
          return {
            x: rect.x + scrollX,
            y: rect.y + scrollY,
            width: rect.width,
          };
        });
      // Project the visible sculpture bounds, not the surrounding square canvas.
      expect(Math.abs(pose.x + pose.size * 0.291366 - target.x)).toBeLessThan(
        2,
      );
      expect(
        Math.abs(pose.y + pose.size * 0.224284 - target.y),
        `page anchor ${index}: ${JSON.stringify({ pose, target })}`,
      ).toBeLessThan(2);
      expect(Math.abs(pose.size * 0.393678 - target.width)).toBeLessThan(2);
    }
    async function checkAnchors() {
      const height = page.viewportSize()!.height;
      // Read authored positions from the page top, before sheets start holding.
      await sample(0);
      const route = await page
        .locator("[data-scene-stop]")
        .evaluateAll((stops) =>
          stops.map((stop) => {
            const rect = stop.getBoundingClientRect();
            const section = stop.closest<HTMLElement>(".landing-section")!;
            const sectionTop = section.getBoundingClientRect().top;
            return {
              top: rect.top + scrollY,
              height: (rect.width * (0.803486 - 0.224284)) / 0.393678,
              heldTop:
                rect.top -
                sectionTop +
                Number.parseFloat(
                  section.style.getPropertyValue("--section-stick-top"),
                ),
            };
          }),
        );
      for (let index = 0; index < route.length; index++) {
        const target = route[index];
        const anchor = await sample(target.top - height * 0.25 + 1);
        await expectAtPageAnchor(anchor, index);
        expect(anchor.opacity).toBe(1);
        if (index === route.length - 1) continue;
        // Reaching the top is not a reason to remove an otherwise visible model.
        // This was the regression that left a large empty space beside headings.
        const reading = await sample(target.top - height * 0.04);
        expectSamePageFrame(reading, anchor);
        expect(reading.opacity).toBe(1);
        expect(reading.turn).toBeGreaterThan(anchor.turn);
        const atTop = await sample(target.top + 1);
        await expectAtPageAnchor(atTop, index);
        expect(atTop.opacity).toBe(1);

        // Where the whole sculpture can scroll out before the next anchor enters,
        // only its last visible sliver fades, with exactly reversible behavior.
        if (
          target.heldTop + target.height <= 0 &&
          route[index + 1].top - height > target.top + target.height
        ) {
          const leavingAt = target.top + target.height * 0.88;
          const outgoing = await sample(leavingAt);
          await expectAtPageAnchor(outgoing, index);
          expect(outgoing.opacity).toBeGreaterThan(0);
          expect(outgoing.opacity).toBeLessThan(1);
          // A nearly transparent model must not keep intercepting page gestures.
          const fadeDistance = Math.min(height * 0.12, target.height * 0.45);
          const fading = await sample(
            target.top + target.height - fadeDistance * 0.1,
          );
          expect(fading.opacity).toBeLessThan(0.05);
          await expect(page.locator(".model-interaction")).toHaveAttribute(
            "inert",
            "",
          );
          await expect(page.locator(".model-interaction")).toHaveAttribute(
            "tabindex",
            "-1",
          );
          const hidden = await sample(target.top + target.height + 1);
          expect(hidden.opacity).toBeLessThan(0.01);
          const reverse = await sample(leavingAt);
          expectSamePageFrame(reverse, outgoing);
          expect(reverse.opacity).toBeCloseTo(outgoing.opacity, 2);
        }
        const arrived = await sample(route[index + 1].top - height * 0.75);
        await expectAtPageAnchor(arrived, index + 1);
        expect(arrived.opacity).toBe(1);
        const reverse = await sample(target.top - height * 0.04);
        expectSamePageFrame(reverse, reading);
        expect(reverse.opacity).toBe(1);
        expect(reverse.turn).toBeLessThan(arrived.turn);
      }
      // The closing sculpture must finish its entrance before the page runs out.
      const bottom = await sample(
        await page.evaluate(
          () => document.documentElement.scrollHeight - innerHeight,
        ),
      );
      await expectAtPageAnchor(bottom, route.length - 1);
      expect(bottom.opacity).toBe(1);
    }
    await checkAnchors();
    await page.clock.resume();

    expect(errors).toEqual([]);
  });
}

test("landing page: live components, deployment and mobile navigation", async ({
  page,
}) => {
  test.setTimeout(90_000);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("SELECT");
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-scene",
    "ready",
    { timeout: 30_000 },
  );
  await expect(page.locator("canvas")).toHaveCount(1);

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

  await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.getByRole("button", { name: "Copy SQL", exact: true }).click();
  await expect(page.locator(".demo-status")).toHaveText("SQL copied.");
  expect(await page.evaluate(() => navigator.clipboard.readText())).toContain(
    "from customers",
  );

  await page
    .getByRole("tab", { name: "chart A timeline from 3 rows", exact: true })
    .click();
  await expect(demo.locator(".apexcharts-canvas")).toBeVisible();
  await expect(page.locator("#demo-source")).toContainText("from tickets");
  // Arrow navigation selects the next component and keeps the active tab focusable.
  await page
    .getByRole("tab", { name: "chart A timeline from 3 rows", exact: true })
    .press("ArrowDown");
  await page.getByRole("tab", { name: "form Fields from rows" }).click();
  await demo.getByLabel("Your name").fill("Ada");
  await demo.getByLabel("Your team").fill("Query crew");
  await demo.getByRole("button", { name: "Save profile" }).click();
  await expect(demo.getByRole("alert")).toContainText("Hello, Ada!");
  await expect(demo.getByLabel("Your team")).toHaveValue("Query crew");
  await page
    .locator("iframe")
    .evaluate((frame) =>
      (frame as HTMLIFrameElement).contentWindow!.location.reload(),
    );
  await expect(demo.getByLabel("Your team")).toHaveValue("Query crew");
  await page.getByRole("tab", { name: "save.sql", exact: true }).click();
  await expect(page.locator("#demo-source")).toContainText("update profiles");
  await expect(page.locator("#demo-source")).toContainText(
    "sqlpage.request_method()",
  );
  await page
    .getByRole("tab", { name: "save.sql", exact: true })
    .press("ArrowLeft");
  await expect(
    page.getByRole("tab", { name: "form.sql", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  await expect(page.locator("#demo-source")).toContainText(
    "/landing-demos/save.sql",
  );
  await page
    .getByRole("tab", { name: "big_number KPIs at a glance", exact: true })
    .click();
  await expect(demo.getByText("98%", { exact: true })).toBeVisible();
  await expect(demo.getByRole("progressbar").last()).toHaveAttribute(
    "aria-valuenow",
    /^98(?:\.0)?$/,
  );
  await expect
    .poll(async () => (await page.locator("iframe").boundingBox())!.height)
    .toBeLessThan(280);

  const catalog = page.getByRole("tab", { name: /^\d+ components/ });
  const count = Number((await catalog.innerText()).match(/\d+/)![0]);
  await catalog.click();
  await expect(demo.locator(".component-catalog a")).toHaveCount(count);
  await expect(
    demo.locator(".component-catalog a").first().locator("svg"),
  ).toHaveCount(2);
  await expect(
    demo.getByRole("link", { name: "form", exact: true }),
  ).toHaveAttribute("href", "/component.sql?component=form");
  await expect(page.locator(".sql-editor")).toBeHidden();
  await page
    .getByRole("tab", { name: "big_number KPIs at a glance", exact: true })
    .click();

  await page
    .getByRole("button", { name: "A SQLPage app", exact: true })
    .click();
  await expect(page.locator(".stack-comparison")).toHaveAttribute(
    "data-stack",
    "sqlpage",
  );
  const comparison = page.locator(".stack-comparison");
  await expect(
    comparison.locator('[data-stack-copy="sqlpage"]').last(),
  ).toHaveAttribute("aria-hidden", "false");
  await page
    .getByRole("button", { name: "A typical webapp", exact: true })
    .click();
  await expect(comparison).toHaveAttribute("data-stack", "typical");
  await expect(
    comparison.locator('[data-stack-copy="sqlpage"]').last(),
  ).toHaveAttribute("aria-hidden", "true");
  const features = page.getByRole("tablist", { name: "Frontend features" });
  await expect(features.getByRole("tab", { name: "Instant" })).toHaveAttribute(
    "aria-selected",
    "true",
  );
  await expect(page.locator("#instant-panel")).toBeVisible();
  for (const [name, id] of [
    ["Make it yours", "customize"],
    ["Ship it anywhere", "ship"],
    ["Safe by default", "safe"],
  ]) {
    await features.getByRole("tab", { name }).click();
    await expect(page.locator(`#${id}-panel`)).toBeVisible();
    await expect(page.locator("#instant-panel")).toBeHidden();
  }
  await features
    .getByRole("tab", { name: "Safe by default" })
    .press("ArrowLeft");
  await expect(features.getByRole("tab", { name: "Instant" })).toBeFocused();
  await expect(page.locator("#instant-panel")).toBeVisible();

  // Each measured anchor owns the same canvas, including after responsive reflow.
  for (const stop of await page.locator("[data-scene-stop]").all()) {
    await stop.evaluate((element) =>
      window.scrollTo(
        0,
        element.getBoundingClientRect().top + scrollY - innerHeight * 0.25,
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
  await expect
    .poll(async () => {
      const canvas = (await page.locator("canvas").boundingBox())!;
      return canvas.y + canvas.height * 0.224284;
    })
    .toBeLessThan(page.viewportSize()!.height);
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
  // Check both sides of the tablet breakpoint, not just the phone layout.
  for (const width of [320, 390, 768, 827, 1000]) {
    await page.setViewportSize({ width, height: 844 });
    await page.locator(".frontend-tabs").scrollIntoViewIfNeeded();
    const panelHeight = (await page.locator(".frontend-panels").boundingBox())!
      .height;
    for (const name of [
      "Make it yours",
      "Ship it anywhere",
      "Safe by default",
      "Instant",
    ]) {
      await features.getByRole("tab", { name }).click();
      expect(
        await page.locator(".frontend-panels").evaluate((panel) => {
          const bounds = panel.getBoundingClientRect();
          return [
            ...panel.querySelectorAll(
              '[role="tabpanel"]:not([hidden]) .feature-node',
            ),
          ].every((node) => {
            const rect = node.getBoundingClientRect();
            return rect.left >= bounds.left && rect.right <= bounds.right;
          });
        }),
      ).toBe(true);
      if (width === 390) {
        expect(
          (await page.locator(".frontend-panels").boundingBox())!.height,
        ).toBeCloseTo(panelHeight, 0);
      }
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
      ).toBe(true);
    }
  }
  expect(errors).toEqual([]);
});

import { expect, type Locator, test } from "@playwright/test";
import { settleLandingPreview } from "./landing-helpers.ts";

async function gesture(element: Locator) {
  return element.evaluate((element) => {
    const style = getComputedStyle(element);
    const [x, y = "0"] = style.translate.split(" ");
    return {
      opacity: Number(style.opacity),
      x: parseFloat(x) || 0,
      y: parseFloat(y) || 0,
      scale: parseFloat(style.scale) || 1,
      rotate: parseFloat(style.rotate) || 0,
    };
  });
}

async function authoredBounds(element: Locator) {
  return element.evaluate((element: HTMLElement) => {
    let top = 0;
    for (
      let ancestor: HTMLElement | null = element;
      ancestor;
      ancestor = ancestor.offsetParent as HTMLElement | null
    ) {
      top += ancestor.offsetTop;
    }
    return { top, height: element.offsetHeight };
  });
}

// Sample the painted upper surface, excluding the closed path's flat base.
async function surface(wave: Locator) {
  return wave
    .locator("path.wave-front")
    .evaluate((element: SVGPathElement) =>
      [100, 300, 500, 700, 900, 1100].map(
        (length) => element.getPointAtLength(length).y,
      ),
    );
}

function difference(a: number[], b: number[]) {
  return (
    a.reduce((sum, value, index) => sum + Math.abs(value - b[index]), 0) /
    a.length
  );
}

for (const viewport of [
  { width: 1280, height: 720 },
  { width: 390, height: 844 },
  { width: 1641, height: 1600 },
]) {
  test(`landing sections cover, elements respond and waves deform at ${viewport.width}px`, async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.setViewportSize(viewport);
    await page.clock.install();
    await page.goto("/");
    await expect(page.locator(".sqlpage-world")).toHaveAttribute(
      "data-scene",
      "ready",
      { timeout: 30_000 },
    );
    await settleLandingPreview(page);
    // Control idle wave time while letting real scroll events and animation frames run.
    await page.clock.pauseAt(await page.evaluate(() => Date.now() + 60_000));
    await page.clock.runFor(32);
    async function scrollTo(y: number) {
      await page.evaluate(
        (y) =>
          new Promise<void>((resolve) => {
            const done = () => {
              window.removeEventListener("scroll", done);
              resolve();
            };
            window.addEventListener("scroll", done, { once: true });
            const before = scrollY;
            window.scrollTo({ top: y, behavior: "instant" });
            if (scrollY === before) done();
          }),
        y,
      );
      await page.clock.runFor(32);
    }
    const sections = page.locator(".landing-section");
    const positions = await sections.evaluateAll((elements) =>
      elements.map((element) => ({
        top: element.getBoundingClientRect().top + scrollY,
        stickyTop: Math.min(
          0,
          innerHeight - element.getBoundingClientRect().height,
        ),
      })),
    );
    for (let index = 0; index < positions.length; index++)
      await expect(sections.nth(index)).toHaveCSS("position", "sticky");
    async function expectedTop(index: number) {
      return Math.max(
        positions[index].stickyTop,
        positions[index].top - (await page.evaluate(() => scrollY)),
      );
    }
    async function surfaceMotion(wave: Locator) {
      let before = await surface(wave);
      let movement = 0;
      // Sum short steps so two distant phases cannot cancel each other out.
      for (let sample = 0; sample < 4; sample++) {
        await page.clock.runFor(80);
        const after = await surface(wave);
        movement += difference(before, after);
        before = after;
      }
      return movement;
    }
    const control = page.locator(".purpose-section .segmented");
    const card = page.locator(".purpose-section .benefit").first();
    const trailingCard = page.locator(".purpose-section .benefit").last();
    const controlBounds = await authoredBounds(control);
    const cardBounds = await authoredBounds(card);
    const trailingCardBounds = await authoredBounds(trailingCard);
    for (let index = 1; index < positions.length; index++) {
      const previous = sections.nth(index - 1);
      const incoming = sections.nth(index);
      const boundary = positions[index].top;
      await scrollTo(boundary - viewport.height * 0.85);
      expect((await previous.boundingBox())!.y).toBeCloseTo(
        await expectedTop(index - 1),
        0,
      );
      const heading = incoming.locator('[data-section-reveal="heading"]');
      const enteringHeading = await gesture(heading);
      expect(enteringHeading.opacity).toBeLessThan(0.2);
      expect(enteringHeading.y).toBeGreaterThan(50);
      const first = (await incoming.boundingBox())!;
      const wave = incoming.locator(".section-wave");
      await expect(wave).toHaveAttribute("data-wave-visible", "");
      await page.clock.runFor(32);
      const shape = await surface(wave);
      expect(Math.max(...shape) - Math.min(...shape)).toBeGreaterThan(35);
      let calmMotion = 0;
      if (index === 1) {
        await page.clock.fastForward(12_000);
        calmMotion = await surfaceMotion(wave);
        expect(calmMotion).toBeGreaterThan(0.1);
      }
      await scrollTo(boundary - viewport.height * 0.45);
      if (index === 1) {
        // Scrolling supplies wind: the water keeps moving after scrolling stops.
        const drivenMotion = await surfaceMotion(wave);
        expect(drivenMotion).toBeGreaterThan(calmMotion * 2);
        expect(drivenMotion).toBeGreaterThan(3);
        await page.clock.fastForward(12_000);
        const settledMotion = await surfaceMotion(wave);
        expect(settledMotion).toBeGreaterThan(0.1);
        expect(settledMotion).toBeLessThan(drivenMotion * 0.5);
      }
      const second = (await incoming.boundingBox())!;
      const settledHeading = await gesture(heading);
      expect(settledHeading.opacity).toBeGreaterThan(0.98);
      expect(settledHeading.y).toBeLessThan(2);
      expect(Math.abs(first.y - second.y - viewport.height * 0.4)).toBeLessThan(
        2,
      );
      expect((await previous.boundingBox())!.y).toBeCloseTo(
        await expectedTop(index - 1),
        0,
      );
      // The incoming surface really paints over the held tail, not merely below it.
      expect(
        await incoming.evaluate((element) => {
          const bounds = element.getBoundingClientRect();
          const hit = document.elementFromPoint(
            innerWidth / 2,
            bounds.top + 12,
          );
          return !!hit && element.contains(hit);
        }),
      ).toBe(true);
      await scrollTo(boundary - viewport.height * 0.85);
      // Element choreography reverses with scrolling; the water retains its momentum.
      const reversed = await gesture(heading);
      expect(reversed.opacity).toBeCloseTo(enteringHeading.opacity, 3);
      expect(reversed.y).toBeCloseTo(enteringHeading.y, 2);
    }

    // Controls slide into reach; cards also grow and tilt into their resting pose.
    for (const [element, bounds, role] of [
      [control, controlBounds, "control"],
      [card, cardBounds, "card"],
    ] as const) {
      const partialScroll = bounds.top - viewport.height * 0.83;
      await scrollTo(partialScroll);
      const entering = await gesture(element);
      expect(entering.opacity).toBeGreaterThan(0.05);
      expect(entering.opacity).toBeLessThan(0.95);
      expect(entering.y).toBeGreaterThan(5);
      if (viewport.width <= 600) {
        expect(entering.x).toBe(0);
        expect(entering.scale).toBe(1);
        expect(entering.rotate).toBe(0);
      } else {
        expect(Math.abs(entering.x)).toBeGreaterThan(5);
        if (role === "card") {
          expect(entering.scale).toBeLessThan(0.98);
          expect(Math.abs(entering.rotate)).toBeGreaterThan(0.5);
        }
      }
      await scrollTo(bounds.top - viewport.height * 0.55);
      await expect(element).toHaveCSS("opacity", "1");
      expect(await gesture(element)).toEqual({
        opacity: 1,
        x: 0,
        y: 0,
        scale: 1,
        rotate: 0,
      });
      await scrollTo(partialScroll);
      const reversed = await gesture(element);
      expect(reversed.opacity).toBeCloseTo(entering.opacity, 3);
      expect(reversed.x).toBeCloseTo(entering.x, 2);
      expect(reversed.y).toBeCloseTo(entering.y, 2);
      expect(reversed.scale).toBeCloseTo(entering.scale, 3);
      expect(reversed.rotate).toBeCloseTo(entering.rotate, 2);
    }

    // Even the last card gets a fully readable interval before the next wave arrives.
    const readingScroll = positions[1].top - positions[1].stickyTop;
    await scrollTo(Math.ceil(readingScroll));
    await expect(trailingCard).toHaveCSS("opacity", "1");
    const readableCard = (await trailingCard.boundingBox())!;
    expect(readableCard.y).toBeGreaterThanOrEqual(0);
    expect(readableCard.y + readableCard.height).toBeLessThanOrEqual(
      viewport.height + 1,
    );

    // Once the fully read tail holds, the next wave gives it an outro.
    const tailBottom =
      positions[1].stickyTop +
      trailingCardBounds.top -
      positions[1].top +
      trailingCardBounds.height;
    const waveHeight = await sections
      .nth(2)
      .evaluate((element) =>
        parseFloat(getComputedStyle(element).getPropertyValue("--wave-height")),
      );
    await scrollTo(
      positions[2].top -
        (tailBottom + waveHeight * 0.75 - 20 - trailingCardBounds.height * 0.2),
    );
    const leaving = await gesture(trailingCard);
    expect(leaving.opacity).toBeGreaterThan(0.01);
    expect(leaving.opacity).toBeLessThan(0.95);
    expect(leaving.y).toBeLessThan(-5);
    if (viewport.width <= 600) expect(leaving.scale).toBe(1);
    else expect(leaving.scale).toBeLessThan(0.98);
    await scrollTo(Math.ceil(readingScroll));
    await expect(trailingCard).toHaveCSS("opacity", "1");

    // Keyboard focus must uncover a control on an earlier, covered section.
    await scrollTo(positions[4].top);
    const stackToggle = page.getByRole("button", {
      name: "A SQLPage app",
      exact: true,
    });
    await stackToggle.focus();
    await page.clock.runFor(32);
    await expect(stackToggle).toBeInViewport();
    await expect(control).toHaveCSS("opacity", "1");
    await page.clock.resume();
    await stackToggle.click();
    await expect(page.locator(".stack-comparison")).toHaveAttribute(
      "data-stack",
      "sqlpage",
    );
    await page.emulateMedia({ reducedMotion: "reduce" });
    await expect(page.locator(".sqlpage-world")).not.toHaveAttribute(
      "data-section-motion",
      "",
    );
    await expect(sections.first()).toHaveCSS("position", "relative");
    const reducedWave = sections.nth(1).locator("path.wave-front");
    const reducedPath = await reducedWave.getAttribute("d");
    await page.evaluate(() => window.scrollTo(0, 0));
    await expect(control).toHaveCSS("opacity", "1");
    await expect(control).toHaveCSS("translate", "none");
    await expect(card).toHaveCSS("scale", "none");
    await expect(reducedWave).toHaveAttribute("d", reducedPath!);
    expect(
      await sections.evaluateAll((elements) =>
        elements.every((element, index) => {
          if (!index) return true;
          return (
            element.getBoundingClientRect().top >=
            elements[index - 1].getBoundingClientRect().bottom - 2
          );
        }),
      ),
    ).toBe(true);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
  });
}

test("every section can be read before its tail is covered at common form factors", async ({
  page,
}) => {
  test.setTimeout(180_000);
  await page.goto("/");
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-scene",
    "ready",
    { timeout: 30_000 },
  );
  await settleLandingPreview(page);
  for (const viewport of [
    { width: 1869, height: 1039 },
    { width: 1280, height: 720 },
    { width: 1024, height: 768 },
    { width: 768, height: 1024 },
    { width: 390, height: 844 },
    { width: 844, height: 390 },
    { width: 1641, height: 1600 },
  ]) {
    await page.setViewportSize(viewport);
    await page.evaluate(() => window.scrollTo(0, 0));
    const sections = page.locator(".landing-section");
    // Pin the top only when the whole section fits; otherwise scroll through first.
    await expect
      .poll(() =>
        sections.evaluateAll((elements) =>
          elements.every(
            (element) =>
              Math.abs(
                Number.parseFloat(getComputedStyle(element).top) -
                  Math.min(
                    0,
                    innerHeight - element.getBoundingClientRect().height,
                  ),
              ) < 1,
          ),
        ),
      )
      .toBe(true);
    const route = await sections.evaluateAll((elements) =>
      elements.map((section) => {
        const rect = section.getBoundingClientRect();
        const reveals = [
          ...section.querySelectorAll<HTMLElement>("[data-section-reveal]"),
        ];
        // Read authored positions without changing DOM or bypassing page initialization.
        const selected = reveals
          .map((element, index) => ({ element, index }))
          .filter(
            ({ element, index }) =>
              element.tagName === "H2" ||
              index === reveals.length - 1 ||
              element.matches(".database-grid a"),
          );
        return {
          top: rect.top + scrollY,
          hold: rect.top + scrollY - Math.min(0, innerHeight - rect.height),
          items: selected.map(({ element, index }) => {
            let top = 0;
            for (
              let parent: HTMLElement | null = element;
              parent;
              parent = parent.offsetParent as HTMLElement | null
            )
              top += parent.offsetTop;
            return { index, top, height: element.offsetHeight };
          }),
        };
      }),
    );
    for (let index = 0; index < route.length; index++) {
      const section = sections.nth(index);
      const { top, hold, items } = route[index];
      if (hold > top + 2) {
        const step = Math.min(100, (hold - top) / 2);
        await page.evaluate((y) => window.scrollTo(0, y), top + step);
        await expect
          .poll(async () => {
            const actualScroll = await page.evaluate(() => scrollY);
            return Math.abs(
              (await section.boundingBox())!.y - (top - actualScroll),
            );
          })
          .toBeLessThan(1);
      }
      for (const item of items) {
        const element = section
          .locator("[data-section-reveal]")
          .nth(item.index);
        const target =
          item.top - Math.max(32, (viewport.height - item.height) / 2);
        await page.evaluate(
          (y) => window.scrollTo({ top: y, behavior: "instant" }),
          Math.min(target, hold),
        );
        await expect(element).toHaveCSS("opacity", "1");
        if (item.height <= viewport.height - 64) {
          // IntersectionObserver rounds rotated card bounds by a fraction of a pixel.
          await expect(element).toBeInViewport({ ratio: 0.9999 });
          expect(
            await element.evaluate((element) => {
              const bounds = element.getBoundingClientRect();
              const hit = document.elementFromPoint(
                bounds.x + bounds.width / 2,
                bounds.y + bounds.height / 2,
              );
              return !!hit && element.contains(hit);
            }),
          ).toBe(true);
        }
      }
    }
    await page.evaluate(() =>
      window.scrollTo(0, document.documentElement.scrollHeight),
    );
    const footer = page.locator(".landing-footer");
    await expect(footer).toBeInViewport({ ratio: 0.9999 });
    await expect
      .poll(() =>
        footer.evaluate(
          (element) =>
            innerHeight -
            element.getBoundingClientRect().bottom -
            Number.parseFloat(
              getComputedStyle(element.closest(".commit-section")!)
                .paddingBottom,
            ),
        ),
      )
      .toBeCloseTo(0, 0);
  }
});

test("landing content remains readable without JavaScript", async ({
  browser,
}) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  await page.goto(test.info().project.use.baseURL!);
  await expect(page.locator(".landing-section").first()).toHaveCSS(
    "position",
    "relative",
  );
  await expect(
    page.getByRole("heading", { name: "Small team. Serious tools;" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "COMMIT; start with a query" }),
  ).toBeVisible();
  await context.close();
});

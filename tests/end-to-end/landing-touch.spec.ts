import { expect, type Locator, test } from "@playwright/test";

async function visibleBounds(sculpture: Locator) {
  const rect = await sculpture.boundingBox();
  if (!rect)
    throw new Error("The sculpture must be visible to receive a gesture");
  return rect;
}

test.describe("landing sculpture touch gestures", () => {
  test.use({
    viewport: { width: 390, height: 844 },
    isMobile: true,
    hasTouch: true,
    reducedMotion: "reduce",
  });

  test("vertical swipes scroll, sideways swipes rotate, and cancellation recovers", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto("/");
    await expect(page.locator(".sqlpage-world")).toHaveAttribute(
      "data-scene",
      "ready",
      { timeout: 30_000 },
    );
    const sculpture = page.getByRole("application", {
      name: "Interactive 3D database sculpture",
    });
    await expect(sculpture).toHaveAttribute("tabindex", "0");
    await expect(sculpture).toHaveCSS("touch-action", "pan-y pinch-zoom");
    await expect(sculpture).toHaveAccessibleDescription(/swipe up or down/);
    await sculpture.evaluate((element) => {
      element.addEventListener("pointercancel", () => {
        element.setAttribute("data-cancelled", "true");
      });
    });

    // Real browser touch input exercises native scrolling and pointercancel.
    // DOM-dispatched PointerEvents cannot reveal a touch-action regression.
    const session = await page.context().newCDPSession(page);
    async function swipe(dx: number, dy: number) {
      const rect = await visibleBounds(sculpture);
      const x = rect.x + rect.width * 0.5;
      const y = Math.min(rect.y + rect.height * 0.65, 700);
      await session.send("Input.dispatchTouchEvent", {
        type: "touchStart",
        touchPoints: [{ x, y }],
      });
      for (let step = 1; step <= 12; step++) {
        await session.send("Input.dispatchTouchEvent", {
          type: "touchMove",
          touchPoints: [{ x: x + (dx * step) / 12, y: y + (dy * step) / 12 }],
        });
        await page.waitForTimeout(16);
      }
      await session.send("Input.dispatchTouchEvent", {
        type: "touchEnd",
        touchPoints: [],
      });
    }

    const canvas = page.locator("canvas");
    const initial = await canvas.screenshot();
    await swipe(95, 0);
    expect(await page.evaluate(() => scrollY)).toBe(0);
    expect((await canvas.screenshot()).equals(initial)).toBe(false);

    await swipe(30, -220);
    await expect.poll(() => page.evaluate(() => scrollY)).toBeGreaterThan(100);
    await expect(sculpture).toHaveAttribute("data-cancelled", "true");

    // A cancelled scroll must release the trackball and allow the next drag.
    await page.evaluate(() => window.scrollTo(0, 0));
    await expect(sculpture).toHaveAttribute("tabindex", "0");
    await expect
      .poll(async () => (await visibleBounds(sculpture)).y)
      .toBeGreaterThan(90);
    const beforeRecovery = await canvas.screenshot();
    await swipe(-95, 0);
    expect(await page.evaluate(() => scrollY)).toBe(0);
    expect((await canvas.screenshot()).equals(beforeRecovery)).toBe(false);

    // Native pinch zoom remains available over the sculpture.
    const rect = await visibleBounds(sculpture);
    const x = rect.x + rect.width * 0.5;
    const y = rect.y + rect.height * 0.5;
    await session.send("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [
        { x: x - 20, y, id: 1 },
        { x: x + 20, y, id: 2 },
      ],
    });
    for (let step = 1; step <= 12; step++) {
      await session.send("Input.dispatchTouchEvent", {
        type: "touchMove",
        touchPoints: [
          { x: x - 20 - step * 4, y, id: 1 },
          { x: x + 20 + step * 4, y, id: 2 },
        ],
      });
      await page.waitForTimeout(16);
    }
    await session.send("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
    await expect
      .poll(() => page.evaluate(() => visualViewport?.scale ?? 1))
      .toBeGreaterThan(1);
    expect(errors).toEqual([]);
    await session.detach();
  });

  test("the full-screen opening sculpture also allows native scrolling", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await page.goto("/");
    await expect(page.locator(".sqlpage-world")).toHaveAttribute(
      "data-scene",
      "ready",
      { timeout: 30_000 },
    );
    const sculpture = page.getByRole("application", {
      name: "Interactive 3D database sculpture",
    });
    const initialWidth = (await visibleBounds(sculpture)).width;
    await sculpture.evaluate((element) => {
      element.addEventListener("pointercancel", () => {
        element.setAttribute("data-cancelled", "true");
      });
    });
    await page.evaluate(() => {
      const hero = document.querySelector<HTMLElement>(".experience");
      const viewport = hero?.querySelector<HTMLElement>(".viewport");
      if (!hero || !viewport)
        throw new Error("The opening scene must be present");
      window.scrollTo(0, (hero.offsetHeight - viewport.offsetHeight) * 0.76);
    });
    await expect(sculpture).toHaveAttribute("tabindex", "0");
    await expect
      .poll(async () => (await visibleBounds(sculpture)).width)
      .toBeGreaterThan(initialWidth * 1.2);
    const rect = await visibleBounds(sculpture);
    const x = rect.x + rect.width / 2;
    const y = Math.min(rect.y + rect.height / 2, 650);
    const before = await page.evaluate(() => scrollY);
    const session = await page.context().newCDPSession(page);
    await session.send("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [{ x, y }],
    });
    for (let step = 1; step <= 12; step++) {
      await session.send("Input.dispatchTouchEvent", {
        type: "touchMove",
        touchPoints: [{ x, y: y - step * 18 }],
      });
      await page.waitForTimeout(16);
    }
    await session.send("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
    await expect
      .poll(() => page.evaluate(() => scrollY))
      .toBeGreaterThan(before + 100);
    await expect(sculpture).toHaveAttribute("data-cancelled", "true");
    await session.detach();
  });
});

test("landing sculpture keeps mouse and keyboard rotation", async ({
  page,
}) => {
  test.setTimeout(90_000);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  const sculpture = page.getByRole("application", {
    name: "Interactive 3D database sculpture",
  });
  await expect(sculpture).toHaveAttribute("tabindex", "0", { timeout: 30_000 });
  const canvas = page.locator("canvas");
  const initial = await canvas.screenshot();
  const rect = await visibleBounds(sculpture);
  await page.mouse.move(rect.x + rect.width / 2, rect.y + rect.height / 2);
  await page.mouse.down();
  await page.mouse.move(
    rect.x + rect.width / 2,
    rect.y + rect.height / 2 + 70,
    {
      steps: 8,
    },
  );
  await page.mouse.up();
  const dragged = await canvas.screenshot();
  expect(dragged.equals(initial)).toBe(false);
  await sculpture.focus();
  await sculpture.press("ArrowRight");
  expect((await canvas.screenshot()).equals(dragged)).toBe(false);
});

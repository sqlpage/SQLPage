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

  // Sample the reading interval too: no parked pose or straight transfer between stops.
  const route = await page
    .locator("[data-scene-stop]")
    .evaluateAll((stops) =>
      stops
        .slice(1, 3)
        .map(
          (stop) =>
            stop.getBoundingClientRect().top + scrollY - innerHeight * 0.25,
        ),
    );
  const poses: { x: number; y: number; turn: number }[] = [];
  let previousTurn = Number(
    await page.locator("canvas").getAttribute("data-scroll-turn"),
  );
  for (const progress of [0, 0.03, 0.06, 0.5, 0.85, 1]) {
    const scroll = Math.round(route[0] + (route[1] - route[0]) * progress);
    await page.evaluate((y) => window.scrollTo(0, y), scroll);
    await expect
      .poll(async () =>
        Number(await page.locator("canvas").getAttribute("data-scroll-turn")),
      )
      .toBeGreaterThan(previousTurn + 0.01);
    poses.push(
      await page.locator("canvas").evaluate((canvas) => {
        const rect = canvas.getBoundingClientRect();
        return {
          x: rect.x + rect.width * 0.488205,
          y: rect.y + rect.height / 2,
          turn: Number(canvas.dataset.scrollTurn),
        };
      }),
    );
    previousTurn = poses.at(-1)!.turn;
  }
  expect(poses[1].turn).toBeGreaterThan(poses[0].turn);
  expect(poses[2].turn).toBeGreaterThan(poses[1].turn);
  // Reading follows ordinary document scrolling; spin is independent of docking.
  const scrollStep = (route[1] - route[0]) * 0.03;
  expect(poses[2].y - poses[1].y).toBeLessThan(-scrollStep * 0.6);
  const restingSpin = (poses[2].turn - poses[1].turn) / 0.03;
  const travelingSpin = (poses[4].turn - poses[3].turn) / 0.35;
  expect(restingSpin / travelingSpin).toBeCloseTo(1, 1);
  expect(Math.abs(poses[1].x - poses[0].x)).toBeLessThan(
    page.viewportSize()!.width * 0.04,
  );
  // The broad passage remains visible and inside the page throughout the transfer.
  for (const pose of poses) {
    expect(pose.x).toBeGreaterThan(0);
    expect(pose.x).toBeLessThan(page.viewportSize()!.width);
  }
  for (const pose of poses) {
    expect(pose.y).toBeGreaterThan(0);
    expect(pose.y).toBeLessThan(page.viewportSize()!.height);
  }

  // Reading poses stay clear of real component controls.
  const readingStops = await page.locator("[data-scene-stop]").all();
  for (let index = 0; index < readingStops.length - 1; index++) {
    const previous = await page
      .locator("canvas")
      .getAttribute("data-scroll-turn");
    await readingStops[index].evaluate((stop) => {
      const section = stop.closest("section")!;
      const next = section.nextElementSibling as HTMLElement;
      window.scrollTo(
        0,
        (section.offsetTop + next.offsetTop) / 2 - innerHeight * 0.12,
      );
    });
    await expect(page.locator("canvas")).not.toHaveAttribute(
      "data-scroll-turn",
      previous!,
    );
    const readingPose = await readingStops[index].evaluate((stop) => {
      const frame = document.querySelector("canvas")!.getBoundingClientRect();
      const body = {
        left: frame.left + frame.width * 0.291366,
        right: frame.left + frame.width * 0.685044,
        top: frame.top + frame.height * 0.224284,
        bottom: frame.top + frame.height * 0.803486,
      };
      const overlap = [
        ...stop
          .closest("section")!
          .querySelectorAll(
            ".demo-layout, .purpose-grid, .deployment-example, .database-grid",
          ),
      ].some((element) => {
        const rect = element.getBoundingClientRect();
        return (
          body.left < rect.right &&
          body.right > rect.left &&
          body.top < rect.bottom &&
          body.bottom > rect.top
        );
      });
      return { overlap, visible: body.bottom > 0 && body.top < innerHeight };
    });
    expect(readingPose.overlap).toBe(false);
    expect(readingPose.visible).toBe(true);
  }

  // Keep the real cinematic check above; stop idle animation while exercising UI
  // so software WebGL on CI does not compete with iframe and input rendering.
  await page.emulateMedia({ reducedMotion: "reduce" });
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-motion",
    "off",
  );

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

import { expect, test } from "@playwright/test";

test("streaming comparison starts early, pauses and restarts with its tab", async ({
  page,
}) => {
  await page.goto("/");
  const diagram = page.locator(".instant-diagram");
  await diagram.scrollIntoViewIfNeeded();
  await expect(diagram).toHaveAttribute("data-stream-motion", "true");
  await expect(
    diagram.getByText("streaming HTML", { exact: true }),
  ).toBeVisible();
  const bounds = await diagram.boundingBox();

  // Seek the real animations to the moment SQLPage has streamed HTML while
  // the illustrative app is still loading JavaScript, before its API/render.
  await diagram.evaluate((element) => {
    for (const animation of element.getAnimations({ subtree: true })) {
      animation.pause();
      animation.currentTime = 1200;
    }
  });
  await expect(diagram.locator(".stream-fill")).toHaveCSS(
    "clip-path",
    "inset(0px 0% 0px 0px)",
  );
  await expect(diagram.locator(".timeline-fill").first()).toHaveCSS(
    "clip-path",
    "inset(0px 0% 0px 0px)",
  );
  await expect(diagram.locator(".timeline-fill").nth(2)).toHaveCSS(
    "clip-path",
    "inset(0px 100% 0px 0px)",
  );
  await expect(diagram.locator(".timeline-fill").last()).toHaveCSS(
    "clip-path",
    "inset(0px 100% 0px 0px)",
  );

  await diagram.evaluate((element) => {
    for (const animation of element.getAnimations({ subtree: true }))
      animation.currentTime = 5600;
  });
  await expect(diagram.locator(".timeline-fill").last()).toHaveCSS(
    "clip-path",
    "inset(0px 0% 0px 0px)",
  );
  expect(await diagram.boundingBox()).toEqual(bounds);

  await diagram
    .getByRole("button", { name: "Pause streaming comparison" })
    .click();
  expect(
    await diagram.evaluate((element) =>
      element
        .getAnimations({ subtree: true })
        .every((animation) => animation.playState === "paused"),
    ),
  ).toBe(true);
  await diagram
    .getByRole("button", { name: "Play streaming comparison" })
    .click();
  expect(
    await diagram.evaluate((element) =>
      element
        .getAnimations({ subtree: true })
        .every((animation) => animation.playState === "running"),
    ),
  ).toBe(true);

  await page.getByRole("tab", { name: "Make it yours" }).click();
  await expect(diagram).toBeHidden();
  await expect(diagram).not.toHaveAttribute("data-stream-motion");
  await page.getByRole("tab", { name: "Instant" }).click();
  await expect(diagram).toHaveAttribute("data-stream-motion", "true");

  await page.emulateMedia({ reducedMotion: "reduce" });
  await expect(diagram).not.toHaveAttribute("data-stream-motion");
  await expect(diagram.locator(".timeline-toggle")).toBeHidden();
  await expect(diagram.locator(".stream-fill")).toHaveCSS("clip-path", "none");
  await expect(
    diagram.getByText("streaming HTML", { exact: true }),
  ).toBeVisible();
});

test("streaming label and timelines fit narrow screens without shifting", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  const diagram = page.locator(".instant-diagram");
  for (const width of [320, 390, 768, 1280]) {
    await page.setViewportSize({ width, height: 844 });
    await diagram.scrollIntoViewIfNeeded();
    await expect(
      diagram.getByText("streaming HTML", { exact: true }),
    ).toBeVisible();
    expect(
      await diagram.evaluate((element) => {
        const bar = element.querySelector(".stream-timeline")!;
        const label = bar.querySelector("span")!.getBoundingClientRect();
        const bounds = bar.getBoundingClientRect();
        return (
          label.left >= bounds.left &&
          label.right <= bounds.right &&
          element.scrollWidth <= element.clientWidth &&
          document.documentElement.scrollWidth <= innerWidth
        );
      }),
    ).toBe(true);
  }
});

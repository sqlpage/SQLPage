import { expect, test } from "../../fixture.ts";

for (const theme of ["light", "dark"]) {
  test(`colored values match their cards in the ${theme} theme`, async ({
    page,
  }) => {
    await page.goto(`/big-number/?theme=${theme}`);
    await expect(page.locator("body")).toHaveAttribute("data-bs-theme", theme);
    await expect(page.locator("body")).toHaveCSS("color-scheme", theme);

    const defaultColor = await page
      .locator("#default-color .h1")
      .evaluate((value) => getComputedStyle(value).color);
    const rem = await page
      .locator("html")
      .evaluate((html) => Number.parseFloat(getComputedStyle(html).fontSize));

    for (const { id, linked } of [
      { id: "plain-color", linked: false },
      { id: "linked-color", linked: true },
    ]) {
      const item = page.locator(`#${id}`);
      const cardColor = await item
        .locator(".card")
        .evaluate((card) => getComputedStyle(card).color);

      // Matching two default-colored elements must not count as a color fix.
      expect(cardColor).not.toBe(defaultColor);
      await expect(item.locator(".h1")).toHaveCSS("color", cardColor);
      await expect(item.locator(".h1 a")).toHaveCount(linked ? 1 : 0);
      if (linked) {
        await expect(item.locator(".h1 a")).toHaveCSS("color", cardColor);
      }
      await expect(item.locator(".card-body")).toHaveCSS(
        "padding-top",
        `${rem}px`,
      );
      await expect(item.locator(".card-body")).toHaveCSS(
        "padding-bottom",
        `${rem}px`,
      );
    }
  });
}

test("long titles and descriptions wrap only when requested", async ({
  page,
}) => {
  await page.setViewportSize({ width: 640, height: 800 });
  for (const wrap of [false, true]) {
    await page.goto(`/big-number/?wrap=${wrap}`);
    const card = page.locator("#long-text .card");
    const title = card.getByText("Items checked across all locations", {
      exact: false,
    });
    const description = card.getByText(
      "Includes items from the main collection",
      {
        exact: false,
      },
    );
    for (const text of [title, description]) {
      const layout = await text.evaluate((element) => ({
        clipped: element.scrollWidth > element.clientWidth,
        multiline:
          element.clientHeight >
          1.5 * Number.parseFloat(getComputedStyle(element).lineHeight),
      }));
      expect(layout).toEqual({ clipped: !wrap, multiline: wrap });
    }
    if (wrap) {
      const neighbor = page.locator("#short-text .card");
      expect((await card.boundingBox())?.height).toBe(
        (await neighbor.boundingBox())?.height,
      );
    }
  }
});

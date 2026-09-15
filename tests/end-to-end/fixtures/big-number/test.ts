import { expect, test } from "../../fixture";

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

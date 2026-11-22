import { expect, test } from "../../fixture.ts";

for (const key of ["Enter", "Space"]) {
  test(`a dismissible alert closes with ${key}`, async ({ page }) => {
    const alert = page.getByRole("alert");
    const close = alert.locator('button[type="button"]');

    await page.keyboard.press("Tab");
    await expect(close).toBeFocused();
    await expect(close).toHaveAccessibleName("Close");
    await page.keyboard.press(key);
    await expect(alert).toHaveCount(0);

    await page.keyboard.press("Tab");
    await expect(
      page.getByRole("button", { name: "Next action" }),
    ).toBeFocused();
  });
}

import { expect, test } from "../../fixture";

test("stops completed loaders and shows only the latest progress", async ({
  page,
}) => {
  await expect(page.locator("#completed-loader")).toBeHidden();
  await expect(page.locator("#active-loader .spinner-border")).toBeVisible();
  await expect(
    page.locator(".sqlpage-progress-container").first(),
  ).toBeHidden();
  const progress = page.getByRole("progressbar");
  await expect(progress).toHaveCount(1);
  await expect(progress).toHaveAttribute("aria-valuenow", "100");
  await expect(progress).toHaveClass(/bg-blue/);
});

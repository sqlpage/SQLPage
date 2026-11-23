import { expect, test } from "../../fixture.ts";

test("slide indicators have distinct accessible names", async ({ page }) => {
  await expect(page.getByRole("button", { name: "Slide 1" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Slide 2" })).toBeVisible();
});

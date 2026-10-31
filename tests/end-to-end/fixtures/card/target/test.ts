import { expect, test } from "../../../fixture.ts";

test("a card's link opens where its target says", async ({ page }) => {
  await expect(
    page.getByRole("link", { name: "In a new tab" }),
  ).toHaveAttribute("target", "_blank");
  await expect(
    page.getByRole("link", { name: "In the same tab" }),
  ).not.toHaveAttribute("target");
});

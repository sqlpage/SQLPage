import { expect, test } from "../../fixture.ts";

test("the search field is announced with its placeholder", async ({ page }) => {
  await expect(
    page.getByRole("searchbox", { name: "Suchen", exact: true }),
  ).toHaveAttribute("placeholder", "Suchen");
  await expect(
    page.getByRole("searchbox", { name: "Search", exact: true }),
  ).toHaveCount(0);
});

test("the search button keeps its own label", async ({ page }) => {
  await expect(page.getByRole("button", { name: "Los", exact: true })).toHaveCount(1);
});

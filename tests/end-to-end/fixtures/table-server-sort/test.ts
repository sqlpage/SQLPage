import { expect, test } from "../../fixture";

test("server-sort columns reload the page in ascending and descending order", async ({
  page,
}) => {
  const sortButton = page.getByRole("button", { name: "name" });

  await sortButton.click();
  await expect(page).toHaveURL(/sort_name=ASCENDING/);
  await expect(page.locator("tbody tr").first()).toHaveText("Alpha");
  await expect(sortButton).toHaveClass(/\basc\b/);

  await sortButton.click();
  await expect(page).toHaveURL(/sort_name=DESCENDING/);
  await expect(page.locator("tbody tr").first()).toHaveText("Zulu");
  await expect(sortButton).toHaveClass(/\bdesc\b/);
});

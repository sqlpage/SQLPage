import { expect, test } from "../../fixture";

test("uses the documented id and switches expanded sections", async ({
  page,
}) => {
  const accordion = page.locator("#example-accordion");
  await expect(accordion).toBeVisible();
  await expect(accordion.getByText("First content")).toBeVisible();
  await expect(accordion.getByText("Second content")).toBeHidden();
  await accordion.getByRole("button", { name: "Second section" }).click();
  await expect(accordion.getByText("Second content")).toBeVisible();
  await expect(accordion.getByText("First content")).toBeHidden();
});

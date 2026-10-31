import { expect, test } from "../../../fixture.ts";

test("cancelling a confirmation stays on the page", async ({ page }) => {
  await page.getByRole("button", { name: "Delete" }).click();
  const dialog = page.getByRole("dialog", { name: "Delete this movie?" });
  await dialog.getByRole("button", { name: "Cancel" }).click();
  await expect(dialog).toBeHidden();
  await expect(page).toHaveURL(/\/button\/confirm\/$/);
});

test("confirming follows the button's link", async ({ page }) => {
  await page.getByRole("button", { name: "Delete" }).click();
  await page
    .getByRole("dialog", { name: "Delete this movie?" })
    .getByRole("button", { name: "Delete" })
    .click();
  await expect(page.getByText("Deleted")).toBeVisible();
});

test("confirming submits the button's form", async ({ page }) => {
  await page.getByRole("button", { name: "Archive" }).click();
  await page
    .getByRole("dialog", { name: "Archive this movie?" })
    .getByRole("button", { name: "Archive" })
    .click();
  await expect(page.getByText("Archived Alien")).toBeVisible();
});

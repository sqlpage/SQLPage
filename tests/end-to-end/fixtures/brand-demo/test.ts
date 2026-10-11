import { expect, test } from "../../fixture.js";

test("the recorded customer app searches, sorts, and persists real records", async ({
  page,
}) => {
  await page.goto("/brand-demo/?reset=1");
  await expect(page.getByRole("cell", { name: "Acme Corp" })).toBeVisible();
  await page.getByRole("searchbox").fill("Globex");
  await expect(page.getByRole("cell", { name: "Globex" })).toBeVisible();
  await expect(page.getByRole("cell", { name: "Acme Corp" })).toBeHidden();
  await page.getByRole("searchbox").fill("");
  await page.getByRole("button", { name: "seats", exact: true }).click();
  await expect(page.locator("tbody tr").first()).toContainText("Initech");
  const defaultIcon = await page
    .locator('link[rel="icon"]')
    .getAttribute("href");
  expect(defaultIcon).toBeTruthy();
  const iconResponse = await page.request.get(defaultIcon!);
  expect(await iconResponse.text()).toContain("#58cce0");
  await page.getByRole("button", { name: "Add a customer" }).click();
  await expect(page.locator('link[rel="icon"]')).toHaveAttribute(
    "href",
    "/brand-demo/symbol.svg",
  );
  await page.getByLabel("Customer name").fill("Northstar Studio");
  await page.getByRole("button", { name: "Save customer" }).click();
  await expect(
    page.getByRole("cell", { name: "Northstar Studio" }),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole("cell", { name: "Northstar Studio" }),
  ).toBeVisible();
});

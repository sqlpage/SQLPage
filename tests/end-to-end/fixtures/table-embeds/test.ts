import { expect, test } from "../../fixture.ts";

test("embeds fragments in multiple columns and skips empty URLs", async ({
  page,
}) => {
  const table = page.locator("#table-embeds table").first();
  await expect(
    table.getByRole("cell", { name: "Inner-only", exact: true }),
  ).toBeVisible();
  await expect(
    table.getByRole("cell", { name: "Second", exact: true }),
  ).toBeVisible();
  await expect(table.locator(":scope > thead")).toHaveCSS("position", "sticky");
  await expect(table.locator("td._col_Preview thead").first()).toHaveCSS(
    "position",
    "static",
  );
  const form = table.locator("#cell-form");
  await form.locator(".ts-control").click();
  await expect(
    form.locator(".ts-dropdown").getByRole("option", { name: "Option" }),
  ).toBeVisible();
  await expect(table.locator("td._col_Details").first()).toBeEmpty();
  await expect(table.locator("td._col_Preview").last()).toBeEmpty();
  await expect(table.locator("td._col_Details").last()).toBeEmpty();

  const requests = await page.evaluate(() =>
    performance
      .getEntriesByType("resource")
      .filter(
        (entry) =>
          entry instanceof PerformanceResourceTiming &&
          entry.initiatorType === "fetch",
      )
      .map((entry) => {
        const url = new URL(entry.name);
        return url.pathname + url.search;
      })
      .sort(),
  );
  expect(requests).toEqual([
    "/form/?fragment=1&id=cell-form&_sqlpage_embed=1",
    "/table-embeds/sample.sql?count=1&label=Second&_sqlpage_embed=1",
    "/table-embeds/sample.sql?count=3&label=Inner-only&_sqlpage_embed=1",
  ]);
});

test("sorting uses embed URLs and search skips embedded cells", async ({
  page,
}) => {
  const wrapper = page.locator("#table-embeds");
  const table = wrapper.locator("table").first();
  await expect(
    table.getByRole("cell", { name: "Inner-only", exact: true }),
  ).toBeVisible();
  const names = table.locator("td._col_Name");
  await table.getByRole("button", { name: "Preview", exact: true }).click();
  await expect(names).toHaveText(["Oak", "Pine", "Moss"]);
  await table.getByRole("button", { name: "Rank", exact: true }).click();
  await expect(names).toHaveText(["Pine", "Moss", "Oak"]);

  const search = wrapper.getByRole("searchbox");
  const visibleNames = names.filter({ visible: true });
  await search.fill("inner-only");
  await expect(visibleNames).toHaveCount(0);
  await search.fill("pine");
  await expect(visibleNames).toHaveText(["Pine"]);
});

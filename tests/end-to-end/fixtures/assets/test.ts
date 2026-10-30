import { expect, test } from "../../fixture.ts";

test("assets do not name source maps that SQLPage does not serve", async ({
  page,
}) => {
  const assets = await page
    .locator("script[src], link[rel=stylesheet]")
    .evaluateAll((elements) =>
      elements
        .map(
          (element) =>
            element.getAttribute("src") ?? element.getAttribute("href"),
        )
        .filter((url): url is string => url?.startsWith("/") ?? false),
    );
  expect(assets).not.toEqual([]);
  for (const asset of assets) {
    const body = await (await page.request.get(asset)).text();
    expect(body, asset).not.toContain("sourceMappingURL=");
  }
});

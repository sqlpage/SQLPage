import path from "node:path";
import { test as base, expect } from "@playwright/test";

const fixturesDirectory = path.resolve(__dirname, "fixtures");

export const test = base.extend({
  page: async ({ page }, use, testInfo) => {
    const fixture = path
      .relative(fixturesDirectory, path.dirname(testInfo.file))
      .split(path.sep)
      .join("/");
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));

    const response = await page.goto(`/${fixture}/`);
    expect(response).not.toBeNull();
    expect(response?.ok(), `loading ${response?.url()}`).toBe(true);
    await expect(
      page.getByRole("heading", { name: "An error occurred" }),
      "SQL fixture rendered without errors",
    ).toHaveCount(0);
    await expect(
      page.locator("[data-pre-init]"),
      "component initialization",
    ).toHaveCount(0);

    await use(page);

    expect(errors, "uncaught browser errors").toEqual([]);
  },
});

export type { Page } from "@playwright/test";
export { expect };

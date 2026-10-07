import path from "node:path";
import { test as base, expect, type Page } from "@playwright/test";

const fixturesDirectory = path.resolve(import.meta.dirname, "fixtures");

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

/** Insert server-rendered SQL output, then announce it like a custom fragment consumer. */
export async function loadFragment(
  page: Page,
  url: string,
  root: string | null = "main",
  parent = "main",
) {
  await page.evaluate(
    async ({ url, root, parent }) => {
      const request = new URL(url, location.href);
      request.searchParams.set("_sqlpage_embed", "1");
      const response = await fetch(request);
      if (!response.ok)
        throw new Error(`Fragment request failed: ${response.status}`);
      const fragment = document.createElement("template");
      fragment.innerHTML = await response.text();
      const destination = document.querySelector(parent);
      if (!destination)
        throw new Error(`Missing fragment destination: ${parent}`);
      destination.append(fragment.content);
      if (root) {
        const target =
          root === "document" ? document : document.querySelector(root);
        if (!target) throw new Error(`Missing fragment root: ${root}`);
        target.dispatchEvent(
          new CustomEvent("fragment-loaded", { bubbles: true }),
        );
      }
    },
    { url, root, parent },
  );
}

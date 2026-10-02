import { expect, type Page, test } from "../../fixture.ts";

const INJECTED_HINT = "injected hint";

async function addTooltip(page: Page) {
  await page.evaluate((hint) => {
    const span = document.createElement("span");
    span.id = "added";
    span.textContent = "added";
    span.setAttribute("data-bs-toggle", "tooltip");
    span.setAttribute("title", hint);
    document.querySelector("main")?.appendChild(span);
  }, INJECTED_HINT);
}

test("shows a tooltip added before the document announces a fragment", async ({
  page,
}) => {
  await addTooltip(page);
  await page.evaluate(() =>
    document.dispatchEvent(new CustomEvent("fragment-loaded")),
  );
  await page.locator("#added").hover();

  await expect(page.locator(".tooltip")).toHaveText(INJECTED_HINT);
});

test("shows a tooltip added before an element announces a fragment", async ({
  page,
}) => {
  await addTooltip(page);
  await page.evaluate(() =>
    document
      .querySelector("main")
      ?.dispatchEvent(new CustomEvent("fragment-loaded", { bubbles: true })),
  );
  await page.locator("#added").hover();

  await expect(page.locator(".tooltip")).toHaveText(INJECTED_HINT);
});

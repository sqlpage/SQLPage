import { expect, type Locator, type Page } from "@playwright/test";

// Shared assertions keep documentation page smokes and component fragments aligned.
export async function checkToastNotifications(page: Page) {
  const automatic = page.locator("#toast-auto");
  await expect(automatic).toBeVisible();
  await expect(automatic).toHaveAttribute("data-bs-delay", "5000");
  await expect(automatic).toHaveAttribute("data-bs-autohide", "true");
  await expect(
    automatic.getByRole("button", { name: "Close notification" }),
  ).toBeVisible();
  await expect(page.locator(".toast.show")).toHaveCount(1);
  const automaticHandle = await automatic.elementHandle();
  await automatic.getByRole("button", { name: "Close notification" }).click();
  await expect(automatic).toBeHidden();
  await expect(page.locator("main")).toBeFocused();

  const stackOne = page.locator("#toast-stack-one");
  const stackTwo = page.locator("#toast-stack-two");
  await expect(stackOne).toBeHidden();
  await page.evaluate(() => {
    document.addEventListener("shown.bs.toast", (event) => {
      const toast = event.target;
      if (!(toast instanceof HTMLElement)) throw new Error("Missing toast");
      toast.dataset.shownCount = String(
        Number(toast.dataset.shownCount ?? 0) + 1,
      );
    });
  });
  await page.getByRole("button", { name: "Show queued notifications" }).click();
  await expect(stackOne).toBeVisible();
  await expect(stackTwo).toBeVisible();
  await expect(stackOne).toHaveAttribute("data-shown-count", "1");
  await expect(stackTwo).toHaveAttribute("data-shown-count", "1");
  await expect(page.locator("#toast-short")).toHaveAttribute(
    "data-shown-count",
    "1",
  );
  await page.evaluate(() =>
    document.dispatchEvent(new CustomEvent("fragment-loaded")),
  );
  await page.waitForTimeout(250);
  await expect(page.locator("#toast-short")).toHaveAttribute(
    "data-shown-count",
    "1",
  );
  expect(decodeURIComponent(new URL(page.url()).hash)).toBe(
    "#queued notifications",
  );
  const stackContainer = stackOne.locator("xpath=..");
  await expect(stackContainer).toHaveAttribute(
    "data-sqlpage-toast-position",
    "top-end",
  );
  await expect(stackContainer).toHaveClass(/\bmh-100\b/);
  await expect(stackContainer).toHaveClass(/\boverflow-auto\b/);
  expect(
    await stackContainer.evaluate(
      (container) => container.parentElement === document.body,
    ),
  ).toBe(true);
  expect(
    await stackTwo
      .locator("xpath=..")
      .getAttribute("data-sqlpage-toast-position"),
  ).toBe("top-end");
  const firstBox = await stackOne.boundingBox();
  const secondBox = await stackTwo.boundingBox();
  expect(firstBox).not.toBeNull();
  expect(secondBox).not.toBeNull();
  expect(secondBox?.y).toBeGreaterThanOrEqual(
    (firstBox?.y ?? 0) + (firstBox?.height ?? 0),
  );

  const temporary = page.locator("#toast-short");
  await expect(temporary).toHaveAttribute("data-bs-delay", "2000");
  await expect(temporary).toBeVisible();
  await expect(temporary).toBeHidden({ timeout: 5000 });
  await expect(stackOne).toBeVisible();
  await expect(automatic).toBeHidden({ timeout: 7000 });
  expect(
    await automaticHandle?.evaluate((element) => element.isConnected),
  ).toBe(false);

  const dismissible = page.locator("#toast-dismissible");
  await expect(dismissible).toHaveAttribute("data-bs-delay", "0");
  await expect(dismissible).toHaveAttribute("data-bs-autohide", "false");
  await page.getByRole("button", { name: "Show dismissible error" }).click();
  const closeButton = dismissible.getByRole("button", {
    name: "Close notification",
  });
  await expect(closeButton).toBeVisible();
  const closeStyle = await closeButton.evaluate((button) => {
    const style = getComputedStyle(button);
    const toast = button.closest(".toast");
    if (!toast) throw new Error("Missing toast");
    const toastStyle = getComputedStyle(toast);
    return {
      backgroundColor: style.backgroundColor,
      color: style.color,
      filter: style.filter,
      maskImage: style.maskImage,
      toastColor: toastStyle.color,
    };
  });
  expect(closeStyle.filter).toBe("none");
  expect(closeStyle.maskImage).not.toBe("none");
  expect(closeStyle.color).toBe(closeStyle.toastColor);
  expect(closeStyle.backgroundColor).toBe(closeStyle.toastColor);
  await page.evaluate(() => {
    window.history.replaceState({ toastTest: true }, "", window.location.href);
  });
  await dismissible.getByRole("button", { name: "Close notification" }).click();
  await expect(dismissible).toBeHidden();
  expect(await page.evaluate(() => window.history.state)).toEqual({
    toastTest: true,
  });
  await page.getByRole("button", { name: "Show dismissible error" }).click();
  await expect(dismissible).toBeVisible();
  await dismissible.getByRole("button", { name: "Close notification" }).click();
  await page
    .getByRole("button", { name: "Show non-dismissible status" })
    .click();
  await expect(
    page.locator("#toast-nondismissible").getByRole("button"),
  ).toHaveCount(0);

  await page.getByRole("button", { name: "Show rich notifications" }).click();
  await expect(page.locator("#toast-markdown strong")).toHaveText("2.0");
  await expect(page.locator("#toast-markdown a")).toHaveAttribute(
    "href",
    "https://example.com/releases",
  );
  const linkStyle = await page.locator("#toast-markdown a").evaluate((link) => {
    if (!link.parentElement) throw new Error("Missing toast link parent");
    const style = getComputedStyle(link);
    return {
      color: style.color,
      parentColor: getComputedStyle(link.parentElement).color,
      textDecorationLine: style.textDecorationLine,
    };
  });
  expect(linkStyle.color).toBe(linkStyle.parentColor);
  expect(linkStyle.textDecorationLine).toBe("underline");
  await expect(page.locator("#toast-plain strong")).toHaveCount(0);
  await expect(page.locator("#toast-plain")).toContainText(
    "<strong>Plain text stays escaped</strong>",
  );
  const whiteToast = page.locator("#toast-plain");
  const whiteToastStyle = await whiteToast
    .locator(".btn-close")
    .evaluate((close) => {
      const toast = close.closest(".toast");
      if (!toast) throw new Error("Missing toast");
      const style = getComputedStyle(toast);
      const closeStyle = getComputedStyle(close);
      const rgba = (color: string) => {
        const canvas = document.createElement("canvas");
        const context = canvas.getContext("2d");
        if (!context) throw new Error("Canvas 2D context is unavailable");
        context.fillStyle = color;
        context.fillRect(0, 0, 1, 1);
        return Array.from(context.getImageData(0, 0, 1, 1).data);
      };
      return {
        backgroundColor: rgba(style.backgroundColor),
        closeColor: rgba(closeStyle.backgroundColor),
        color: rgba(style.color),
      };
    });
  expect(whiteToastStyle.backgroundColor).toEqual([255, 255, 255, 255]);
  expect(whiteToastStyle.color).toEqual([31, 41, 55, 255]);
  expect(whiteToastStyle.closeColor).toEqual(whiteToastStyle.color);

  const bottomContainer = page.locator(
    '[data-sqlpage-toast-position="bottom-center"]',
  );
  await page.getByRole("button", { name: "Show bottom notification" }).click();
  await expect(page.locator("#toast-bottom-center")).toBeVisible();
  await expect(bottomContainer).toHaveClass(/\bbottom-0\b/);
  await expect(bottomContainer).toHaveClass(/\bstart-50\b/);
  await expect(bottomContainer).toHaveClass(/\btranslate-middle-x\b/);
}

export async function checkTableFiltering(page: Page, fragment = false) {
  const tableSection = page.locator(".card", {
    has: page.getByRole("cell", { name: "Chart", exact: true }),
  });

  if (fragment) {
    await tableSection
      .locator(".card-body")
      .dispatchEvent("fragment-loaded", { bubbles: true });
    await expect(page.locator('[data-pre-init="table"]')).not.toHaveCount(0);
  }
  const searchInput = tableSection.getByPlaceholder("Search…");
  await searchInput.fill("chart");
  const chartCell = tableSection.getByRole("cell", { name: "Chart" });
  await expect(chartCell).toBeVisible();
  await expect(chartCell).toHaveClass(/\b_col_name\b/);
  await expect(chartCell).toHaveCSS("vertical-align", "middle");
  await expect(
    tableSection.getByRole("cell", { name: "Table" }),
  ).not.toBeVisible();
}

const numbersInColumn = async (table: Locator, cells: string) => {
  const texts = await table.locator(cells).allInnerTexts();
  expect(texts.length).toBeGreaterThan(1);
  return texts.map((text) => Number.parseInt(text.replace(/[^0-9]/g, ""), 10));
};

const ascending = (values: number[]) => [...values].sort((a, b) => a - b);

export async function checkTableSort(
  page: Page,
  column: "id" | "Amount in stock",
  reverse = false,
) {
  const table = page.locator(".table-responsive", {
    has: page.getByRole("cell", { name: "31456" }),
  });
  await table.getByRole("button", { name: column }).click();
  if (reverse) await table.getByRole("button", { name: column }).click();
  const values = await numbersInColumn(
    table,
    column === "id" ? "td._col_id" : "td._col_Amount_in_stock",
  );
  expect(values).toEqual(
    reverse ? ascending(values).reverse() : ascending(values),
  );
}

export async function checkModal(page: Page) {
  await expect(page.locator("body > #my_modal")).toBeAttached();
  await expect(page.locator("body > #my_embed_form_modal")).toBeAttached();
  const openButton = page.getByRole("button", { name: "Open a simple modal" });
  await openButton.click();

  const modal = page.getByRole("dialog", { name: "A modal box" });
  await expect(modal).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(modal).not.toBeVisible();

  await openButton.click();
  await expect(modal).toBeVisible();
  await modal.getByRole("button", { name: "Close" }).first().click();
  await expect(modal).not.toBeVisible();
}

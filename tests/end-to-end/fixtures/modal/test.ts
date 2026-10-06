import { expect, test } from "../../fixture.ts";

test("opens a colored modal once on load", async ({ page }) => {
  const modal = page.locator("#notice");
  const header = modal.locator(".modal-header");
  await expect(modal).toBeVisible();
  await expect(modal.getByText("It works !")).toBeVisible();
  await expect(header).toHaveAttribute("class", /bg-green/);
  await expect(header).toHaveAttribute("class", /text-green-fg/);
  await expect(modal.locator(".modal-body")).toHaveAttribute(
    "class",
    "modal-body",
  );
  await expect(modal.locator(".modal-footer")).toHaveAttribute(
    "class",
    "modal-footer",
  );
  await expect(
    modal.locator(".modal-footer").getByRole("button", { name: "Close" }),
  ).toBeVisible();
  await expect(modal.locator("script")).toHaveCount(0);
  await expect(modal).not.toHaveAttribute("data-modal-visible");

  const headerColor = await header.evaluate(
    (element) => getComputedStyle(element).color,
  );
  await expect(header.locator(".btn-close")).toHaveCSS("color", headerColor);

  await page.keyboard.press("Escape");
  await expect(modal).toBeHidden();
  await page.evaluate(() => {
    document.dispatchEvent(new CustomEvent("fragment-loaded"));
  });
  await expect(modal).toBeHidden();
});

test("header close button dismisses a modal opened on load", async ({
  page,
}) => {
  const modal = page.locator("#notice");
  await expect(modal).toBeVisible();
  await modal.locator(".modal-header .btn-close").click();
  await expect(modal).toBeHidden();
});

test("stays closed until a hash link opens it", async ({ page }) => {
  for (const url of [
    "/modal/?scenario=closed",
    "/modal/?scenario=closed&visible=0",
  ]) {
    await page.goto(url);
    const modal = page.locator("#notice");
    await expect(modal).toBeHidden();
    await expect(modal).not.toHaveAttribute("data-modal-visible");
  }

  await page.goto("/modal/?scenario=closed");
  await page.getByRole("link", { name: "Open notice" }).click();
  const modal = page.locator("#notice");
  await expect(modal).toBeVisible();
  await expect(page).toHaveURL(/#notice$/);
  await page.keyboard.press("Escape");
  await expect(modal).toBeHidden();
  await expect(page).toHaveURL(/#$/);
});

test("maps white to the light foreground and ignores an empty color", async ({
  page,
}) => {
  await page.goto("/modal/?scenario=colors");
  await expect(page.locator("#white")).toBeVisible();
  await expect(page.locator("#white .modal-header")).toHaveAttribute(
    "class",
    /bg-white/,
  );
  await expect(page.locator("#white .modal-header")).toHaveAttribute(
    "class",
    /text-light-fg/,
  );
  await expect(page.locator("#omitted .modal-header")).toHaveAttribute(
    "class",
    "modal-header",
  );
  await expect(page.locator("#empty .modal-header")).toHaveAttribute(
    "class",
    "modal-header",
  );
});

test("opens only the later visible modal", async ({ page }) => {
  await page.goto("/modal/?scenario=two");
  await expect(page.locator("#second")).toBeVisible();
  await expect(page.locator("#first")).toBeHidden();
});

test("keeps a hashed modal instead of the visible one", async ({ page }) => {
  await page.goto("/modal/?scenario=hash-modal#other");
  await expect(page.locator("#other")).toBeVisible();
  await expect(page.locator("#notice")).toBeHidden();
});

test("opens the visible modal when the hash is not a modal", async ({
  page,
}) => {
  await page.goto("/modal/?scenario=hash-anchor#section");
  await expect(page.locator("#notice")).toBeVisible();
  await expect(page).toHaveURL(/#section$/);
});

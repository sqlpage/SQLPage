import { expect, test } from "@playwright/test";

test("landing page links work without JavaScript", async ({
  browser,
  baseURL,
}) => {
  const context = await browser.newContext({
    javaScriptEnabled: false,
    viewport: { width: 390, height: 844 },
    baseURL,
  });
  const page = await context.newPage();
  await page.goto("/index.sql");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("SELECT");
  await expect(page.locator(".scene-preview")).toBeVisible();
  await expect(page.getByRole("link", { name: "Run it" })).toHaveAttribute(
    "href",
    "/your-first-sql-website/",
  );
  await page.getByRole("link", { name: "Documentation", exact: true }).click();
  await expect(page).toHaveURL(/\/documentation.sql$/);
  await context.close();
});

test("mobile navigation survives a scene failure and respects reduced motion", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.route("https://cdn.jsdelivr.net/**", (route) => route.abort());
  await page.goto("/");
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-motion",
    "off",
  );
  await expect(page.locator(".scene-error")).toBeVisible();
  await expect(page.locator(".scene-preview")).toBeVisible();
  const trigger = page.getByRole("button", { name: "Open navigation" });
  await trigger.click();
  const dialog = page.getByRole("dialog", { name: "SQLPage" });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("link", { name: "Examples" })).toHaveAttribute(
    "href",
    "/examples/",
  );
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
  await page.getByRole("button", { name: "Try again" }).click();
  await expect(page.locator(".scene-error")).toBeVisible();
  await trigger.click();
  await dialog.getByRole("link", { name: "Documentation" }).click();
  await expect(page).toHaveURL(/\/documentation.sql$/);
});

test("landing scene initializes with its page script policy", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const response = await page.goto("/");
  expect(response?.headers()["content-security-policy"]).toBe(
    "script-src 'self' https://cdn.jsdelivr.net",
  );
  await expect(page.locator(".sqlpage-world")).toHaveAttribute(
    "data-scene",
    "ready",
    {
      timeout: 30_000,
    },
  );
  await expect(page.getByRole("application")).toHaveAttribute(
    "data-ready",
    "true",
  );
  await expect(page.locator(".scene-canvas canvas")).toBeVisible();
  expect(errors).toEqual([]);
});

import { expect, test } from "@playwright/test";

const BASE = "http://localhost:8080/";

test("Open documentation", async ({ page }) => {
  await page.goto(BASE);

  // Expect a title "to contain" a substring.
  await expect(page).toHaveTitle(/SQLPage.*/);

  // open the submenu
  await page.getByText("Documentation", { exact: true }).first().click();
  await page.getByText("All Components").click();
  const components = ["form", "map", "chart", "button"];
  for (const component of components) {
    await expect(
      page.getByRole("link", { name: component }).first(),
    ).toBeVisible();
  }
});

test("chart", async ({ page }) => {
  await page.goto(`${BASE}/documentation.sql?component=chart#component`);
  await expect(page.getByText("Loading...")).not.toBeVisible();
  await expect(page.locator(".apexcharts-canvas").first()).toBeVisible();
});

test("map", async ({ page }) => {
  await page.goto(`${BASE}/documentation.sql?component=map#component`);
  await expect(page.getByText("Loading...")).not.toBeVisible();
  await expect(page.locator(".leaflet-marker-icon").first()).toBeVisible();
});

test("form example", async ({ page }) => {
  await page.goto(`${BASE}/examples/multistep-form`);
  // Single selection matching the value or label
  await page.getByLabel("From").selectOption("Paris");
  await page.getByText("Next").click();
  await page.getByLabel(/\bTo\b/).selectOption("Mexico");
  await page.getByText("Next").click();
  await page.getByLabel("Number of Adults").fill("1");
  await page.getByText("Next").click();
  await page.getByLabel("Passenger 1 (adult)").fill("John Doe");
  await page.getByText("Book the flight").click();
  await expect(page.getByText("John Doe").first()).toBeVisible();
});

test("File upload", async ({ page }) => {
  await page.goto(BASE);
  await page.getByRole("button", { name: "Examples", exact: true }).click();
  await page.getByText("File uploads").click();
  const my_svg = '<svg><text y="20">Hello World</text></svg>';
  // @ts-ignore
  const buffer = Buffer.from(my_svg);
  await page.getByLabel("Picture").setInputFiles({
    name: "small.svg",
    mimeType: "image/svg+xml",
    buffer,
  });
  await page.getByRole("button", { name: "Upload picture" }).click();
  await expect(
    page.locator("img[src^=data]").first().getAttribute("src"),
  ).resolves.toBe(`data:image/svg+xml;base64,${buffer.toString("base64")}`);
});

test("Authentication example", async ({ page }) => {
  await page.goto(`${BASE}/examples/authentication/login.sql`);
  await expect(page.locator("h1", { hasText: "Authentication" })).toBeVisible();

  const usernameInput = page.getByLabel("Username");
  const passwordInput = page.getByLabel("Password");
  const loginButton = page.getByRole("button", { name: "Log in" });

  await expect(usernameInput).toBeVisible();
  await expect(passwordInput).toBeVisible();
  await expect(loginButton).toBeVisible();

  await usernameInput.fill("admin");
  await passwordInput.fill("admin");
  await loginButton.click();

  await expect(page.getByText("You are logged in as admin")).toBeVisible();
});

test("table filtering", async ({ page }) => {
  await page.goto(`${BASE}/documentation.sql?component=table`);
  const tableSection = page.locator(".table-responsive", {
    has: page.getByRole("cell", { name: "Chart" }),
  });

  const searchInput = tableSection.getByPlaceholder("Search…");
  await searchInput.fill("chart");
  await expect(tableSection.getByRole("cell", { name: "Chart" })).toBeVisible();
  await expect(
    tableSection.getByRole("cell", { name: "Table" }),
  ).not.toBeVisible();
});

test("table sorting", async ({ page }) => {
  await page.goto(`${BASE}/documentation.sql?component=table`);
  const tableSection = page.locator(".table-responsive", {
    has: page.getByRole("cell", { name: "31456" }),
  });

  // Test numeric sorting on id column
  await tableSection.getByRole("button", { name: "id" }).click();
  let ids = await tableSection.locator("td.id").allInnerTexts();
  let numericIds = ids.map((id) => Number.parseInt(id));
  const sortedIds = [...numericIds].sort((a, b) => a - b);
  expect(numericIds).toEqual(sortedIds);

  // Test reverse sorting
  await tableSection.getByRole("button", { name: "id" }).click();
  ids = await tableSection.locator("td.id").allInnerTexts();
  numericIds = ids.map((id) => Number.parseInt(id));
  const reverseSortedIds = [...numericIds].sort((a, b) => b - a);
  expect(numericIds).toEqual(reverseSortedIds);

  // Test amount in stock column sorting
  await tableSection.getByRole("button", { name: "Amount in stock" }).click();
  const amounts = await tableSection.locator("td.Amount").allInnerTexts();
  const numericAmounts = amounts.map((amount) => Number.parseInt(amount));
  const sortedAmounts = [...numericAmounts].sort((a, b) => a - b);
  expect(numericAmounts).toEqual(sortedAmounts);
});

test("no console errors on table page", async ({ page }) => {
  const errors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") {
      errors.push(msg.text());
    }
  });

  await page.goto(`${BASE}/documentation.sql?component=table`);
  await page.waitForLoadState("networkidle");

  expect(errors).toHaveLength(0);
});

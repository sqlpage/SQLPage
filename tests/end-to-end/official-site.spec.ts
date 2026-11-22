import { expect, type Locator, type Page, test } from "@playwright/test";
import {
  checkModal,
  checkTableFiltering,
  checkTableSort,
  checkToastNotifications,
} from "./component-assertions.ts";

test("Open documentation", async ({ page }) => {
  await page.goto("/");

  await expect(page).toHaveTitle(/SQLPage.*/);

  await page.getByText("Documentation", { exact: true }).first().click();
  const components = ["form", "map", "chart", "button"];
  for (const component of components) {
    await expect(
      page.getByRole("link", { name: component }).first(),
    ).toBeVisible();
  }
});

async function expectContrast(
  locator: Locator,
  minimum: number,
  marker = false,
) {
  await expect(locator).toBeVisible();
  const ratio = await locator.evaluate((element, marker) => {
    // Normalize computed CSS colors and composite transparent fills onto their surfaces.
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 1;
    const context = canvas.getContext("2d", { willReadFrequently: true });
    if (!context) throw new Error("Could not read computed colors");

    function luminance(ctx: CanvasRenderingContext2D) {
      const [red = 0, green = 0, blue = 0] = ctx.getImageData(0, 0, 1, 1).data;
      const linearize = (channel: number) => {
        const value = channel / 255;
        return value <= 0.04045
          ? value / 12.92
          : ((value + 0.055) / 1.055) ** 2.4;
      };
      return (
        0.2126 * linearize(red) +
        0.7152 * linearize(green) +
        0.0722 * linearize(blue)
      );
    }

    const backgrounds = [];
    for (
      let parent: Element | null = element;
      parent;
      parent = parent.parentElement
    ) {
      backgrounds.push(getComputedStyle(parent).backgroundColor);
    }
    for (const background of backgrounds.reverse()) {
      context.fillStyle = background;
      context.fillRect(0, 0, 1, 1);
    }
    const background = luminance(context);
    context.fillStyle = marker
      ? getComputedStyle(element, "::before").backgroundColor
      : getComputedStyle(element).color;
    context.fillRect(0, 0, 1, 1);
    const foreground = luminance(context);
    return (
      (Math.max(foreground, background) + 0.05) /
      (Math.min(foreground, background) + 0.05)
    );
  }, marker);
  expect(ratio, `contrast for ${locator}`).toBeGreaterThanOrEqual(minimum);
}

test("active page numbers have readable contrast in every pagination style", async ({
  page,
}) => {
  await page.goto("/component.sql?component=pagination");
  const activePages = page.locator(".pagination .active > .page-link");
  await expect(activePages.first()).toBeVisible();
  for (const link of await activePages.all()) {
    await expectContrast(link, 4.5);
  }
  await expectContrast(page.locator(".pagination .page-link.disabled"), 4.5);
});

test("the official theme keeps filled controls and alerts readable", async ({
  page,
}) => {
  await page.goto("/component.sql?component=form");
  const checkedSwitch = page
    .locator(".form-switch .form-check-input:checked")
    .first();
  await expectContrast(checkedSwitch, 3, true);

  await page.goto("/component.sql?component=button");
  const info = page
    .getByText("Info", { exact: true })
    .and(page.locator(".btn-info"));
  await expectContrast(info, 4.5);
  await info.hover();
  await expectContrast(info, 4.5);

  await page.goto("/component.sql?component=alert");
  await expectContrast(
    page.locator(".alert-important .alert-title").first(),
    4.5,
  );
  await expectContrast(page.locator(".alert-important .alert-link"), 4.5);
});

test("light color swatches and homepage links have readable labels", async ({
  page,
}) => {
  await page.goto("/colors.sql");
  await expectContrast(
    page.getByRole("heading", { name: "white", exact: true }),
    4.5,
  );

  await page.goto("/");
  await expectContrast(page.locator(".demo-docs"), 4.5);
});

test("browse, filter and copy a component example on a phone", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/your-first-sql-website/");
  await page
    .getByRole("navigation", { name: "Main navigation" })
    .getByRole("link", { name: "Components" })
    .click();
  await page.locator("summary", { hasText: "Components" }).click();
  const topics = page.getByRole("navigation", { name: "Browse components" });
  await topics
    .getByRole("searchbox", { name: "Filter components" })
    .fill("butt");
  await expect(
    topics.getByRole("link", { name: "chart", exact: true }),
  ).toBeHidden();
  await topics.getByRole("link", { name: "button", exact: true }).click();

  await page.locator("summary", { hasText: "On this page" }).click();
  await page
    .getByRole("navigation", { name: "On this page" })
    .getByRole("link", { name: "Examples" })
    .click();
  await expect(page).toHaveURL(/#examples$/);
  const example = page.locator("#example-1");
  await example.getByRole("button", { name: "Copy SQL" }).click();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
    await example.locator("pre code").textContent(),
  );
  await expect(
    page.getByRole("button", { name: "Enabled", exact: true }),
  ).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(
    390,
  );
});

test("search a guide and follow its link to the function reference", async ({
  page,
}) => {
  await page.goto("/documentation.sql");
  const search = page.getByRole("searchbox", {
    name: "Search documentation",
    exact: true,
  });
  await search.fill("first website");
  await search.press("Enter");
  await page
    .getByRole("main")
    .getByRole("link", { name: /Your first website/ })
    .click();
  await page.getByRole("link", { name: "Read request variables" }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(
    /sqlpage\.variables\(/,
  );
});

test("chart documentation renders its examples without browser errors", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });

  await test.step("Open the chart reference", async () => {
    await page.goto("/documentation.sql?component=chart#component");
    await expect(
      page.getByRole("heading", { name: "The chart component", exact: true }),
    ).toBeVisible();
  });

  await test.step("Render every inline and embedded example", async () => {
    // Work around Chromium's slow :has() invalidation on this many-chart page.
    // https://issues.chromium.org/issues/576187899
    await expect(page.getByText("Loading...")).toHaveCount(0, {
      timeout: 15_000,
    });
    await expect(page.locator('[data-pre-init="chart"]')).toHaveCount(0);
    const examples = page.locator(
      '[data-sqlpage-js*="apexcharts."], [data-embed]',
    );
    await expect(examples.first()).toBeVisible();
    for (const example of await examples.all()) {
      await expect(example.locator(".apexcharts-canvas")).toBeVisible();
    }
    expect(errors).toEqual([]);
  });
});

test("map", async ({ page }) => {
  await page.goto("/documentation.sql?component=map#component");
  await expect(page.getByText("Loading...")).not.toBeVisible();
  await expect(page.locator(".leaflet-marker-icon").first()).toBeVisible();
});

test("toast notifications initialize, stack, dismiss, and render safely (page)", async ({
  page,
}) => {
  await page.goto("/documentation.sql?component=toast#component");
  await checkToastNotifications(page);
});

test("form example", async ({ page }) => {
  await page.goto("/examples/multistep-form");
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
  await page.goto("/your-first-sql-website");
  await page.getByRole("link", { name: "Examples", exact: true }).click();
  await page.getByText("File uploads").click();
  const my_svg = '<svg><text y="20">Hello World</text></svg>';
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
  await page.goto("/examples/authentication/login.sql");
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

test("table filtering (page)", async ({ page }) => {
  await page.goto("/documentation.sql?component=table");
  await checkTableFiltering(page);
});

test("table sorts a column when its header is clicked (page)", async ({
  page,
}) => {
  await page.goto("/documentation.sql?component=table");
  await checkTableSort(page, "id");
});

test("table reverses the sort when the header is clicked again (page)", async ({
  page,
}) => {
  await page.goto("/documentation.sql?component=table");
  await checkTableSort(page, "id", true);
});

test("table sorts a column of formatted numbers by value (page)", async ({
  page,
}) => {
  await page.goto("/documentation.sql?component=table");
  await checkTableSort(page, "Amount in stock");
});

async function checkNoConsoleErrors(page: Page, component: string) {
  const errors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") {
      errors.push(msg.text());
    }
  });

  await page.goto(`/documentation.sql?component=${component}`);
  await page.waitForLoadState();

  expect(errors).toHaveLength(0);
}

test("no console errors on table page", async ({ page }) => {
  await checkNoConsoleErrors(page, "table");
});

test("no console errors on map page", async ({ page }) => {
  await checkNoConsoleErrors(page, "map");
});

test("no console errors on card page", async ({ page }) => {
  await checkNoConsoleErrors(page, "card");
});

test("CSP issues unique nonces per request", async ({ page }) => {
  const csp1 = await (await page.goto("/"))?.headerValue(
    "content-security-policy",
  );
  const csp2 = await (await page.reload())?.headerValue(
    "content-security-policy",
  );

  expect(csp1, `check if ${csp1} != ${csp2}`).not.toEqual(csp2);
});

test("form component documentation", async ({ page }) => {
  await page.goto("/component.sql?component=form");

  const componentForm = page.locator("form", {
    has: page.getByRole("radio", { name: "Chart" }),
  });

  await expect(componentForm).toBeVisible();

  const mapRadio = componentForm.getByRole("radio", { name: "Map" });
  await expect(mapRadio).toHaveValue("map");
  await expect(mapRadio).toBeChecked();

  await componentForm.getByLabel("Chart").click({ force: true });
  await componentForm.getByRole("button", { name: "Submit" }).click();

  await expect(
    page.getByRole("heading", { name: /chart/i, level: 1 }),
  ).toBeVisible();
});

test("form select combines initial options with remote search results", async ({
  page,
}) => {
  await page.goto("/component.sql?component=form");

  const select = page
    .locator(
      'select[data-options_source="examples/from_component_options_source.sql?category=component"]',
    )
    .first();
  await expect(select).toBeAttached();
  await page.waitForFunction(
    () =>
      !!document.querySelector<HTMLSelectElement>(
        'select[data-options_source="examples/from_component_options_source.sql?category=component"]',
      )?.tomselect,
  );

  const initialState = await select.evaluate((element: HTMLSelectElement) => {
    const tomselect = element.tomselect;
    return {
      value: tomselect?.getValue(),
      labels: Object.fromEntries(
        Object.entries(tomselect?.options ?? {}).map(([value, option]) => [
          value,
          option?.label,
        ]),
      ),
    };
  });
  expect(initialState).toEqual({
    value: "form",
    labels: { form: "Form" },
  });

  await select.evaluate((element: HTMLSelectElement) =>
    element.tomselect?.focus(),
  );
  const formResponse = page.waitForResponse((response) =>
    response
      .url()
      .includes(
        "examples/from_component_options_source.sql?category=component&search=form",
      ),
  );
  await page.keyboard.type("form");
  await formResponse;
  await expect
    .poll(async () =>
      select.evaluate((element: HTMLSelectElement) => ({
        value: element.tomselect?.getValue(),
        formLabel: element.tomselect?.options.form?.label,
      })),
    )
    .toEqual({
      value: "form",
      formLabel: "form",
    });

  await select.evaluate((element: HTMLSelectElement) =>
    element.tomselect?.setTextboxValue(""),
  );
  const mapResponse = page.waitForResponse((response) =>
    response
      .url()
      .includes(
        "examples/from_component_options_source.sql?category=component&search=map",
      ),
  );
  await page.keyboard.type("map");
  await mapResponse;
  await expect
    .poll(async () =>
      select.evaluate((element: HTMLSelectElement) => ({
        value: element.tomselect?.getValue(),
        formLabel: element.tomselect?.options.form?.label,
        mapLabel: element.tomselect?.options.map?.label,
      })),
    )
    .toEqual({
      value: "form",
      formLabel: "form",
      mapLabel: "map",
    });
});

test("form type=select searchable=true", async ({ page }) => {
  await page.goto("/examples/form");

  const form = page.locator("form").filter({
    has: page.locator('select[name="region"]'),
  });
  const regionSelect = form.locator('select[name="region"]');
  const regionField = form.locator("label").filter({
    has: page.locator('select[name="region"]'),
  });
  const regionCombobox = regionField.locator('input[role="combobox"]');
  const dropdown = regionField.getByRole("listbox");
  const selectedRegion = (name: string) =>
    regionField.getByText(name, { exact: true }).filter({ visible: true });

  await expect(selectedRegion("North America")).toBeVisible();
  await expect(regionSelect).toHaveValue("NA");

  await selectedRegion("North America").click();
  await expect(dropdown).toBeVisible();
  await expect(dropdown.getByRole("option")).toHaveCount(3);

  await regionCombobox.fill("south");
  await expect(dropdown.getByRole("option")).toHaveCount(1);
  const southAmerica = dropdown.getByRole("option", {
    name: "South America",
    exact: true,
  });
  await expect(southAmerica).toBeVisible();

  await southAmerica.click();
  await page.evaluate(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      ),
  );
  await expect(dropdown).not.toBeVisible();
  await expect(regionCombobox).toHaveAttribute("aria-expanded", "false");
  await expect(regionSelect).toHaveValue("SA");
  await expect(selectedRegion("South America")).toBeVisible();

  const terms = form.getByLabel("I accept the terms and conditions");
  await form
    .locator("label")
    .filter({ has: page.locator('input[name="terms"]') })
    .click();
  await expect(terms).toBeChecked();
  await form.getByRole("button", { name: /submit/i }).click();

  await expect(page).toHaveURL(/\/examples\/show_variables\.sql$/);
  await expect(page.getByText(":region = SA", { exact: true })).toBeVisible();
});

test("modal (page)", async ({ page }) => {
  await page.goto("/documentation.sql?component=modal#component");
  await checkModal(page);
});

test("table action buttons - edit_url and delete_url", async ({ page }) => {
  await page.goto("/documentation.sql?component=table");
  const tableSection = page.locator(".table-responsive", {
    has: page.getByRole("cell", { name: "PharmaCo" }),
  });

  const editButton = tableSection.getByTitle("Edit").first();
  await expect(editButton).toBeVisible();
  await expect(editButton).toHaveAttribute("href", /action=edit&update_id=\d+/);

  const deleteButton = tableSection.getByTitle("Delete").first();
  await expect(deleteButton).toBeVisible();
  await expect(deleteButton).toHaveAttribute(
    "href",
    /action=delete&delete_id=\d+/,
  );
});

test("table action buttons - custom_actions", async ({ page }) => {
  await page.goto("/documentation.sql?component=table");
  const tableSection = page.locator(".table-responsive", {
    has: page.getByRole("cell", { name: "PharmaCo" }),
  });

  const historyButton = tableSection
    .getByTitle("View Standard History")
    .first();
  await expect(historyButton).toBeVisible();
  await expect(historyButton).toHaveAttribute(
    "href",
    /action=history&standard_id=\d+/,
  );
});

test("table action buttons - _sqlpage_actions", async ({ page }) => {
  await page.goto("/documentation.sql?component=table");
  const tableSection = page.locator(".table-responsive", {
    has: page.getByRole("cell", { name: "PharmaCo" }),
  });

  const pdfButtons = tableSection.getByTitle("View Presentation");
  await expect(pdfButtons.first()).toBeVisible();
  await expect(pdfButtons).toHaveCount(3);

  const firstPdfButton = pdfButtons.first();
  await expect(firstPdfButton).toHaveAttribute(
    "href",
    "https://sql-page.com/pgconf/2024-sqlpage-badass.pdf",
  );

  const setInUseButton = tableSection.getByTitle("Set In Use");
  await expect(setInUseButton).toBeVisible();
  await expect(setInUseButton).toHaveAttribute(
    "href",
    /action=set_in_use&standard_id=32/,
  );

  const retireButton = tableSection.getByTitle("Retire Standard");
  await expect(retireButton).toBeVisible();
  await expect(retireButton).toHaveAttribute(
    "href",
    /action=retire&standard_id=33/,
  );
});

test("table action buttons - disabled action", async ({ page }) => {
  await page.goto("/documentation.sql?component=table");
  const tableSection = page.locator(".table-responsive", {
    has: page.getByRole("cell", { name: "PharmaCo" }),
  });

  const viewPresentationButtons = tableSection.getByTitle("View Presentation");
  await expect(viewPresentationButtons).toHaveCount(3);

  const actionColumnButtons = tableSection.locator(
    "td._col_Action a[data-action='Action']",
  );
  await expect(actionColumnButtons).toHaveCount(3);

  const emptyActionButton = actionColumnButtons.last();
  await expect(emptyActionButton).toHaveAttribute("href", "null");
  await expect(emptyActionButton).toHaveAttribute("title", "Action");
});

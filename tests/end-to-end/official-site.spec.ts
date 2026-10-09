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

test("chart", async ({ page }) => {
  await page.goto("/documentation.sql?component=chart#component");
  await expect(page.getByText("Loading...")).not.toBeVisible();
  await expect(page.locator(".apexcharts-canvas").first()).toBeVisible();
});

test("chart supports hiding legend", async ({ page }) => {
  await page.goto("/documentation.sql?component=chart#component");

  const expensesChart = page.locator(".card", {
    has: page.getByRole("heading", { name: "Expenses" }),
  });

  await expect(expensesChart.locator(".apexcharts-canvas")).toBeVisible();
  await expect(expensesChart.locator(".apexcharts-legend")).toBeHidden();
});

const DATA_POINT_MARKERS = ".apexcharts-series-markers > .apexcharts-marker";

const chartCard = (page: Page, title: string) =>
  page.locator(".card", { has: page.getByRole("heading", { name: title }) });

const drawnPoints = (card: Locator, series: string) =>
  card
    .locator(`.apexcharts-series[seriesName='${series}'] ${DATA_POINT_MARKERS}`)
    .evaluateAll((markers) =>
      markers.map((m) => ({
        x: m.getAttribute("cx"),
        y: m.getAttribute("cy"),
      })),
    );

test("stacked chart draws every series at every x of the chart", async ({
  page,
}) => {
  await page.goto("/documentation.sql?component=chart#component");
  const powerChart = chartCard(page, "Power draw");
  await expect(powerChart.locator(".apexcharts-canvas")).toBeVisible();

  const cpu = await drawnPoints(powerChart, "CPU");
  const gpu = await drawnPoints(powerChart, "GPU");

  expect(cpu).toHaveLength(4);
  expect(gpu.map((p) => p.x)).toEqual(cpu.map((p) => p.x));
});

test("stacked chart raises a series only where it has a value", async ({
  page,
}) => {
  await page.goto("/documentation.sql?component=chart#component");
  const powerChart = chartCard(page, "Power draw");
  await expect(powerChart.locator(".apexcharts-canvas")).toBeVisible();

  const cpu = await drawnPoints(powerChart, "CPU");
  const gpu = await drawnPoints(powerChart, "GPU");

  expect([gpu[0], gpu[3]]).toEqual([cpu[0], cpu[3]]);
  expect(Number(gpu[1]?.y)).toBeLessThan(Number(cpu[1]?.y));
});

test("chart draws a yline as a line and a yline_end as a band", async ({
  page,
}) => {
  await page.goto("/documentation.sql?component=chart#component");

  const temperature = page.locator(".card", {
    has: page.getByRole("heading", { name: "CPU temperature" }),
  });
  await expect(temperature.locator(".apexcharts-canvas")).toBeVisible();

  const annotations = temperature.locator(".apexcharts-yaxis-annotations");
  const lines = annotations.locator("line");
  const bands = annotations.locator(".apexcharts-annotation-rect");

  await expect(lines).toHaveCount(1);
  await expect(bands).toHaveCount(1);
  await expect(annotations.getByText("target")).toBeVisible();
  await expect(annotations.getByText("throttling")).toBeVisible();
});

test("chart draws an xline as a line and an xline_end as a band", async ({
  page,
}) => {
  await page.goto("/documentation.sql?component=chart#component");

  const latency = page.locator(".card", {
    has: page.getByRole("heading", { name: "Request latency" }),
  });
  await expect(latency.locator(".apexcharts-canvas")).toBeVisible();

  const annotations = latency.locator(".apexcharts-xaxis-annotations");
  const lines = annotations.locator("line");
  const bands = annotations.locator(".apexcharts-annotation-rect");

  await expect(lines).toHaveCount(1);
  await expect(bands).toHaveCount(1);
  await expect(annotations.getByText("deploy")).toBeVisible();
  await expect(annotations.getByText("incident")).toBeVisible();
});

test("horizontal chart draws a yline down it and an xline across it", async ({
  page,
}) => {
  await page.goto("/documentation.sql?component=chart#component");

  const disks = page.locator(".card", {
    has: page.getByRole("heading", { name: "Disk usage" }),
  });
  await expect(disks.locator(".apexcharts-canvas")).toBeVisible();

  const down = disks.locator(".apexcharts-xaxis-annotations");
  const across = disks.locator(".apexcharts-yaxis-annotations");

  await expect(down.locator("line")).toHaveCount(1);
  await expect(down.getByText("full")).toBeVisible();
  await expect(across.locator("line")).toHaveCount(1);
  await expect(across.getByText("watched")).toBeVisible();
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
  await page.getByRole("button", { name: "Examples", exact: true }).click();
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

test("no console errors on chart page", async ({ page }) => {
  await checkNoConsoleErrors(page, "chart");
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
  await page.keyboard.type("form");
  await page.waitForResponse((response) =>
    response
      .url()
      .includes(
        "examples/from_component_options_source.sql?category=component&search=form",
      ),
  );
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
  await page.keyboard.type("map");
  await page.waitForResponse((response) =>
    response
      .url()
      .includes(
        "examples/from_component_options_source.sql?category=component&search=map",
      ),
  );
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

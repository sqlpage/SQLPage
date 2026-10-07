import { expect, type Page, test } from "../../fixture.ts";

/** Load server-rendered SQL output as a custom fragment consumer would. */
async function appendFragment(
  page: Page,
  file: string,
  root?: string,
  instance?: string,
) {
  await page.evaluate(
    async ({ file, root, instance }) => {
      const url = new URL(`/fragment-loaded/${file}.sql`, location.href);
      url.searchParams.set("_sqlpage_embed", "1");
      if (instance) url.searchParams.set("instance", instance);
      const response = await fetch(url);
      if (!response.ok)
        throw new Error(`Fragment request failed: ${response.status}`);
      const fragment = document.createElement("template");
      fragment.innerHTML = await response.text();
      document.querySelector("main")?.append(fragment.content);
      if (root) {
        const target =
          root === "document" ? document : document.querySelector(root);
        if (!target) throw new Error(`Missing fragment root: ${root}`);
        target.dispatchEvent(
          new CustomEvent("fragment-loaded", { bubbles: true }),
        );
      }
    },
    { file, root, instance },
  );
}

async function countSubmissions(page: Page, selector: string) {
  await page.locator(selector).evaluate((element) => {
    const form = element as HTMLFormElement;
    form.dataset.submissions = "0";
    form.submit = () => {
      form.dataset.submissions = String(Number(form.dataset.submissions) + 1);
    };
  });
}

// Card fetches and their chart/select scripts are asynchronous. Wait until
// those scripts complete their initial document pass before testing local events.
test.beforeEach(async ({ page }) => {
  await expect(page.locator(".apexcharts-canvas")).toHaveCount(2);
  await expect(page.locator(".ts-wrapper")).toHaveCount(2);
});

for (const root of ["document", "main", "#added"]) {
  test(`initializes a tooltip when ${root} announces a fragment`, async ({
    page,
  }) => {
    await appendFragment(page, "tooltip", root);
    // Re-announcing a fragment must retain the existing Bootstrap instance.
    await page.evaluate(() =>
      document.dispatchEvent(new CustomEvent("fragment-loaded")),
    );
    await page.locator("#added").hover();
    await expect(page.locator(".tooltip")).toHaveText("injected hint");
    await page.getByRole("heading", { name: "Existing form" }).hover();
    await expect(page.locator(".tooltip")).toHaveCount(0);
  });
}

test("initializes card fragments and keeps existing form handlers singular", async ({
  page,
}) => {
  await expect(page.locator(".apexcharts-canvas")).toHaveCount(2);
  await expect(page.locator(".ts-wrapper")).toHaveCount(2);
  await expect(page.locator("body > #first-modal")).toBeAttached();
  await expect(page.locator("body > #second-modal")).toBeAttached();
  await expect(
    page.getByText("first notification", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("second notification", { exact: true }),
  ).toBeVisible();
  await countSubmissions(page, "#existing-form");
  await countSubmissions(page, "#first-form");
  await page.getByLabel("Existing value", { exact: true }).fill("one change");
  await page.getByLabel("Existing value", { exact: true }).blur();
  await expect(page.locator("#existing-form")).toHaveAttribute(
    "data-submissions",
    "1",
  );

  await page.evaluate(() => {
    for (let i = 0; i < 3; i++)
      document.dispatchEvent(new CustomEvent("fragment-loaded"));
    const input = document.querySelector<HTMLInputElement>(
      'input[name="existing_file"]',
    );
    if (!input) throw new Error("Missing file input");
    const setValidity = input.setCustomValidity.bind(input);
    input.dataset.validations = "0";
    input.setCustomValidity = (message) => {
      input.dataset.validations = String(Number(input.dataset.validations) + 1);
      setValidity(message);
    };
  });
  await page.getByLabel("Existing file", { exact: true }).setInputFiles({
    name: "small.txt",
    mimeType: "text/plain",
    buffer: Buffer.from("small"),
  });
  await expect(
    page.getByLabel("Existing file", { exact: true }),
  ).toHaveAttribute("data-validations", "1");
  await expect(page.locator("#existing-form")).toHaveAttribute(
    "data-submissions",
    "2",
  );
  await page.getByLabel("first value", { exact: true }).fill("changed");
  await page.getByLabel("first value", { exact: true }).blur();
  await expect(page.locator("#first-form")).toHaveAttribute(
    "data-submissions",
    "1",
  );

  await page.locator("#first-table input.search").fill("Alpha");
  await expect(page.locator("#first-table tbody tr:visible")).toHaveCount(1);
  await page.locator("#first-table input.search").fill("");
  await page.locator("#first-table button.sort").click();
  await expect(page.locator("#first-table tbody tr").first()).toContainText(
    "Alpha",
  );
  await page.getByRole("button", { name: "first choices" }).click();
  await expect(page.locator(".dropdown-menu.show")).toBeVisible();
  await page.getByRole("button", { name: "first choices" }).click();
  await expect(page.locator(".dropdown-menu.show")).toHaveCount(0);
  await page.evaluate(() => {
    window.location.hash = "first-modal";
  });
  await expect(page.getByRole("dialog")).toBeVisible();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "close", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("initializes a form root and its searchable select", async ({ page }) => {
  await appendFragment(page, "form", "#root-form");
  await expect(page.locator("#root-form .ts-wrapper")).toBeAttached();
  await countSubmissions(page, "#root-form");
  await page.getByLabel("Root value", { exact: true }).fill("changed");
  await page.getByLabel("Root value", { exact: true }).blur();
  await expect(page.locator("#root-form")).toHaveAttribute(
    "data-submissions",
    "1",
  );
});

test("initializes a file input root without initializing its surrounding form", async ({
  page,
}) => {
  await appendFragment(page, "form", 'input[name="root_file"]');
  await countSubmissions(page, "#root-form");
  await page.getByLabel("Root file", { exact: true }).setInputFiles({
    name: "small.txt",
    mimeType: "text/plain",
    buffer: Buffer.from("small"),
  });
  await expect(page.locator("#root-form")).toHaveAttribute(
    "data-submissions",
    "0",
  );
  await expect(
    page.locator("#root-form [data-pre-init=select-dropdown]"),
  ).toBeAttached();
  const input = page.getByLabel("Root file", { exact: true });
  const maxSize = await input.getAttribute("data-max-size");
  await input.setInputFiles({
    name: "large.txt",
    mimeType: "text/plain",
    buffer: Buffer.alloc(Number(maxSize) + 1),
  });
  await expect(input).toHaveClass(/is-invalid/);
  await expect(input).toHaveJSProperty(
    "validationMessage",
    `File size must be less than ${Number(maxSize) / 1000} kB.`,
  );
});

test("initializes the root table and chart without initializing unrelated fragments", async ({
  page,
}) => {
  await appendFragment(page, "form");
  await appendFragment(page, "table", "#root-table [data-pre-init=table]");
  await page.locator("#root-table input.search").fill("Alpha");
  await expect(page.locator("#root-table tbody tr:visible")).toHaveCount(1);
  await appendFragment(page, "chart", "#root-chart");
  await expect(page.locator("#root-chart .apexcharts-canvas")).toBeAttached();
  await expect(
    page.locator("#root-form [data-pre-init=select-dropdown]"),
  ).toBeAttached();
  await page.locator("#root-form select").evaluate((element) => {
    element.dispatchEvent(
      new CustomEvent("fragment-loaded", { bubbles: true }),
    );
  });
  await expect(page.locator("#root-form .ts-wrapper")).toBeAttached();
});

test("initializes a map root after loading Leaflet", async ({ page }) => {
  await appendFragment(page, "map", "#root-map .leaflet");
  await expect(page.locator("#root-map .leaflet-map-pane")).toBeAttached();
  await page.locator("#root-map").evaluate((element) => {
    element.dispatchEvent(
      new CustomEvent("fragment-loaded", { bubbles: true }),
    );
  });
  await expect(page.locator("#root-map .leaflet-map-pane")).toHaveCount(1);
});

test("initializes maps arriving while their shared dependency is pending", async ({
  page,
}) => {
  let releaseLeaflet = () => {};
  const dependencyReady = new Promise<void>((resolve) => {
    releaseLeaflet = resolve;
  });
  await page.route("**/dist/leaflet.js", async (route) => {
    await dependencyReady;
    await route.continue();
  });
  try {
    await appendFragment(page, "map", "#first-map .leaflet", "first-map");
    await appendFragment(page, "map", "#second-map .leaflet", "second-map");
  } finally {
    releaseLeaflet();
  }
  await expect(page.locator(".leaflet-map-pane")).toHaveCount(2);
  // With Leaflet loaded, a later event should initialize only its own root.
  await appendFragment(page, "map", "#third-map .leaflet", "third-map");
  await expect(page.locator(".leaflet-map-pane")).toHaveCount(3);
});

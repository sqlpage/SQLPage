import { expect, loadFragment, type Page, test } from "../../fixture.ts";

const rootForm = "/form/?fragment=1&id=root-form";
const announceDocument = (page: Page) =>
  page.evaluate(() =>
    document.dispatchEvent(new CustomEvent("fragment-loaded")),
  );

async function trackHandlers(page: Page, selector: string) {
  await page.locator(selector).evaluate((element) => {
    const form = element as HTMLFormElement;
    form.dataset.submissions = "0";
    form.submit = () => {
      form.dataset.submissions = String(Number(form.dataset.submissions) + 1);
    };
    const input = form.querySelector<HTMLInputElement>('input[type="file"]');
    if (!input) throw new Error("Missing file input");
    const setValidity = input.setCustomValidity.bind(input);
    input.dataset.validations = "0";
    input.setCustomValidity = (message) => {
      input.dataset.validations = String(Number(input.dataset.validations) + 1);
      setValidity(message);
    };
  });
}

// Card fetches and their chart/select scripts must finish the initial document pass.
test.beforeEach(async ({ page }) => {
  await expect(page.locator(".apexcharts-canvas")).toHaveCount(1);
  await expect(page.locator(".ts-wrapper")).toHaveCount(2);
});

for (const root of ["document", "main", "#added"]) {
  test(`initializes a tooltip when ${root} announces a fragment`, async ({
    page,
  }) => {
    await loadFragment(page, "/fragment-loaded/tooltip.sql", root);
    await page.locator("#added").hover();
    await expect(page.locator(".tooltip")).toHaveText("injected hint");
    await announceDocument(page);
    await expect(page.locator(".tooltip")).toHaveText("injected hint");
    await page.getByRole("heading", { name: "Embedded form" }).hover();
    await expect(page.locator(".tooltip")).toHaveCount(0);
  });
}

test("card and document events keep existing and embedded form handlers singular", async ({
  page,
}) => {
  await trackHandlers(page, "#existing-form");
  await trackHandlers(page, "#first-form");
  const existing = page.locator("#existing-form");
  await existing.getByLabel("Modern text field").fill("before repeated events");
  await existing.getByLabel("Modern text field").blur();
  await expect(existing).toHaveAttribute("data-submissions", "1");
  for (let i = 0; i < 3; i++) await announceDocument(page);
  const upload = existing.getByLabel("Upload", { exact: true });
  await upload.setInputFiles({
    name: "small.txt",
    mimeType: "text/plain",
    buffer: Buffer.from("small"),
  });
  await expect(upload).toHaveAttribute("data-validations", "1");
  await expect(existing).toHaveAttribute("data-submissions", "2");
  const embedded = page.locator("#first-form");
  await embedded.getByLabel("Modern text field").fill("changed");
  await embedded.getByLabel("Modern text field").blur();
  await expect(embedded).toHaveAttribute("data-submissions", "1");
});

for (const root of ["#root-form", '#root-form input[type="file"]']) {
  test(`initializes ${root} without adding handlers outside the root`, async ({
    page,
  }) => {
    await loadFragment(page, rootForm, root);
    await trackHandlers(page, "#root-form");
    const form = page.locator("#root-form");
    const isForm = root === "#root-form";
    await expect(form.locator(".ts-wrapper")).toHaveCount(isForm ? 1 : 0);
    await form.getByLabel("Modern text field").fill("changed");
    await form.getByLabel("Modern text field").blur();
    await expect(form).toHaveAttribute("data-submissions", isForm ? "1" : "0");
    const upload = form.getByLabel("Upload", { exact: true });
    await upload.setInputFiles({
      name: "small.txt",
      mimeType: "text/plain",
      buffer: Buffer.from("small"),
    });
    await expect(form).toHaveAttribute("data-submissions", isForm ? "2" : "0");
    const maxSize = Number(await upload.getAttribute("data-max-size"));
    await upload.setInputFiles({
      name: "large.txt",
      mimeType: "text/plain",
      buffer: Buffer.alloc(maxSize + 1),
    });
    await expect(upload).toHaveClass(/is-invalid/);
    await expect(upload).toHaveJSProperty(
      "validationMessage",
      `File size must be less than ${maxSize / 1000} kB.`,
    );
  });
}

test("chart and select roots leave unrelated fragments pending", async ({
  page,
}) => {
  await loadFragment(page, rootForm, null);
  await loadFragment(page, "/chart/", "main > #test-chart");
  await expect(
    page.locator("main > #test-chart .apexcharts-canvas"),
  ).toBeAttached();
  const select = page.locator("#root-form select");
  await expect(select).toHaveAttribute("data-pre-init", "select-dropdown");
  await select.dispatchEvent("fragment-loaded", { bubbles: true });
  await expect(page.locator("#root-form .ts-wrapper")).toBeAttached();
});

test("map roots initialize before and after their shared dependency loads", async ({
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
  await loadFragment(page, "/map/", null);
  const pending = page.locator("main > .card > .card-body > .leaflet").first();
  const root = "main > .card:last-child .leaflet";
  try {
    for (let i = 0; i < 2; i++) await loadFragment(page, "/map/", root);
  } finally {
    releaseLeaflet();
  }
  await expect(page.locator(".leaflet-map-pane")).toHaveCount(2);
  await expect(pending).toHaveAttribute("data-pre-init", "map");
  await loadFragment(page, "/map/", root);
  await expect(page.locator(".leaflet-map-pane")).toHaveCount(3);
  await page.locator(root).dispatchEvent("fragment-loaded", { bubbles: true });
  await expect(page.locator(".leaflet-map-pane")).toHaveCount(3);
  await expect(pending).toHaveAttribute("data-pre-init", "map");
  await pending.dispatchEvent("fragment-loaded", { bubbles: true });
  await expect(page.locator(".leaflet-map-pane")).toHaveCount(4);
});

for (const bundle of ["apexcharts", "tomselect"]) {
  test(`${bundle} initializes only announced roots while its bundle loads`, async ({
    page,
  }) => {
    await page.goto("/fragment-loaded/?lazy=1");
    const select = bundle === "tomselect";
    const url = select ? "/form/?fragment=1" : "/chart/";
    const component = select ? "form" : "[data-sqlpage-js]";
    const ready = select ? ".ts-wrapper" : ".apexcharts-canvas";
    await loadFragment(page, url, null);
    const pending = page.locator(`main > ${component}`).first();
    let releaseBundle = () => {};
    const gate = new Promise<void>((resolve) => {
      releaseBundle = resolve;
    });
    await page.route(`**/${bundle}.*.js`, async (route) => {
      await gate;
      await route.continue();
    });
    try {
      for (let i = 0; i < 2; i++)
        await loadFragment(page, url, `main > ${component}:last-child`);
    } finally {
      releaseBundle();
    }
    await expect(page.locator(ready)).toHaveCount(2);
    await expect(select ? pending.locator("select") : pending).toHaveAttribute(
      "data-pre-init",
      select ? "select-dropdown" : "chart",
    );
    await pending.dispatchEvent("fragment-loaded", { bubbles: true });
    await expect(page.locator(ready)).toHaveCount(3);
  });
}

for (const lazy of [false, true]) {
  test(`relocated modals initialize their contents with ${lazy ? "lazy" : "loaded"} bundles`, async ({
    page,
  }) => {
    if (lazy) await page.goto("/fragment-loaded/?lazy=1");
    await loadFragment(page, "/fragment-loaded/modal.sql", null);
    const parent = "#nested-modal .modal-body";
    for (const url of [
      rootForm,
      "/chart/",
      "/fragment-loaded/tooltip.sql",
      "/facet/",
      "/fragment-loaded/modal.sql?notification=1",
    ]) {
      await loadFragment(page, url, null, parent);
    }
    await page
      .locator("main")
      .dispatchEvent("fragment-loaded", { bubbles: true });
    await expect(page.locator("body > #nested-modal")).toBeAttached();
    await expect(page.locator("#nested-modal .ts-wrapper")).toBeAttached();
    await expect(
      page.locator("#nested-modal .apexcharts-canvas"),
    ).toBeAttached();
    await expect(page.locator("#nested-toast")).toBeVisible();
    await page.evaluate(() => {
      window.location.hash = "nested-modal";
    });
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.locator("#added").hover();
    await expect(page.locator(".tooltip")).toHaveText("injected hint");
    await page
      .getByRole("button", { name: "Constitution", exact: true })
      .click();
    await expect(page.locator(".dropdown-menu.show")).toBeVisible();
    await page
      .getByRole("button", { name: "Constitution", exact: true })
      .click();
    await expect(page.locator(".dropdown-menu.show")).toHaveCount(0);
  });
}

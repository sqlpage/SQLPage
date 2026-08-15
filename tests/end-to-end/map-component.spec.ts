import { expect, type Page, test } from "@playwright/test";

const BASE = process.env.SQLPAGE_TEST_BASE ?? "http://localhost:8080/";

type Marker = { coords?: string; title: string };

const PARIS = "48.85,2.35";
const PARIS_WITHOUT_ITS_LONGITUDE = "48.85,";
const NOT_COORDINATES = "somewhere nice";

async function renderMap(
  page: Page,
  center: string | null,
  markers: Marker[] = [],
) {
  return page.evaluate(
    async ({ center, markers }) => {
      document.getElementById("test-map")?.remove();
      const container = document.createElement("div");
      container.id = "test-map";
      container.className = "leaflet";
      container.style.height = "200px";
      container.dataset.zoom = "5";
      container.dataset.max_zoom = "18";
      if (center !== null) container.dataset.center = center;
      container.innerHTML = markers
        .map(
          (m) =>
            `<div class="marker"${m.coords === undefined ? "" : ` data-coords="${m.coords}"`}><h3>${m.title}</h3></div>`,
        )
        .join("");
      container.dataset.preInit = "map";
      document.body.appendChild(container);

      const errors: string[] = [];
      const record = (e: ErrorEvent) => errors.push(e.message);
      window.addEventListener("error", record);

      const logged: string[] = [];
      const console_error = console.error;
      console.error = (...args) => logged.push(args.join(" "));

      container.dispatchEvent(
        new CustomEvent("fragment-loaded", { bubbles: true }),
      );
      await new Promise((resolve) => setTimeout(resolve, 500));

      console.error = console_error;
      window.removeEventListener("error", record);

      return {
        errors,
        logged,
        markers: container.querySelectorAll(".leaflet-marker-icon").length,
        initialized: !!container.querySelector(".leaflet-map-pane"),
      };
    },
    { center, markers },
  );
}

test.beforeEach(async ({ page }) => {
  await page.goto(`${BASE}/documentation.sql?component=map#component`);
  await page.waitForFunction(() => "L" in window);
});

test("centers the map on a pair of coordinates", async ({ page }) => {
  const map = await renderMap(page, PARIS);

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([]);
  expect(map.initialized).toBe(true);
});

test("reports a center whose longitude is missing", async ({ page }) => {
  const map = await renderMap(page, PARIS_WITHOUT_ITS_LONGITUDE);

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([
    expect.stringContaining(PARIS_WITHOUT_ITS_LONGITUDE),
  ]);
  expect(map.initialized).toBe(true);
});

test("reports a center that is not a pair of numbers", async ({ page }) => {
  const map = await renderMap(page, NOT_COORDINATES);

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([expect.stringContaining(NOT_COORDINATES)]);
  expect(map.initialized).toBe(true);
});

test("draws a marker at a pair of coordinates", async ({ page }) => {
  const map = await renderMap(page, PARIS, [{ coords: PARIS, title: "Paris" }]);

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([]);
  expect(map.markers).toBe(1);
});

test("reports a marker whose longitude is missing, keeping the others", async ({
  page,
}) => {
  const map = await renderMap(page, PARIS, [
    { coords: PARIS_WITHOUT_ITS_LONGITUDE, title: "Half of Paris" },
    { coords: PARIS, title: "Paris" },
  ]);

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([
    expect.stringContaining(PARIS_WITHOUT_ITS_LONGITUDE),
  ]);
  expect(map.markers).toBe(1);
});

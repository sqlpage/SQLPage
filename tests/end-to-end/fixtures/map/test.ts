import { expect, type Page, test } from "../../fixture";

const PARIS_WITHOUT_ITS_LONGITUDE = "48.85,";
const NOT_COORDINATES = "somewhere nice";

async function renderMap(page: Page, fixture: string, markerCount = 0) {
  const errors: string[] = [];
  const logged: string[] = [];
  const recordPageError = (error: Error) => errors.push(error.message);
  const recordConsoleError = (message: { type(): string; text(): string }) => {
    if (message.type() === "error") logged.push(message.text());
  };
  page.on("pageerror", recordPageError);
  page.on("console", recordConsoleError);
  const response = await page.goto(`/map/${fixture}.sql`);
  expect(response?.ok(), `loading ${response?.url()}`).toBe(true);
  await expect(page.locator("[data-pre-init='map']")).toHaveCount(0);
  await expect(page.locator(".leaflet-map-pane")).toBeAttached();
  await expect(page.locator(".leaflet-marker-icon")).toHaveCount(markerCount);
  page.off("pageerror", recordPageError);
  page.off("console", recordConsoleError);

  return page.locator(".leaflet").evaluate(
    (container, messages) => ({
      ...messages,
      markers: container.querySelectorAll(".leaflet-marker-icon").length,
      initialized: !!container.querySelector(".leaflet-map-pane"),
    }),
    { errors, logged },
  );
}

test("centers the map on a pair of coordinates", async ({ page }) => {
  const map = await renderMap(page, "valid-center");

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([]);
  expect(map.initialized).toBe(true);
});

test("reports a center whose longitude is missing", async ({ page }) => {
  const map = await renderMap(page, "missing-center-longitude");

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([
    expect.stringContaining(PARIS_WITHOUT_ITS_LONGITUDE),
  ]);
  expect(map.initialized).toBe(true);
});

test("reports a center that is not a pair of numbers", async ({ page }) => {
  const map = await renderMap(page, "invalid-center");

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([expect.stringContaining(NOT_COORDINATES)]);
  expect(map.initialized).toBe(true);
});

test("draws a marker at a pair of coordinates", async ({ page }) => {
  const map = await renderMap(page, "valid-marker", 1);

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([]);
  expect(map.markers).toBe(1);
});

test("reports a marker whose longitude is missing, keeping the others", async ({
  page,
}) => {
  const map = await renderMap(page, "invalid-and-valid-markers", 1);

  expect(map.errors).toEqual([]);
  expect(map.logged).toEqual([
    expect.stringContaining(PARIS_WITHOUT_ITS_LONGITUDE),
  ]);
  expect(map.markers).toBe(1);
});

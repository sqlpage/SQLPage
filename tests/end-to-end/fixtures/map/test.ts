import { expect, type Page, test } from "../../fixture";

type Marker = { coords?: string; title: string };

const PARIS = "48.85,2.35";
const PARIS_WITHOUT_ITS_LONGITUDE = "48.85,";
const NOT_COORDINATES = "somewhere nice";

async function renderMap(
  page: Page,
  center: string | null,
  markers: Marker[] = [],
) {
  const errors: string[] = [];
  const logged: string[] = [];
  const recordPageError = (error: Error) => errors.push(error.message);
  const recordConsoleError = (message: { type(): string; text(): string }) => {
    if (message.type() === "error") logged.push(message.text());
  };
  page.on("pageerror", recordPageError);
  page.on("console", recordConsoleError);
  const coordinates = (coords: string) => {
    const [latitude, longitude = ""] = coords.split(",", 2);
    return { latitude, longitude };
  };
  const isValid = (coords: string) =>
    coords.split(",", 2).length === 2 &&
    coords
      .split(",", 2)
      .every((part) => Number.isFinite(Number.parseFloat(part)));
  const validMarkers = markers.filter(
    (marker) => marker.coords !== undefined && isValid(marker.coords),
  ).length;
  const invalidCoordinates =
    (center !== null && !isValid(center) ? 1 : 0) +
    markers.filter(
      (marker) => marker.coords !== undefined && !isValid(marker.coords),
    ).length;
  const query = new URLSearchParams({
    properties: JSON.stringify([
      {
        component: "map",
        title: "Map test fixture",
        height: 200,
        tile_source: false,
        ...(center === null ? {} : coordinates(center)),
      },
      ...markers.map(({ coords, ...marker }) => ({
        ...marker,
        ...(coords === undefined ? {} : coordinates(coords)),
      })),
    ]),
  });
  const response = await page.goto(`/map/?${query}`);
  expect(response?.ok(), `loading ${response?.url()}`).toBe(true);
  await expect(page.locator("[data-pre-init='map']")).toHaveCount(0);
  await expect(page.locator(".leaflet-map-pane")).toBeAttached();
  await expect(page.locator(".leaflet-marker-icon")).toHaveCount(validMarkers);
  await expect.poll(() => logged.length).toBe(invalidCoordinates);
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

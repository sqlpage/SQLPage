import { expect, type Page, test } from "@playwright/test";

const BASE = process.env.SQLPAGE_TEST_BASE ?? "http://localhost:8080/";

type ChartPoint = { x: string | number | Date; y: number | null };

declare global {
  interface Window {
    charts?: {
      w: {
        config: {
          chart: { type: string; stacked: boolean };
          series: { name: string; data: ChartPoint[] }[];
        };
      };
    }[];
  }
}

type Row = [series: string, x: unknown, y: unknown, z?: unknown];

const A_DAY_OF_WORK: Row[] = [
  ["Coding", "Mon", 6],
  ["Coding", "Tue", 4],
  ["Coding", "Wed", 7],
];

const TASKS_OVER_TIME: Row[] = [
  ["Design", "Alice", ["2024-03-01", "2024-03-05"]],
  ["Build", "Bob", ["2024-03-04", "2024-03-09"]],
];

const CPU_AT_EVERY_MINUTE: Row[] = [
  ["CPU", "2024-01-01T00:00:00Z", 10],
  ["CPU", "2024-01-01T00:01:00Z", 20],
  ["CPU", "2024-01-01T00:02:00Z", 30],
  ["CPU", "2024-01-01T00:03:00Z", 40],
];

const GPU_ONLY_ONCE_THE_RENDER_STARTED: Row[] = [
  ["GPU", "2024-01-01T00:01:00Z", 50],
  ["GPU", "2024-01-01T00:02:00Z", 50],
  ["GPU", "2024-01-01T00:03:00Z", 50],
];

const A_IN_EVERY_QUARTER: Row[] = [
  ["A", "Q1", 1],
  ["A", "Q2", 2],
  ["A", "Q3", 3],
];

const B_MISSING_THE_FIRST_QUARTER: Row[] = [
  ["B", "Q2", 20],
  ["B", "Q3", 30],
];

const A_QUARTERS_OUT_OF_ORDER: Row[] = [
  ["A", "Q3", 3],
  ["A", "Q1", 1],
  ["A", "Q2", 2],
];

const A_FROM_THE_SECOND_CATEGORY: Row[] = [
  ["A", "X2", 10],
  ["A", "X3", 30],
];

const B_UNTIL_THE_SECOND_CATEGORY: Row[] = [
  ["B", "X1", 25],
  ["B", "X2", 20],
];

async function renderChart(
  page: Page,
  chart: Record<string, unknown>,
  rows: Row[],
) {
  return page.evaluate(
    ({ chart, rows }) => {
      document.getElementById("test-chart")?.remove();
      const container = document.createElement("div");
      container.id = "test-chart";
      container.setAttribute("data-pre-init", "chart");
      const payload = JSON.stringify({
        colors: [],
        marker: 4,
        ...chart,
        points: rows,
      });
      container.innerHTML = `<data hidden>${payload}</data><div class="chart" style="height:250px"></div>`;
      document.body.appendChild(container);

      const failures: string[] = [];
      const reportError = console.error;
      console.error = (...args) => failures.push(args.map(String).join(" "));
      const before = window.charts?.length ?? 0;
      container.dispatchEvent(
        new CustomEvent("fragment-loaded", { bubbles: true }),
      );
      console.error = reportError;

      const rendered = window.charts?.[before];
      const series = (rendered?.w.config.series ?? []).map((s) => ({
        name: s.name,
        points: s.data.map((p) => [
          p.x instanceof Date ? p.x.toISOString() : p.x,
          p.y,
        ]),
      }));
      const drawnPerSeries = series.map(({ name }) => {
        const markers = [
          ...container.querySelectorAll<SVGGraphicsElement>(
            `.apexcharts-series[seriesName='${name}'] .apexcharts-series-markers > .apexcharts-marker`,
          ),
        ].map((m) => m.getBBox());
        return {
          name,
          lefts: markers.map((b) => Math.round(b.x)),
          heights: markers.map((b) => Math.round(b.y)),
        };
      });
      const shapes = [
        ...container.querySelectorAll<SVGGraphicsElement>(
          ".apexcharts-bar-area, .apexcharts-rangebar-area, .apexcharts-treemap-rect",
        ),
      ].map((shape) => {
        const { x, y, width, height } = shape.getBBox();
        return { x, y, width, height };
      });

      return {
        failures,
        type: rendered?.w.config.chart.type ?? null,
        stacked: rendered?.w.config.chart.stacked ?? null,
        series,
        drawnPerSeries,
        shapes,
      };
    },
    { chart, rows },
  );
}

test.beforeEach(async ({ page }) => {
  await page.goto(`${BASE}/documentation.sql?component=chart#component`);
  await page.waitForSelector(".apexcharts-canvas");
});

test("draws a column chart as a vertical bar chart", async ({ page }) => {
  const chart = await renderChart(page, { type: "column" }, A_DAY_OF_WORK);

  expect(chart.failures).toEqual([]);
  expect(chart.shapes).toHaveLength(3);
  expect(chart.type).toBe("bar");

  expect(new Set(chart.shapes.map((s) => s.x)).size).toBe(3);
  expect(new Set(chart.shapes.map((s) => s.height)).size).toBe(3);
});

test("gives a stacked series a zero at every x it did not measure", async ({
  page,
}) => {
  const chart = await renderChart(
    page,
    { type: "area", stacked: true, time: true },
    [...CPU_AT_EVERY_MINUTE, ...GPU_ONLY_ONCE_THE_RENDER_STARTED],
  );

  expect(chart.failures).toEqual([]);
  expect(chart.series.map((s) => s.name)).toEqual(["CPU", "GPU"]);
  expect(chart.series[1].points).toEqual([
    ["2024-01-01T00:00:00.000Z", 0],
    ["2024-01-01T00:01:00.000Z", 50],
    ["2024-01-01T00:02:00.000Z", 50],
    ["2024-01-01T00:03:00.000Z", 50],
  ]);
});

test("stacks a series above the one it shares an x with", async ({ page }) => {
  const chart = await renderChart(
    page,
    { type: "area", stacked: true, time: true },
    [...CPU_AT_EVERY_MINUTE, ...GPU_ONLY_ONCE_THE_RENDER_STARTED],
  );
  const [cpu, gpu] = chart.drawnPerSeries;

  expect(gpu.heights).toHaveLength(4);
  expect(gpu.heights[0]).toBe(cpu.heights[0]);
  expect(gpu.heights[1]).toBeLessThan(cpu.heights[1]);
});

test("keeps a lone series in the order the query returned it (#930)", async ({
  page,
}) => {
  const chart = await renderChart(
    page,
    { type: "bar" },
    A_QUARTERS_OUT_OF_ORDER,
  );

  expect(chart.failures).toEqual([]);
  expect(chart.series[0].points).toEqual([
    ["Q3", 3],
    ["Q1", 1],
    ["Q2", 2],
  ]);
});

test("orders by name the categories two bar series do not share (#951)", async ({
  page,
}) => {
  const chart = await renderChart(page, { type: "bar" }, [
    ...A_FROM_THE_SECOND_CATEGORY,
    ...B_UNTIL_THE_SECOND_CATEGORY,
  ]);

  expect(chart.failures).toEqual([]);
  expect(chart.series[0].points).toEqual([
    ["X1", 0],
    ["X2", 10],
    ["X3", 30],
  ]);
  expect(chart.series[1].points).toEqual([
    ["X1", 25],
    ["X2", 20],
    ["X3", 0],
  ]);
});

test("leaves the points of a chart that does not stack alone", async ({
  page,
}) => {
  const chart = await renderChart(page, { type: "area", time: true }, [
    ...CPU_AT_EVERY_MINUTE,
    ...GPU_ONLY_ONCE_THE_RENDER_STARTED,
  ]);

  expect(chart.failures).toEqual([]);
  expect(chart.series[1].points).toEqual([
    ["2024-01-01T00:01:00.000Z", 50],
    ["2024-01-01T00:02:00.000Z", 50],
    ["2024-01-01T00:03:00.000Z", 50],
  ]);
});

test("stacks a bar series on the categories it skipped", async ({ page }) => {
  const chart = await renderChart(page, { type: "bar", stacked: true }, [
    ...A_IN_EVERY_QUARTER,
    ...B_MISSING_THE_FIRST_QUARTER,
  ]);

  expect(chart.failures).toEqual([]);
  expect(chart.series[1].points).toEqual([
    ["Q1", 0],
    ["Q2", 20],
    ["Q3", 30],
  ]);
});

test("lines an unstacked series up with the categories it skipped", async ({
  page,
}) => {
  const chart = await renderChart(page, { type: "line" }, [
    ...A_IN_EVERY_QUARTER,
    ...B_MISSING_THE_FIRST_QUARTER,
  ]);

  expect(chart.failures).toEqual([]);
  expect(chart.series[1].points).toEqual([
    ["Q1", null],
    ["Q2", 20],
    ["Q3", 30],
  ]);
});

test("draws nothing where an unstacked series has no value", async ({
  page,
}) => {
  const chart = await renderChart(page, { type: "line" }, [
    ...A_IN_EVERY_QUARTER,
    ...B_MISSING_THE_FIRST_QUARTER,
  ]);
  const [a, b] = chart.drawnPerSeries;

  expect(a.lefts).toHaveLength(3);
  expect(b.lefts).toEqual(a.lefts.slice(1));
});

test("keeps a measured zero apart from a missing value", async ({ page }) => {
  const chart = await renderChart(page, { type: "line" }, [
    ...A_IN_EVERY_QUARTER,
    ["B", "Q2", 0],
    ["B", "Q3", 30],
  ]);
  const [a, b] = chart.drawnPerSeries;

  expect(chart.series[1].points).toEqual([
    ["Q1", null],
    ["Q2", 0],
    ["Q3", 30],
  ]);
  expect(b.lefts).toEqual(a.lefts.slice(1));
});

for (const type of ["area", "scatter", "heatmap"]) {
  test(`lines up the series of a ${type} chart on a category axis`, async ({
    page,
  }) => {
    const chart = await renderChart(page, { type }, [
      ...A_IN_EVERY_QUARTER,
      ...B_MISSING_THE_FIRST_QUARTER,
    ]);

    expect(chart.failures).toEqual([]);
    expect(chart.series[1].points.map((p) => p[0])).toEqual(["Q1", "Q2", "Q3"]);
  });
}

test("keeps the bubble size of the points it lined up", async ({ page }) => {
  const chart = await renderChart(page, { type: "bubble" }, [
    ["A", "Q1", 1, 30],
    ["A", "Q2", 2, 30],
    ["B", "Q2", 5, 70],
  ]);

  expect(chart.failures).toEqual([]);
  expect(chart.series[1].points).toEqual([
    ["Q1", null],
    ["Q2", 5],
  ]);
});

test("leaves a rangeBar chart on a category axis alone", async ({ page }) => {
  const chart = await renderChart(
    page,
    { type: "rangeBar", time: true },
    TASKS_OVER_TIME,
  );

  expect(chart.failures).toEqual([]);
  expect(chart.shapes).toHaveLength(2);
});

test("leaves a treemap chart alone", async ({ page }) => {
  const chart = await renderChart(page, { type: "treemap" }, [
    ["North America", "United States", 35],
    ["North America", "Canada", 15],
    ["Europe", "France", 30],
    ["Europe", "Germany", 55],
  ]);

  expect(chart.failures).toEqual([]);
  expect(chart.shapes).toHaveLength(4);
});

test("draws a rangeBar chart that asks to be stacked", async ({ page }) => {
  const chart = await renderChart(
    page,
    { type: "rangeBar", stacked: true, time: true },
    TASKS_OVER_TIME,
  );

  expect(chart.failures).toEqual([]);
  expect(chart.shapes).toHaveLength(2);
  expect(chart.stacked).toBe(false);
});

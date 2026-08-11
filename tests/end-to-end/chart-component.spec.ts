import { expect, type Page, test } from "@playwright/test";

const BASE = process.env.SQLPAGE_TEST_BASE ?? "http://localhost:8080/";

declare global {
  interface Window {
    charts?: {
      w: { config: { chart: { type: string; stacked: boolean } } };
    }[];
  }
  function sqlpage_chart(): void;
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
      sqlpage_chart();
      console.error = reportError;

      const rendered = window.charts?.[before];
      const shapes = [
        ...container.querySelectorAll<SVGGraphicsElement>(
          ".apexcharts-bar-area, .apexcharts-rangebar-area",
        ),
      ].map((shape) => {
        const { x, y, width, height } = shape.getBBox();
        return { x, y, width, height };
      });

      return {
        failures,
        type: rendered?.w.config.chart.type ?? null,
        stacked: rendered?.w.config.chart.stacked ?? null,
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

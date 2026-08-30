import { expect, type Page, test } from "../../fixture";

type ChartPoint = { x: string | number | Date; y: number | null };

declare global {
  interface Window {
    charts?: {
      w: {
        config: {
          chart: { type: string; stacked: boolean };
          series: { name: string; data?: ChartPoint[] }[];
        };
      };
    }[];
  }
}

const MARKS =
  ".apexcharts-bar-area, .apexcharts-rangebar-area, .apexcharts-treemap-rect, .apexcharts-pie-area, .apexcharts-heatmap-rect, .apexcharts-series .apexcharts-marker";

const RED = "#f03e3e";
const ORANGE = "#f76707";
const GREEN = "#37b24d";
async function renderChart(page: Page, fixture: string) {
  const failures: string[] = [];
  const recordError = (message: { type(): string; text(): string }) => {
    if (message.type() === "error") failures.push(message.text());
  };
  page.on("console", recordError);
  const response = await page.goto(`/chart/${fixture}.sql`);
  expect(response?.ok(), `loading ${response?.url()}`).toBe(true);
  await expect(page.locator("#test-chart .apexcharts-canvas")).toBeVisible();
  page.off("console", recordError);

  return page.evaluate(
    ({ failures, marks }) => {
      const container = document.getElementById("test-chart");
      if (!container) throw new Error("Chart fixture did not render");
      const rendered = window.charts?.[0];
      const series = (rendered?.w.config.series ?? []).map((s) => ({
        name: s.name,
        points: (s.data ?? []).map((p) => [
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
        ...container.querySelectorAll<SVGGraphicsElement>(marks),
      ].map((shape) => {
        const { x, y, width, height } = shape.getBBox();
        return { x, y, width, height, fill: shape.getAttribute("fill") };
      });
      const annotated = [
        ...container.querySelectorAll(
          ".apexcharts-xaxis-annotations, .apexcharts-yaxis-annotations",
        ),
      ];
      const count = (selector: string) =>
        annotated.reduce((n, g) => n + g.querySelectorAll(selector).length, 0);
      const referenceLines = {
        lines: count("line"),
        labelBoxes: count("rect"),
        labelTexts: annotated.flatMap((g) =>
          [...g.querySelectorAll("text")].map((t) => t.textContent),
        ),
        strokes: annotated.flatMap((g) =>
          [...g.querySelectorAll("line")].map((l) => l.getAttribute("stroke")),
        ),
      };

      return {
        failures,
        type: rendered?.w.config.chart.type ?? null,
        stacked: rendered?.w.config.chart.stacked ?? null,
        series,
        drawnPerSeries,
        shapes,
        referenceLines,
      };
    },
    { failures, marks: MARKS },
  );
}

const fills = (chart: Awaited<ReturnType<typeof renderChart>>) =>
  chart.shapes.map(({ fill }) => {
    const channels = fill?.match(/^rgba\((\d+),(\d+),(\d+),[\d.]+\)$/);
    if (!channels) return fill;
    const hex = channels
      .slice(1)
      .map((c) => Number(c).toString(16).padStart(2, "0"));
    return `#${hex.join("")}`;
  });

test("draws a column chart as a vertical bar chart", async ({ page }) => {
  const chart = await renderChart(page, "column");

  expect(chart.failures).toEqual([]);
  expect(chart.shapes).toHaveLength(3);
  expect(chart.type).toBe("bar");

  expect(new Set(chart.shapes.map((s) => s.x)).size).toBe(3);
  expect(new Set(chart.shapes.map((s) => s.height)).size).toBe(3);
});

test("gives a stacked series a zero at every x it did not measure", async ({
  page,
}) => {
  const chart = await renderChart(page, "stacked-time-series");

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
  const chart = await renderChart(page, "stacked-time-series");
  const [cpu, gpu] = chart.drawnPerSeries;

  expect(gpu.heights).toHaveLength(4);
  expect(gpu.heights[0]).toBe(cpu.heights[0]);
  expect(gpu.heights[1]).toBeLessThan(cpu.heights[1]);
});

test("keeps a lone series in the order the query returned it (#930)", async ({
  page,
}) => {
  const chart = await renderChart(page, "out-of-order");

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
  const chart = await renderChart(page, "disjoint-categories");

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
  const chart = await renderChart(page, "unstacked-time-series");

  expect(chart.failures).toEqual([]);
  expect(chart.series[1].points).toEqual([
    ["2024-01-01T00:01:00.000Z", 50],
    ["2024-01-01T00:02:00.000Z", 50],
    ["2024-01-01T00:03:00.000Z", 50],
  ]);
});

test("stacks a bar series on the categories it skipped", async ({ page }) => {
  const chart = await renderChart(page, "stacked-categories");

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
  const chart = await renderChart(page, "line-categories");

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
  const chart = await renderChart(page, "line-categories");
  const [a, b] = chart.drawnPerSeries;

  expect(a.lefts).toHaveLength(3);
  expect(b.lefts).toEqual(a.lefts.slice(1));
});

test("keeps a measured zero apart from a missing value", async ({ page }) => {
  const chart = await renderChart(page, "zero-and-missing");
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
    const chart = await renderChart(page, `${type}-categories`);

    expect(chart.failures).toEqual([]);
    expect(chart.series[1].points.map((p) => p[0])).toEqual(["Q1", "Q2", "Q3"]);
  });
}

test("keeps the bubble size of the points it lined up", async ({ page }) => {
  const chart = await renderChart(page, "bubble-categories");

  expect(chart.failures).toEqual([]);
  expect(chart.series[1].points).toEqual([
    ["Q1", null],
    ["Q2", 5],
  ]);
});

test("leaves a rangeBar chart on a category axis alone", async ({ page }) => {
  const chart = await renderChart(page, "range-bar");

  expect(chart.failures).toEqual([]);
  expect(chart.shapes).toHaveLength(2);
});

test("leaves a treemap chart alone", async ({ page }) => {
  const chart = await renderChart(page, "treemap");

  expect(chart.failures).toEqual([]);
  expect(chart.shapes).toHaveLength(4);
});

test("draws a rangeBar chart that asks to be stacked", async ({ page }) => {
  const chart = await renderChart(page, "stacked-range-bar");

  expect(chart.failures).toEqual([]);
  expect(chart.shapes).toHaveLength(2);
  expect(chart.stacked).toBe(false);
});

test("gives the tooltip title the color of the tooltip around it", async ({
  page,
}) => {
  await renderChart(page, "tooltip");
  await page.locator("#test-chart .apexcharts-inner").hover({ force: true });

  const title = page.locator("#test-chart .apexcharts-tooltip-title");
  await expect(title).toHaveText("Tue");
  const colors = await title.evaluate((el) => ({
    title: getComputedStyle(el).color,
    tooltip: getComputedStyle(el.parentElement as HTMLElement).color,
  }));

  expect(colors.title).toBe(colors.tooltip);
});

test("draws a reference line that carries no label", async ({ page }) => {
  const chart = await renderChart(page, "unlabeled-reference-lines");

  expect(chart.failures).toEqual([]);
  expect(chart.referenceLines.lines).toBe(2);
  expect(chart.referenceLines.labelBoxes).toBe(0);
  expect(chart.referenceLines.labelTexts).toEqual(["", ""]);
});

test("draws a box behind the label of a reference line that carries one", async ({
  page,
}) => {
  const chart = await renderChart(page, "labeled-reference-line");

  expect(chart.failures).toEqual([]);
  expect(chart.referenceLines.lines).toBe(1);
  expect(chart.referenceLines.labelBoxes).toBe(1);
  expect(chart.referenceLines.labelTexts).toEqual(["limit"]);
});

for (const type of [
  "bar",
  "column",
  "rangeBar",
  "pie",
  "treemap",
  "line",
  "area",
  "scatter",
  "bubble",
]) {
  test(`colors every mark of a ${type} chart from its own row`, async ({
    page,
  }) => {
    const chart = await renderChart(page, `colored-${type}`);

    expect(chart.failures).toEqual([]);
    expect(fills(chart)).toEqual([RED, GREEN]);
  });
}

test("colors each bar of a horizontal bar chart from its own row (#1228)", async ({
  page,
}) => {
  const chart = await renderChart(page, "colored-horizontal-bar");

  expect(chart.failures).toEqual([]);
  expect(fills(chart)).toEqual([RED, ORANGE, GREEN]);
});

test("leaves a heatmap, which shades its cells from their own value, alone", async ({
  page,
}) => {
  const shaded = await renderChart(page, "uncolored-heatmap");
  const colored = await renderChart(page, "colored-heatmap");

  expect(colored.failures).toEqual([]);
  expect(fills(colored)).toEqual(fills(shaded));
});

test("leaves a row without a color on the color of its series", async ({
  page,
}) => {
  const plain = await renderChart(page, "uncolored-bar");
  const mixed = await renderChart(page, "mixed-colors");

  expect(mixed.failures).toEqual([]);
  expect(fills(mixed)).toEqual([fills(plain)[0], RED]);
});

test("lets a row color override the color given to the whole chart", async ({
  page,
}) => {
  const chart = await renderChart(page, "chart-and-row-colors");

  expect(chart.failures).toEqual([]);
  expect(fills(chart)).toEqual(["#339af0", RED]);
});

test("keeps the color of the series when a row names a color SQLPage does not know", async ({
  page,
}) => {
  const plain = await renderChart(page, "uncolored-bar");
  const unknown = await renderChart(page, "unknown-colors");

  expect(unknown.failures).toEqual([]);
  expect(fills(unknown)).toEqual(fills(plain));
});

test("keeps coloring reference lines from their own row", async ({ page }) => {
  const chart = await renderChart(page, "colored-reference-line");

  expect(chart.failures).toEqual([]);
  expect(chart.referenceLines.strokes).toEqual([GREEN]);
});

import type { ApexOptions } from "apexcharts";
import ApexCharts from "apexcharts";
import {
  align_series_for,
  type ChartPoint,
  type ChartSeries,
  type Series,
  xaxis_type_for,
} from "./chart_series.ts";
import { add_init_fn } from "./init.ts";

type DataPoint = {
  name: string;
  x: string | number | null;
  y: string | number | number[] | null;
  color?: string | null;
  z?: string | number | null;
  link?: string;
};

type AxisTitles = Record<"x" | "y" | "z", string | undefined>;

type TooltipArgs = {
  seriesIndex: number;
  dataPointIndex: number;
  // biome-ignore lint/suspicious/noExplicitAny: ApexCharts leaves its tooltip context untyped
  w: any;
};

function linkTooltipValue(
  value: string | number | null,
  link: string | undefined,
) {
  const text = value == null ? "" : String(value);
  if (!link || !value) return text;
  const anchor = document.createElement("a");
  anchor.setAttribute("href", link);
  anchor.textContent = text;
  return anchor.outerHTML;
}

const rangeBarLabel = (_value: string | number, args?: TooltipArgs) =>
  args ? args.w.config.series[args.seriesIndex].name : "";

const pieLabel = (value: string | number, args?: TooltipArgs) =>
  args
    ? `${args.w.config.labels[args.seriesIndex]}: ${Number(value).toFixed()}%`
    : "";

const numberLabel = (value: string | number) =>
  value == null ? "" : value.toLocaleString?.() || String(value);

const sqlpage_chart = (() => {
  function sqlpage_chart() {
    const charts = document.querySelectorAll<HTMLElement>(
      "[data-pre-init=chart]",
    );
    for (const c of charts) {
      try {
        build_sqlpage_chart(c);
      } catch (e) {
        console.error(e);
      }
    }
  }

  const tblrColors = [
    ["blue", "#1c7ed6", "#339af0"],
    ["red", "#f03e3e", "#ff6b6b"],
    ["green", "#37b24d", "#51cf66"],
    ["pink", "#d6336c", "#f06595"],
    ["purple", "#ae3ec9", "#cc5de8"],
    ["orange", "#f76707", "#ff922b"],
    ["cyan", "#1098ad", "#22b8cf"],
    ["teal", "#0ca678", "#20c997"],
    ["yellow", "#f59f00", "#fcc419"],
    ["indigo", "#4263eb", "#5c7cfa"],
    ["lime", "#74b816", "#94d82d"],
    ["azure", "#339af0", "#339af0"],
    ["gray", "#495057", "#adb5bd"],
    ["black", "#000000", "#000000"],
    ["white", "#ffffff", "#f8f9fa"],
  ];
  const colorNames = new Map(
    tblrColors.flatMap(([name, dark, light]): [string, string][] => [
      [name, dark],
      [`${name}-lt`, light],
    ]),
  );
  const isDarkTheme = document.body?.dataset?.bsTheme === "dark";

  const STACKABLE_CHART_TYPES = ["line", "area", "bar"];
  const STROKE_WIDTHS = new Map([
    ["area", 3],
    ["line", 2],
  ]);
  const APEXCHARTS_TYPE_ALIASES = new Map([["column", "bar"]]);

  const referenceColor = colorNames.get(isDarkTheme ? "gray-lt" : "gray");

  type ReferenceLine = Record<
    "xline" | "xline_end" | "yline" | "yline_end" | "label" | "color",
    string | number | null
  >;

  const named_color = (name: unknown): string | undefined =>
    typeof name === "string" ? colorNames.get(name) : undefined;

  const reference_color = (name: string | number | null) =>
    named_color(name) || referenceColor;

  function reference_lines(
    rows: ReferenceLine[],
    column: "x" | "y",
    axis: "x" | "y",
    to_axis_value: (value: string | number) => unknown,
  ): object[] {
    const on_axis = (value: string | number | null) => {
      if (value == null) return null;
      const placed = to_axis_value(value);
      return Number.isNaN(placed) ? null : placed;
    };
    return rows.flatMap((row) => {
      const from = on_axis(row[`${column}line`]);
      if (from == null) return [];
      const color = reference_color(row.color);
      return [
        {
          [axis]: from,
          [`${axis}2`]: on_axis(row[`${column}line_end`]),
          borderColor: color,
          fillColor: color,
          strokeDashArray: 4,
          label: {
            text: row.label,
            orientation: column === "y" ? "horizontal" : "vertical",
            borderColor: color,
            style: { background: color, color: isDarkTheme ? "#000" : "#fff" },
          },
        },
      ];
    });
  }

  function build_sqlpage_chart(c: HTMLElement) {
    const [data_element] = c.getElementsByTagName("data");
    const data = JSON.parse(data_element.textContent);
    const chartContainer = c.querySelector(".chart") as HTMLElement;
    chartContainer.innerHTML = "";
    const is_timeseries = !!data.time;
    const chart_type =
      APEXCHARTS_TYPE_ALIASES.get(data.type) || data.type || "line";
    const is_stacked =
      !!data.stacked && STACKABLE_CHART_TYPES.includes(chart_type);
    const points: DataPoint[] = data.points
      .filter(Array.isArray)
      .map(([name, x, y, color, z, link]) => ({
        name,
        x,
        y,
        color,
        z,
        link: link ?? undefined,
      }));
    const reference_rows: ReferenceLine[] = data.points.filter(
      (row: unknown) => !Array.isArray(row),
    );
    const series_map: Series = new Map();
    for (const { name, x: old_x, y: old_y, color, z, link } of points) {
      const point_series: ChartSeries = series_map.get(name) ?? {
        name,
        data: [],
      };
      series_map.set(name, point_series);
      let x: string | number | Date | null = old_x;
      let y = old_y;
      if (is_timeseries) {
        if (typeof x === "number") x = new Date(x * 1000);
        else if (chart_type === "rangeBar" && Array.isArray(y))
          y = y.map((y) => new Date(y).getTime());
        else x = new Date((x ?? 0) as string | number);
      }
      point_series.data.push({
        x,
        y,
        z,
        link,
        fillColor: named_color(color),
      } as ChartPoint);
    }
    if (data.xmin == null) data.xmin = undefined;
    if (data.xmax == null) data.xmax = undefined;
    if (data.ymin == null) data.ymin = undefined;
    if (data.ymax == null) data.ymax = undefined;

    const palette = [
      ...data.colors.map(named_color).filter((c) => c !== undefined),
      ...tblrColors.map(([_, dark, light]) => (isDarkTheme ? dark : light)),
      ...tblrColors.map(([_, dark, light]) => (isDarkTheme ? light : dark)),
    ];
    let colors = palette;

    const chart_series = [...series_map.values()];
    const xaxis_type = xaxis_type_for(
      chart_series,
      chart_type,
      is_timeseries,
      !!data.horizontal,
    );

    const labels =
      chart_type === "pie"
        ? points.map(({ name, x }) => String(x || name))
        : undefined;
    const series =
      chart_type === "pie"
        ? points.map(({ y }) => Number.parseFloat(String(y)))
        : chart_series.length > 1
          ? align_series_for(chart_series, chart_type, is_stacked)
          : chart_series;
    if (chart_type === "pie")
      colors = points.map(
        ({ color }, i) => named_color(color) || palette[i % palette.length],
      );

    const to_timestamp = (v) =>
      (typeof v === "number" ? new Date(v * 1000) : new Date(v)).getTime();
    const dates_are_values = is_timeseries && chart_type === "rangeBar";
    const to_value = dates_are_values ? to_timestamp : Number;
    const to_category =
      is_timeseries && !dates_are_values ? to_timestamp : (v) => v;
    const inverted =
      chart_type === "rangeBar" || (chart_type === "bar" && !!data.horizontal);
    const value_axis = inverted ? "x" : "y";
    const category_axis = inverted ? "y" : "x";
    const axis_titles: AxisTitles = {
      x: data.xtitle || undefined,
      y: data.ytitle || undefined,
      z: data.ztitle || undefined,
    };
    const has_point_links = points.some((point) => point.link);
    const text_x_values = chart_series.every(({ data }) =>
      data.every(({ x }) => x == null || typeof x === "string"),
    );
    const options: ApexOptions = {
      annotations: {
        [`${value_axis}axis`]: reference_lines(
          reference_rows,
          "y",
          value_axis,
          to_value,
        ),
        [`${category_axis}axis`]: reference_lines(
          reference_rows,
          "x",
          category_axis,
          to_category,
        ),
      },
      chart: {
        type: chart_type,
        fontFamily: "inherit",
        background: "transparent",
        parentHeightOffset: 0,
        height: chartContainer.style.height,
        stacked: is_stacked,
        toolbar: {
          show: !!data.toolbar,
        },
        animations: {
          enabled: false,
        },
        zoom: {
          enabled: false,
        },
        events: {
          dataPointSelection: (_event, _chart, args) => {
            const link = args && pointLink(args, points);
            if (link) window.location.assign(link);
          },
        },
      },
      theme: {
        mode: isDarkTheme ? "dark" : "light",
        palette: "palette4",
      },
      legend: {
        show: data.show_legend === null || !!data.show_legend,
      },
      dataLabels: {
        enabled: !!data.labels,
        dropShadow: {
          enabled: true,
          color: "var(--tblr-primary-bg-subtle)",
        },
        formatter:
          chart_type === "rangeBar"
            ? rangeBarLabel
            : chart_type === "pie"
              ? pieLabel
              : numberLabel,
      },
      fill: {
        type: chart_type === "area" ? "gradient" : "solid",
      },
      stroke: {
        width: STROKE_WIDTHS.get(chart_type) ?? 0,
        lineCap: "round",
        curve: "smooth",
      },
      xaxis: {
        tooltip: {
          enabled: false,
        },
        min: data.xmin,
        max: data.xmax,
        title: {
          text: axis_titles.x,
        },
        type: xaxis_type,
        labels: {
          datetimeUTC: false,
        },
        // Numeric axes count intervals; category and time axes use tickAmount
        // as a target for label density.
        tickAmount: data.xticks || undefined,
      },
      yaxis: {
        logarithmic: !!data.logarithmic,
        min: data.ymin,
        max: data.ymax,
        stepSize: data.ystep,
        tickAmount: data.yticks,
        title: {
          text: axis_titles.y,
        },
      },
      markers: {
        size: data.marker || 0,
        strokeWidth: 0,
        hover: {
          sizeOffset: 5,
        },
      },
      tooltip: {
        fillSeriesColor: false,
        interactive: has_point_links,
        custom:
          chart_type === "bubble" || chart_type === "scatter"
            ? (args: TooltipArgs) => chartTooltip(args, axis_titles)
            : undefined,
        x: {
          formatter:
            has_point_links && text_x_values
              ? (value, args) =>
                  linkTooltipValue(value, args?.w && pointLink(args, points))
              : undefined,
        },
        y: {
          formatter: (value, args) => {
            if (value == null) return "";
            let formatted: string;
            if (is_timeseries && chart_type === "rangeBar") {
              const d = new Date(value);
              formatted =
                d.getHours() === 0 && d.getMinutes() === 0
                  ? d.toLocaleDateString()
                  : d.toLocaleString();
            } else {
              formatted = value.toLocaleString(undefined, {
                maximumFractionDigits: 2,
              });
            }
            const w = args?.w || args;
            return linkTooltipValue(
              formatted,
              w?.config &&
                args?.seriesIndex &&
                pointLink({ ...args, w }, points),
            );
          },
        },
      },
      plotOptions: {
        bar: {
          horizontal: !!data.horizontal || chart_type === "rangeBar",
          borderRadius: 5,
        },
        bubble: { minBubbleRadius: 5 },
      },
      colors,
      series,
    };
    if (labels) options.labels = labels;
    const chart = new ApexCharts(chartContainer, options);
    chart.render().catch(console.error);
    if (window.charts) window.charts.push(chart);
    else window.charts = [chart];
    c.removeAttribute("data-pre-init");
  }

  function chartTooltip(
    { seriesIndex, dataPointIndex, w }: TooltipArgs,
    titles: AxisTitles,
  ) {
    const series = w.config.series[seriesIndex];
    const name = series?.name || "";
    const point = series?.data[dataPointIndex];

    const tooltip = document.createElement("div");
    tooltip.className = "apexcharts-tooltip-text";
    tooltip.style.fontFamily = "inherit";

    const seriesName = document.createElement("div");
    seriesName.className = "apexcharts-tooltip-y-group";
    seriesName.style.fontWeight = "bold";
    seriesName.innerText = name;
    tooltip.appendChild(seriesName);

    for (const axis of ["x", "y", "z"] as const) {
      const value = point[axis];
      if (value == null) continue;
      const axisValue = document.createElement("div");
      axisValue.className = "apexcharts-tooltip-y-group";
      const title = titles[axis] || axis;
      const labelSpan = document.createElement("span");
      labelSpan.className = "apexcharts-tooltip-text-y-label";
      labelSpan.innerText = `${title}: `;
      axisValue.appendChild(labelSpan);
      const valueSpan = document.createElement("span");
      valueSpan.className = "apexcharts-tooltip-text-y-value";
      const formatter = axis === "y" && w.config.tooltip.y.formatter;
      const format = (v) =>
        formatter ? formatter(v, { seriesIndex, dataPointIndex, w }) : v;
      const formatted = Array.isArray(value)
        ? value.map(format).join(" - ")
        : format(value);
      if (axis === "y" && point.link) valueSpan.innerHTML = formatted;
      else valueSpan.innerText = formatted;
      axisValue.appendChild(valueSpan);
      tooltip.appendChild(axisValue);
    }
    return tooltip.outerHTML;
  }

  function pointLink(
    { seriesIndex, dataPointIndex, w }: TooltipArgs,
    points: DataPoint[],
  ) {
    const series = w.config.series[seriesIndex];
    return Array.isArray(series?.data)
      ? series.data[dataPointIndex]?.link
      : points[seriesIndex]?.link;
  }

  return sqlpage_chart;
})();

add_init_fn(sqlpage_chart);

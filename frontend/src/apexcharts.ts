import type { ApexChart, ApexOptions } from "apexcharts";
import ApexCharts from "apexcharts";
import { type ReferenceLine, read_chart_data } from "./chart_data.ts";
import {
  align_series_for,
  type ChartSeries,
  type PlotValue,
  type Series,
  type XValue,
  xaxis_type_for,
} from "./chart_series.ts";
import { add_init_fn, type InitRoot, select_all } from "./init.ts";

type AxisTitles = Record<"x" | "y" | "z", string | undefined>;

type PointIndex = { seriesIndex: number; dataPointIndex: number };

/**
 * ApexCharts leaves the options a chart rendered, on `chart.w`, undescribed.
 * These are the ones every chart built here sets.
 */
export type RenderedChart = ApexCharts & {
  w: {
    config: {
      chart: { type: string; stacked: boolean };
      xaxis: { type?: string; tickAmount?: number };
      series: ChartSeries[] | number[];
      tooltip: { custom?: (point: PointIndex) => string };
    };
    globals: { labels: (string | number)[]; isDataXYZ: boolean };
  };
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

const rangeBarLabel =
  (names: (string | number)[]) =>
  (_value: string | number, point?: PointIndex) =>
    point ? String(names[point.seriesIndex] ?? "") : "";

const pieLabel =
  (labels: string[]) => (value: string | number, point?: PointIndex) =>
    point ? `${labels[point.seriesIndex]}: ${Number(value).toFixed()}%` : "";

const numberLabel = (value: string | number) =>
  value == null ? "" : value.toLocaleString?.() || String(value);

const dateLabel = (value: number) => {
  const date = new Date(value);
  if (date.getHours() === 0 && date.getMinutes() === 0)
    return date.toLocaleDateString();
  return date.toLocaleString();
};

/**
 * Bubble and scatter points carry an x, a y and a z that each need naming, and
 * the tooltip ApexCharts draws names none of them.
 */
const axisTooltip =
  (
    series: ChartSeries[],
    titles: AxisTitles,
    formatValue: (value: number | null) => string,
  ) =>
  ({ seriesIndex, dataPointIndex }: PointIndex) => {
    const plotted = series[seriesIndex];
    const point = plotted?.data[dataPointIndex];

    const tooltip = document.createElement("div");
    tooltip.className = "apexcharts-tooltip-text";
    tooltip.style.fontFamily = "inherit";

    const seriesName = document.createElement("div");
    seriesName.className = "apexcharts-tooltip-y-group";
    seriesName.style.fontWeight = "bold";
    seriesName.innerText = String(plotted?.name ?? "");
    tooltip.appendChild(seriesName);

    for (const axis of ["x", "y", "z"] as const) {
      const measured = point?.[axis];
      if (measured == null) continue;
      const format = (value: XValue) =>
        axis === "y" && typeof value === "number"
          ? formatValue(value)
          : String(value ?? "");
      const axisValue = document.createElement("div");
      axisValue.className = "apexcharts-tooltip-y-group";
      const labelSpan = document.createElement("span");
      labelSpan.className = "apexcharts-tooltip-text-y-label";
      labelSpan.innerText = `${titles[axis] || axis}: `;
      axisValue.appendChild(labelSpan);
      const valueSpan = document.createElement("span");
      valueSpan.className = "apexcharts-tooltip-text-y-value";
      const formatted = Array.isArray(measured)
        ? measured.map(format).join(" - ")
        : format(measured);
      if (axis === "y" && point?.link)
        valueSpan.innerHTML = linkTooltipValue(formatted, point.link);
      else valueSpan.innerText = formatted;
      axisValue.appendChild(valueSpan);
      tooltip.appendChild(axisValue);
    }
    return tooltip.outerHTML;
  };

function sqlpage_chart(root: InitRoot) {
  const charts = select_all(root, "[data-pre-init=chart]", HTMLElement);
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
] as const;
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

const named_color = (name: unknown): string | undefined =>
  typeof name === "string" ? colorNames.get(name) : undefined;

const reference_color = (name: PlotValue) =>
  named_color(name) || referenceColor;

function reference_lines(
  rows: ReferenceLine[],
  column: "x" | "y",
  axis: "x" | "y",
  to_axis_value: (value: PlotValue) => unknown,
): object[] {
  const on_axis = (value: PlotValue) => {
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
  const chartContainer = c.querySelector<HTMLElement>(".chart");
  if (!data_element || !chartContainer)
    throw new Error("Chart component is missing its data or container");
  const data = read_chart_data(data_element.textContent);
  chartContainer.innerHTML = "";
  const is_timeseries = data.time;
  const chart_type =
    APEXCHARTS_TYPE_ALIASES.get(data.type) || data.type || "line";
  const is_pie = chart_type === "pie";
  const is_stacked = data.stacked && STACKABLE_CHART_TYPES.includes(chart_type);
  const { points } = data;
  const series_map: Series = new Map();
  for (const { name, x: old_x, y: old_y, color, z, link } of points) {
    const point_series: ChartSeries = series_map.get(name) ?? {
      name,
      data: [],
    };
    series_map.set(name, point_series);
    let x: XValue = old_x;
    let y: PlotValue | PlotValue[] = old_y;
    if (is_timeseries) {
      if (typeof x === "number") x = new Date(x * 1000);
      else if (chart_type === "rangeBar" && Array.isArray(y))
        y = y.map((value) => new Date(value ?? 0).getTime());
      else x = new Date(x ?? 0);
    }
    point_series.data.push({
      x,
      y,
      z,
      link,
      fillColor: named_color(color),
    });
  }

  const palette = [
    ...data.colors.map(named_color).filter((color) => color !== undefined),
    ...tblrColors.map(([_, dark, light]) => (isDarkTheme ? dark : light)),
    ...tblrColors.map(([_, dark, light]) => (isDarkTheme ? light : dark)),
  ];

  const chart_series = [...series_map.values()];
  const aligned_series =
    chart_series.length > 1
      ? align_series_for(chart_series, chart_type, is_stacked)
      : chart_series;
  const xaxis_type = xaxis_type_for(
    chart_series,
    chart_type,
    is_timeseries,
    data.horizontal,
  );

  const pie_labels = is_pie
    ? points.map(({ name, x }) => String(x || name))
    : [];
  const series = is_pie
    ? points.map(({ y }) => Number.parseFloat(String(y)))
    : aligned_series;
  const colors = is_pie
    ? points.map(
        ({ color }, i) =>
          named_color(color) ??
          palette[i % palette.length] ??
          tblrColors[0][isDarkTheme ? 1 : 2],
      )
    : palette;

  const to_timestamp = (value: PlotValue) =>
    (typeof value === "number"
      ? new Date(value * 1000)
      : new Date(value ?? 0)
    ).getTime();
  const dates_are_values = is_timeseries && chart_type === "rangeBar";
  const to_value = dates_are_values ? to_timestamp : Number;
  const to_category =
    is_timeseries && !dates_are_values
      ? to_timestamp
      : (value: PlotValue) => value;
  const inverted =
    chart_type === "rangeBar" || (chart_type === "bar" && data.horizontal);
  const value_axis = inverted ? "x" : "y";
  const category_axis = inverted ? "y" : "x";
  const axis_titles: AxisTitles = {
    x: data.xtitle,
    y: data.ytitle,
    z: data.ztitle,
  };
  const has_point_links = points.some((point) => point.link);
  const text_x_values = chart_series.every(({ data }) =>
    data.every(({ x }) => x == null || typeof x === "string"),
  );
  const formatValue = (value: number | null) => {
    if (value == null) return "";
    if (dates_are_values) return dateLabel(value);
    return value.toLocaleString(undefined, { maximumFractionDigits: 2 });
  };
  const pointLink = is_pie
    ? ({ seriesIndex }: PointIndex) => points[seriesIndex]?.link
    : ({ seriesIndex, dataPointIndex }: PointIndex) =>
        aligned_series[seriesIndex]?.data[dataPointIndex]?.link;
  const options: ApexOptions = {
    annotations: {
      [`${value_axis}axis`]: reference_lines(
        data.reference_lines,
        "y",
        value_axis,
        to_value,
      ),
      [`${category_axis}axis`]: reference_lines(
        data.reference_lines,
        "x",
        category_axis,
        to_category,
      ),
    },
    chart: {
      // The query may name any type ApexCharts draws, not only the ones
      // the component documents.
      type: chart_type as ApexChart["type"],
      fontFamily: "inherit",
      background: "transparent",
      parentHeightOffset: 0,
      height: chartContainer.style.height,
      stacked: is_stacked,
      toolbar: {
        show: data.toolbar,
      },
      animations: {
        enabled: false,
      },
      zoom: {
        enabled: false,
      },
      events: {
        dataPointSelection: (_event, _chart, args) => {
          const link = args && pointLink(args);
          if (link) window.location.assign(link);
        },
      },
    },
    theme: {
      mode: isDarkTheme ? "dark" : "light",
      palette: "palette4",
    },
    legend: {
      show: data.show_legend,
    },
    dataLabels: {
      enabled: data.labels,
      dropShadow: {
        enabled: true,
        color: "var(--tblr-primary-bg-subtle)",
      },
      formatter: is_pie
        ? pieLabel(pie_labels)
        : chart_type === "rangeBar"
          ? rangeBarLabel(aligned_series.map(({ name }) => name))
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
      logarithmic: data.logarithmic,
      min: data.ymin,
      max: data.ymax,
      stepSize: data.ystep,
      tickAmount: data.yticks,
      title: {
        text: axis_titles.y,
      },
    },
    markers: {
      size: data.marker ?? 0,
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
          ? axisTooltip(aligned_series, axis_titles, formatValue)
          : undefined,
      x: {
        formatter:
          has_point_links && text_x_values
            ? (value, args) => linkTooltipValue(value, args && pointLink(args))
            : undefined,
      },
      y: {
        formatter: (value, args) =>
          linkTooltipValue(
            formatValue(value),
            args?.seriesIndex !== undefined ? pointLink(args) : undefined,
          ),
      },
    },
    plotOptions: {
      bar: {
        horizontal: data.horizontal || chart_type === "rangeBar",
        borderRadius: 5,
      },
      bubble: { minBubbleRadius: 5 },
    },
    colors,
    // ApexCharts draws a numeric series name but declares only a string.
    series: series as ApexOptions["series"],
  };
  if (is_pie) options.labels = pie_labels;
  const chart = new ApexCharts(chartContainer, options) as RenderedChart;
  chart.render().catch(console.error);
  if (window.charts) window.charts.push(chart);
  else window.charts = [chart];
  c.removeAttribute("data-pre-init");
}

add_init_fn(sqlpage_chart);

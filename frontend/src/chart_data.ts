import type { PlotValue } from "./chart_series.ts";

export type DataPoint = {
  name: string | number;
  x: PlotValue;
  y: PlotValue | PlotValue[];
  color: PlotValue;
  z: PlotValue | undefined;
  link: string | undefined;
};

/** A row that draws a line across the chart instead of plotting a point. */
export type ReferenceLine = Record<
  "xline" | "xline_end" | "yline" | "yline_end" | "label" | "color",
  PlotValue
>;

/** One chart's properties, as `chart.handlebars` encodes them. */
export type ChartData = {
  type: string;
  time: boolean;
  labels: boolean;
  marker: number | undefined;
  xtitle: string | undefined;
  ytitle: string | undefined;
  ztitle: string | undefined;
  xticks: number | undefined;
  yticks: number | undefined;
  ystep: number | undefined;
  xmin: number | undefined;
  ymin: number | undefined;
  xmax: number | undefined;
  ymax: number | undefined;
  toolbar: boolean;
  show_legend: boolean;
  logarithmic: boolean;
  horizontal: boolean;
  stacked: boolean;
  colors: PlotValue[];
  points: DataPoint[];
  reference_lines: ReferenceLine[];
};

const plot_value = (value: unknown): PlotValue => {
  if (value == null) return null;
  if (typeof value === "number" || typeof value === "string") return value;
  if (typeof value === "boolean") return Number(value);
  return String(value);
};

const text_value = (value: unknown) =>
  String(plot_value(value) ?? "") || undefined;

const number_value = (value: unknown) => {
  const text = plot_value(value);
  const number = text == null || text === "" ? Number.NaN : Number(text);
  return Number.isFinite(number) ? number : undefined;
};

const data_point = ([name, x, y, color, z, link]: unknown[]): DataPoint => ({
  name: plot_value(name) ?? "",
  x: plot_value(x),
  y: Array.isArray(y) ? y.map(plot_value) : plot_value(y),
  color: plot_value(color),
  // ApexCharts reads a point as three-dimensional as soon as it carries a z,
  // so a row that never mentioned one must not carry a null.
  z: z === undefined ? undefined : plot_value(z),
  link: text_value(link),
});

const reference_line = (row: Record<string, unknown>): ReferenceLine => ({
  xline: plot_value(row.xline),
  xline_end: plot_value(row.xline_end),
  yline: plot_value(row.yline),
  yline_end: plot_value(row.yline_end),
  label: plot_value(row.label),
  color: plot_value(row.color),
});

export function read_chart_data(json: string | null): ChartData {
  const data = JSON.parse(json ?? "");
  const rows = Array.isArray(data.points) ? data.points : [];
  return {
    type: text_value(data.type) ?? "",
    time: !!data.time,
    labels: !!data.labels,
    marker: number_value(data.marker),
    xtitle: text_value(data.xtitle),
    ytitle: text_value(data.ytitle),
    ztitle: text_value(data.ztitle),
    xticks: number_value(data.xticks),
    yticks: number_value(data.yticks),
    ystep: number_value(data.ystep),
    xmin: number_value(data.xmin),
    ymin: number_value(data.ymin),
    xmax: number_value(data.xmax),
    ymax: number_value(data.ymax),
    toolbar: !!data.toolbar,
    show_legend: data.show_legend == null || !!data.show_legend,
    logarithmic: !!data.logarithmic,
    horizontal: !!data.horizontal,
    stacked: !!data.stacked,
    colors: Array.isArray(data.colors) ? data.colors.map(plot_value) : [],
    points: rows.filter(Array.isArray).map(data_point),
    reference_lines: rows
      .filter((row: unknown) => !Array.isArray(row))
      .map(reference_line),
  };
}

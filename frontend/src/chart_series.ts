export type XValue = number | string | Date;
export type ChartPoint = {
  x: XValue;
  y: number | string | number[] | null;
  z?: number;
  fillColor?: string;
  link?: string;
};
export type ChartSeries = { name: string; data: ChartPoint[] };
export type Series = Map<string, ChartSeries>;

const NUMERIC_X_CHART_TYPES = ["line", "area", "bar", "scatter", "bubble"];

const Y_WHEN_A_SERIES_SKIPS_A_LABEL = new Map<string, number | null>([
  ["bar", 0],
  ["line", null],
  ["area", null],
  ["scatter", null],
  ["bubble", null],
  ["heatmap", null],
]);

/** equal x values share a key */
const x_key = (x: XValue): number | string =>
  x instanceof Date ? x.getTime() : x;

const x_is_text = (series: ChartSeries[]) =>
  typeof series[0]?.data?.[0]?.x === "string";

export function xaxis_type_for(
  series: ChartSeries[],
  chart_type: string,
  is_timeseries: boolean,
  is_horizontal: boolean,
) {
  if (is_timeseries) return "datetime";
  if (x_is_text(series)) return "category";
  if (
    typeof series[0]?.data?.[0]?.x === "number" &&
    !is_horizontal &&
    NUMERIC_X_CHART_TYPES.includes(chart_type)
  )
    return "numeric";
  return undefined;
}

/**
 * @returns every x the series hold, in their own order where they agree and in
 * ascending order where they diverge
 */
export function merged_x_values(series: ChartSeries[]): XValue[] {
  const unread = series.map(({ data }) => data.map(({ x }) => x));
  const merged = new Map();
  while (unread.some((xs) => xs.length > 0)) {
    const with_lowest_x = unread
      .filter((xs) => xs.length > 0)
      .reduce((a, b) => (b[0] < a[0] ? b : a));
    const x = with_lowest_x.shift() as XValue;
    merged.set(x_key(x), x);
  }
  return [...merged.values()];
}

/**
 * ApexCharts pairs points across series by index rather than by x, so a
 * series that skips an x lands on the wrong one. Give every series the same
 * amount of x values.
 *
 * @param y_when_missing what a series with no value at an x is worth there:
 *   zero to add nothing to a stack, null to leave a gap.
 */
export function align_series(
  series: ChartSeries[],
  y_when_missing: number | null,
): ChartSeries[] {
  const all_x = merged_x_values(series);
  return series.map(({ name, data }) => {
    const by_x = new Map(data.map((point) => [x_key(point.x), point]));
    return {
      name,
      data: all_x.map((x) => {
        const point = by_x.get(x_key(x));
        return { ...point, x, y: point?.y ?? y_when_missing };
      }),
    };
  });
}

export function align_series_for(
  series: ChartSeries[],
  chart_type: string,
  is_stacked: boolean,
): ChartSeries[] {
  if (is_stacked) return align_series(series, 0);
  const y_when_missing = Y_WHEN_A_SERIES_SKIPS_A_LABEL.get(chart_type);
  if (x_is_text(series) && y_when_missing !== undefined)
    return align_series(series, y_when_missing);
  return series;
}

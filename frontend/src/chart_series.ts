export type PlotValue = string | number | null;
export type XValue = PlotValue | Date;
export type ChartPoint = {
  x: XValue;
  y: PlotValue | PlotValue[];
  z?: PlotValue;
  fillColor?: string;
  link?: string;
};
export type ChartSeries = { name: string | number; data: ChartPoint[] };
export type Series = Map<ChartSeries["name"], ChartSeries>;

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
const x_key = (x: XValue): PlotValue => (x instanceof Date ? x.getTime() : x);

/** A missing x sorts as zero, which is how JavaScript compares it. */
const is_lower = (x: XValue, than: XValue) => (x ?? 0) < (than ?? 0);

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
  const unread = series.map(({ data }) => {
    const iterator = data.values();
    return { iterator, next: iterator.next() };
  });
  const merged = new Map<PlotValue, XValue>();
  while (true) {
    let lowest: { stream: (typeof unread)[number]; x: XValue } | undefined;
    for (const stream of unread) {
      if (stream.next.done) continue;
      const x = stream.next.value.x;
      if (!lowest || is_lower(x, lowest.x)) lowest = { stream, x };
    }
    if (!lowest) break;
    merged.set(x_key(lowest.x), lowest.x);
    lowest.stream.next = lowest.stream.iterator.next();
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

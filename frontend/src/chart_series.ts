export type XValue = number | string | Date;
export type YValue = number | string | null | (number | string)[];
export type ChartPoint = { x: XValue; y: YValue; z?: number };
export type ChartSeries = { name: string; data: ChartPoint[] };

/** Equal x values share a key. */
const x_key = (x: XValue) => (x instanceof Date ? x.getTime() : x);

export const x_is_text = (series: ChartSeries[]) =>
  typeof series[0]?.data[0]?.x === "string";

const STACKED_SERIES_SKIPPING_AN_X_ADDS_NOTHING = 0;

const Y_WHEN_A_SERIES_SKIPS_A_LABEL: Record<string, 0 | null> = {
  bar: 0,
  line: null,
  area: null,
  scatter: null,
  bubble: null,
  heatmap: null,
};

/**
 * Every x the series hold, in their own order where they agree and in
 * ascending order where they diverge.
 */
export function merged_x_values(series: ChartSeries[]): XValue[] {
  const unread = series.map(({ data }) => data.map(({ x }) => x));
  const merged = new Map<number | string, XValue>();
  while (unread.some((xs) => xs.length > 0)) {
    const with_lowest_x = unread
      .filter((xs) => xs.length > 0)
      .reduce((a, b) => (b[0] < a[0] ? b : a));
    const [x] = with_lowest_x.splice(0, 1);
    merged.set(x_key(x), x);
  }
  return [...merged.values()];
}

/**
 * ApexCharts pairs points across series by index rather than by x, so a series
 * that skips an x lands on the wrong one. Give every series the same amount of
 * x values, worth `y_when_missing` where a series has no value.
 */
export function align_series(
  series: ChartSeries[],
  y_when_missing: 0 | null,
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
  if (is_stacked)
    return align_series(series, STACKED_SERIES_SKIPPING_AN_X_ADDS_NOTHING);
  if (x_is_text(series) && chart_type in Y_WHEN_A_SERIES_SKIPS_A_LABEL)
    return align_series(series, Y_WHEN_A_SERIES_SKIPS_A_LABEL[chart_type]);
  return series;
}

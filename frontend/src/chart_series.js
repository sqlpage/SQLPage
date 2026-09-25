/** @typedef {number|string|Date} XValue */
/** @typedef { {x:XValue, y:number|string|number[]|null, z?:number, fillColor?:string, link?:string} } ChartPoint */
/** @typedef { {name:string, data:ChartPoint[]} } ChartSeries */
/** @typedef { Map<string, ChartSeries> } Series */

const NUMERIC_X_CHART_TYPES = ["line", "area", "bar", "scatter", "bubble"];

const Y_WHEN_A_SERIES_SKIPS_A_LABEL = {
  bar: 0,
  line: null,
  area: null,
  scatter: null,
  bubble: null,
  heatmap: null,
};

/** @param {XValue} x @returns {number|string} equal x values share a key */
const x_key = (x) => (x instanceof Date ? x.getTime() : x);

/** @param {ChartSeries[]} series */
const x_is_text = (series) => typeof series[0]?.data?.[0]?.x === "string";

/** @param {ChartSeries[]} series @param {string} chart_type */
export function xaxis_type_for(
  series,
  chart_type,
  is_timeseries,
  is_horizontal,
) {
  if (is_timeseries) return "datetime";
  if (x_is_text(series)) return "category";
  if (
    typeof series[0]?.data?.[0]?.x === "number" &&
    !is_horizontal &&
    NUMERIC_X_CHART_TYPES.includes(chart_type)
  )
    return "numeric";
}

/**
 * @param {ChartSeries[]} series
 * @returns {XValue[]} every x the series hold, in their own order where they
 * agree and in ascending order where they diverge
 */
export function merged_x_values(series) {
  const unread = series.map(({ data }) => data.map(({ x }) => x));
  const merged = new Map();
  while (unread.some((xs) => xs.length > 0)) {
    const with_lowest_x = unread
      .filter((xs) => xs.length > 0)
      .reduce((a, b) => (b[0] < a[0] ? b : a));
    const x = /** @type {XValue} */ (with_lowest_x.shift());
    merged.set(x_key(x), x);
  }
  return [...merged.values()];
}

/**
 * ApexCharts pairs points across series by index rather than by x, so a
 * series that skips an x lands on the wrong one. Give every series the same
 * amount of x values.
 *
 * @param {ChartSeries[]} series
 * @param {number|null} y_when_missing what a series with no value at an x is
 *   worth there: zero to add nothing to a stack, null to leave a gap.
 * @returns {ChartSeries[]}
 */
export function align_series(series, y_when_missing) {
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

/**
 * @param {ChartSeries[]} series
 * @param {string} chart_type
 * @param {boolean} is_stacked
 * @returns {ChartSeries[]}
 */
export function align_series_for(series, chart_type, is_stacked) {
  if (is_stacked) return align_series(series, 0);
  if (x_is_text(series) && chart_type in Y_WHEN_A_SERIES_SKIPS_A_LABEL)
    return align_series(series, Y_WHEN_A_SERIES_SKIPS_A_LABEL[chart_type]);
  return series;
}

import ApexCharts from "apexcharts";
import { align_series_for, xaxis_type_for } from "./chart_series.js";
import { add_init_fn } from "./init.js";

/**
 * @typedef {import("./chart_series.js").ChartSeries} ChartSeries
 * @typedef {import("./chart_series.js").ChartPoint} ChartPoint
 * @typedef {import("./chart_series.js").Series} Series
 * @typedef {object} DataPoint
 * @property {string} name
 * @property {string|number|null} x
 * @property {string|number|number[]|null} y
 * @property {string|null} [color]
 * @property {string|number|null} [z]
 * @property {string} [link]
 */

/** @param {string|number|null} value @param {string|undefined} link */
function formatTooltipX(value, link) {
  if (!link || !value) return value;
  const anchor = document.createElement("a");
  anchor.setAttribute("href", link);
  anchor.textContent = String(value);
  return anchor.outerHTML;
}

const sqlpage_chart = (() => {
  function sqlpage_chart() {
    /** @type {NodeListOf<HTMLElement>} */
    const charts = document.querySelectorAll("[data-pre-init=chart]");
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
  const colorNames = Object.fromEntries(
    tblrColors.flatMap(([name, dark, light]) => [
      [name, dark],
      [`${name}-lt`, light],
    ]),
  );
  const isDarkTheme = document.body?.dataset?.bsTheme === "dark";

  const STACKABLE_CHART_TYPES = ["line", "area", "bar"];
  const APEXCHARTS_TYPE_ALIASES = { column: "bar" };

  const referenceColor = colorNames[isDarkTheme ? "gray-lt" : "gray"];

  /** @typedef { {[property:string]: string|number|null} } ReferenceLine */

  /** @param {unknown} name @returns {string|undefined} */
  const named_color = (name) =>
    typeof name === "string" ? colorNames[name] : undefined;

  /** @param {string|number|null} name */
  const reference_color = (name) => named_color(name) || referenceColor;

  /**
   * @param {ReferenceLine[]} rows - the rows that carry an xline or a yline
   * @param {"x"|"y"} column - the column the reference is written in
   * @param {"x"|"y"} axis - the apexcharts axis that column is drawn on
   * @param {(value: any) => any} to_axis_value - puts a SQL value on the axis
   * @returns {object[]} apexcharts axis annotations
   */
  function reference_lines(rows, column, axis, to_axis_value) {
    const on_axis = (value) => {
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

  /** @param {HTMLElement} c */
  function build_sqlpage_chart(c) {
    const [data_element] = c.getElementsByTagName("data");
    const data = JSON.parse(data_element.textContent);
    const chartContainer = /** @type {HTMLElement} */ (
      c.querySelector(".chart")
    );
    chartContainer.innerHTML = "";
    const is_timeseries = !!data.time;
    const chart_type =
      APEXCHARTS_TYPE_ALIASES[data.type] || data.type || "line";
    const is_stacked =
      !!data.stacked && STACKABLE_CHART_TYPES.includes(chart_type);
    /** @type {DataPoint[]} */
    const points = data.points
      .filter(Array.isArray)
      .map(([name, x, y, color, z, link]) => ({
        name,
        x,
        y,
        color,
        z,
        link: link ?? undefined,
      }));
    /** @type {ReferenceLine[]} */
    const reference_rows = data.points.filter((row) => !Array.isArray(row));
    /** @type { Series } */
    const series_map = new Map();
    for (const { name, x: old_x, y: old_y, color, z, link } of points) {
      /** @type {ChartSeries} */
      const point_series = series_map.get(name) ?? { name, data: [] };
      series_map.set(name, point_series);
      /** @type {string|number|Date|null} */
      let x = old_x;
      let y = old_y;
      if (is_timeseries) {
        if (typeof x === "number") x = new Date(x * 1000);
        else if (chart_type === "rangeBar" && Array.isArray(y))
          y = y.map((y) => new Date(y).getTime());
        else x = new Date(/** @type {string|number} */ (x ?? 0));
      }
      point_series.data.push(
        /** @type {ChartPoint} */ ({
          x,
          y,
          z,
          link,
          fillColor: named_color(color),
        }),
      );
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
      chart_type === "pie" ? points.map(({ name, x }) => x || name) : undefined;
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
    const has_point_links = points.some((point) => point.link);
    const text_x_values = chart_series.every(({ data }) =>
      data.every(({ x }) => x == null || typeof x === "string"),
    );
    const options = {
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
            const link = pointLink(args, points);
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
            ? (_val, { seriesIndex, w }) => w.config.series[seriesIndex].name
            : chart_type === "pie"
              ? (value, { seriesIndex, w }) =>
                  `${w.config.labels[seriesIndex]}: ${value.toFixed()}%`
              : (value) => value?.toLocaleString?.() || value,
      },
      fill: {
        type: chart_type === "area" ? "gradient" : "solid",
      },
      stroke: {
        width:
          {
            area: 3,
            line: 2,
          }[chart_type] || 0,
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
          text: data.xtitle || undefined,
        },
        type: xaxis_type,
        labels: {
          datetimeUTC: false,
        },
      },
      yaxis: {
        logarithmic: !!data.logarithmic,
        min: data.ymin,
        max: data.ymax,
        stepSize: data.ystep,
        tickAmount: data.yticks,
        title: {
          text: data.ytitle || undefined,
        },
      },
      zaxis: {
        title: {
          text: data.ztitle || undefined,
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
            ? bubbleTooltip
            : undefined,
        x: {
          formatter:
            has_point_links && text_x_values
              ? (value, args) =>
                  formatTooltipX(value, args?.w && pointLink(args, points))
              : undefined,
        },
        y: {
          formatter: (value) => {
            if (value == null) return "";
            if (is_timeseries && chart_type === "rangeBar") {
              const d = new Date(value);
              if (d.getHours() === 0 && d.getMinutes() === 0)
                return d.toLocaleDateString();
              return d.toLocaleString();
            }
            return value.toLocaleString(undefined, {
              maximumFractionDigits: 2,
            });
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
    // Numeric axes count intervals; category and time axes use tickAmount as a
    // target for label density.
    if (data.xticks) options.xaxis.tickAmount = data.xticks;
    const chart = new ApexCharts(
      chartContainer,
      /** @type {import("apexcharts").ApexOptions} */ (options),
    );
    chart.render().catch(console.error);
    if (window.charts) window.charts.push(chart);
    else window.charts = [chart];
    c.removeAttribute("data-pre-init");
  }

  /**
   * @param {{seriesIndex:number, dataPointIndex:number, w:any}} args
   */
  function chartTooltip({ seriesIndex, dataPointIndex, w }) {
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

    for (const axis of ["x", "y", "z"]) {
      const value = point[axis];
      if (value == null) continue;
      const axisValue = document.createElement("div");
      axisValue.className = "apexcharts-tooltip-y-group";
      let axis_conf = w.config[`${axis}axis`];
      if (axis_conf.length) axis_conf = axis_conf[0];
      const title = axis_conf.title.text || axis;
      const labelSpan = document.createElement("span");
      labelSpan.className = "apexcharts-tooltip-text-y-label";
      labelSpan.innerText = `${title}: `;
      axisValue.appendChild(labelSpan);
      const valueSpan = document.createElement("span");
      valueSpan.className = "apexcharts-tooltip-text-y-value";
      const formatter = axis === "y" && w.config.tooltip.y.formatter;
      const format = (v) =>
        formatter ? formatter(v, { seriesIndex, dataPointIndex, w }) : v;
      valueSpan.innerText = Array.isArray(value)
        ? value.map(format).join(" - ")
        : format(value);
      axisValue.appendChild(valueSpan);
      tooltip.appendChild(axisValue);
    }
    return tooltip.outerHTML;
  }

  /**
   * @param {{seriesIndex:number, dataPointIndex:number, w:any}} args
   * @param {DataPoint[]} points
   */
  function pointLink({ seriesIndex, dataPointIndex, w }, points) {
    const series = w.config.series[seriesIndex];
    return Array.isArray(series?.data)
      ? series.data[dataPointIndex]?.link
      : points[seriesIndex]?.link;
  }

  function bubbleTooltip(args) {
    return chartTooltip(args);
  }

  return sqlpage_chart;
})();

add_init_fn(sqlpage_chart);

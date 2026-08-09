/* !include https://cdn.jsdelivr.net/npm/apexcharts@5.13.0/dist/apexcharts.min.js */

sqlpage_chart = (() => {
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
  const Y_WHEN_A_SERIES_SKIPS_A_LABEL = {
    bar: 0,
    line: null,
    area: null,
    scatter: null,
    bubble: null,
    heatmap: null,
  };

  /** @typedef {number|string|Date} XValue */
  /** @typedef { {name:string, data:{x:XValue,y:number|null,z?:number}[]} } ChartSeries */
  /** @typedef { { [name:string]: ChartSeries } } Series */

  /** @param {XValue} x @returns {number|string} equal x values share a key */
  const x_key = (x) => (x instanceof Date ? x.getTime() : x);

  /** @param {ChartSeries[]} series */
  const x_is_text = (series) => typeof series[0]?.data[0]?.x === "string";

  /**
   * @param {ChartSeries[]} series
   * @returns {XValue[]} every x the series hold, in their own order where they
   * agree and in ascending order where they diverge
   */
  function merged_x_values(series) {
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
  function align_series(series, y_when_missing) {
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
  function align_series_for(series, chart_type, is_stacked) {
    if (is_stacked) return align_series(series, 0);
    if (x_is_text(series) && chart_type in Y_WHEN_A_SERIES_SKIPS_A_LABEL)
      return align_series(series, Y_WHEN_A_SERIES_SKIPS_A_LABEL[chart_type]);
    return series;
  }

  // The unit tests load this file as a CommonJS module; browsers have no `module`.
  if (typeof module !== "undefined")
    module.exports = { align_series, align_series_for, merged_x_values };

  const referenceColor = colorNames[isDarkTheme ? "gray-lt" : "gray"];

  /** @typedef { {[property:string]: string|number|null} } ReferenceLine */

  /** @param {string|number|null} name */
  const reference_color = (name) =>
    (typeof name === "string" && colorNames[name]) || referenceColor;

  /**
   * @param {ReferenceLine[]} rows - the rows that carry a yline
   * @param {"x"|"y"} axis - the apexcharts axis the y column is drawn on
   * @param {(value: any) => any} to_axis_value - puts a SQL value on the axis
   * @returns {object[]} apexcharts axis annotations
   */
  function y_reference_lines(rows, axis, to_axis_value) {
    return rows.flatMap((row) => {
      if (row.yline == null) return [];
      const from = to_axis_value(row.yline);
      if (Number.isNaN(from)) return [];
      const color = reference_color(row.yline_color);
      const annotation = {
        [axis]: from,
        borderColor: color,
        fillColor: color,
        strokeDashArray: 4,
      };
      // apexcharts reads label.text unconditionally, so an annotation without
      // a label must not have the key at all.
      if (row.yline_label)
        annotation.label = {
          text: row.yline_label,
          orientation: "horizontal",
          borderColor: color,
          style: { background: color, color: isDarkTheme ? "#000" : "#fff" },
        };
      return [annotation];
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
    const points = data.points.filter(Array.isArray);
    const reference_rows = data.points.filter((row) => !Array.isArray(row));
    /** @type { Series } */
    const series_map = {};
    for (const [name, old_x, old_y, z] of points) {
      series_map[name] = series_map[name] || { name, data: [] };
      let x = old_x;
      let y = old_y;
      if (is_timeseries) {
        if (typeof x === "number") x = new Date(x * 1000);
        else if (chart_type === "rangeBar" && Array.isArray(y))
          y = y.map((y) => new Date(y).getTime());
        else x = new Date(x);
      }
      series_map[name].data.push({ x, y, z });
    }
    if (data.xmin == null) data.xmin = undefined;
    if (data.xmax == null) data.xmax = undefined;
    if (data.ymin == null) data.ymin = undefined;
    if (data.ymax == null) data.ymax = undefined;

    const colors = [
      ...data.colors.filter((c) => c).map((c) => colorNames[c]),
      ...tblrColors.map(([_, dark, light]) => (isDarkTheme ? dark : light)),
      ...tblrColors.map(([_, dark, light]) => (isDarkTheme ? light : dark)),
    ];

    let series = Object.values(series_map);

    let labels;
    const categories = x_is_text(series);
    if (chart_type === "pie") {
      labels = points.map(([name, x, _y]) => x || name);
      series = points.map(([_name, _x, y]) => Number.parseFloat(y));
    } else if (series.length > 1)
      series = align_series_for(series, chart_type, is_stacked);

    const to_value =
      is_timeseries && chart_type === "rangeBar"
        ? (v) =>
            (typeof v === "number" ? new Date(v * 1000) : new Date(v)).getTime()
        : Number;
    const inverted =
      chart_type === "rangeBar" || (chart_type === "bar" && !!data.horizontal);
    const value_axis = inverted ? "x" : "y";
    const options = {
      annotations: {
        [`${value_axis}axis`]: y_reference_lines(
          reference_rows,
          value_axis,
          to_value,
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
        type: is_timeseries ? "datetime" : categories ? "category" : undefined,
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
        custom:
          chart_type === "bubble" || chart_type === "scatter"
            ? bubbleTooltip
            : undefined,
        y: {
          formatter: (value) => {
            if (value == null) return "";
            if (is_timeseries && chart_type === "rangeBar") {
              const d = new Date(value);
              if (d.getHours() === 0 && d.getMinutes() === 0)
                return d.toLocaleDateString();
              return d.toLocaleString();
            }
            const str_val = value.toLocaleString();
            if (str_val.length > 10 && Number.isNaN(value))
              return value.toFixed(2);
            return str_val;
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
    // tickamount is the number of intervals, not the number of ticks
    if (data.xticks) options.xaxis.tickAmount = data.xticks;
    const chart = new ApexCharts(chartContainer, options);
    chart.render();
    if (window.charts) window.charts.push(chart);
    else window.charts = [chart];
    c.removeAttribute("data-pre-init");
  }

  function bubbleTooltip({ seriesIndex, dataPointIndex, w }) {
    const { name, data } = w.config.series[seriesIndex];
    const point = data[dataPointIndex];

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
      valueSpan.innerText = value;
      axisValue.appendChild(valueSpan);
      tooltip.appendChild(axisValue);
    }
    return tooltip.outerHTML;
  }

  return sqlpage_chart;
})();

add_init_fn(sqlpage_chart);

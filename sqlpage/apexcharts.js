/* !include https://cdn.jsdelivr.net/npm/apexcharts@4.5.0/dist/apexcharts.min.js */

sqlpage_chart = (() => {
  function sqlpage_chart() {
    for (const c of document.querySelectorAll("[data-pre-init=chart]")) {
      try {
        build_sqlpage_chart(c);
      } catch (e) {
        console.error(e);
      }
    }
  }

  const tblrColors = Object.fromEntries(
    [
      "azure",
      "red",
      "lime",
      "purple",
      "yellow",
      "gray-600",
      "orange",
      "pink",
      "teal",
      "indigo",
      "cyan",
      "green",
      "blue-lt",
      "yellow-lt",
      "pink-lt",
      "green-lt",
      "orange-lt",
      "blue",
      "gray-500",
      "gray-400",
      "gray-300",
      "gray-200",
      "gray-100",
      "gray-50",
      "black",
    ].map((c) => [
      c,
      getComputedStyle(document.documentElement).getPropertyValue(
        `--tblr-${c}`,
      ),
    ]),
  );

  /** @typedef { { [name:string]: {data:{x:number|string|Date,y:number}[], name:string} } } Series */

  /**
   * Aligns series data points by their x-axis categories, ensuring all series have data points
   * for each unique category. Missing values are filled with zeros. Categories are sorted.
   *
   * @example
   * // Input series:
   * const series = [
   *   { name: "A", data: [{x: "Jan", y: 10}, {x: "Mar", y: 30}] },
   *   { name: "B", data: [{x: "Jan", y: 20}, {x: "Feb", y: 25}] }
   * ];
   *
   * // Output after align_categories:
   * // [
   * //   { name: "A", data: [{x: "Feb", y: 0}, {x: "Jan", y: 10}, {x: "Mar", y: 30}] },
   * //   { name: "B", data: [{x: "Feb", y: 25}, {x: "Jan", y: 20}, {x: "Mar", y: 0}] }
   * // ]
   *
   * @param {Series[string][]} series - Array of series objects, each containing name and data points
   * @returns {Series[string][]} Aligned series with consistent categories across all series
   */
  function align_categories(series) {
    // Collect all unique categories
    const categories = new Set(series.flatMap((s) => s.data.map((p) => p.x)));
    const sortedCategories = Array.from(categories).sort();

    // Create a map of category -> value for each series
    return series.map((s) => {
      const valueMap = new Map(s.data.map((point) => [point.x, point.y]));

      return {
        name: s.name,
        data: sortedCategories.map((category) => ({
          x: category,
          y: valueMap.get(category) || 0,
        })),
      };
    });
  }

  /** @param {HTMLElement} c */
  function build_sqlpage_chart(c) {
    const [data_element] = c.getElementsByTagName("data");
    const data = JSON.parse(data_element.textContent);
    const chartContainer = c.querySelector(".chart");
    chartContainer.innerHTML = "";
    const is_timeseries = !!data.time;
    /** @type { Series } */
    const series_map = {};
    for (const [name, old_x, old_y, z] of data.points) {
      series_map[name] = series_map[name] || { name, data: [] };
      let x = old_x;
      let y = old_y;
      if (is_timeseries) {
        if (typeof x === "number") x = new Date(x * 1000);
        else if (data.type === "rangeBar" && Array.isArray(y))
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
      ...data.colors.filter((c) => c).map((c) => tblrColors[c]),
      ...Object.values(tblrColors),
    ];

    let series = Object.values(series_map);

    let labels;
    const categories =
      series.length > 0 && typeof series[0].data[0].x === "string";
    if (data.type === "pie") {
      labels = data.points.map(([name, x, y]) => x || name);
      series = data.points.map(([name, x, y]) => y);
    } else if (categories && data.type === "bar")
      series = align_categories(series);

    const options = {
      chart: {
        type: data.type || "line",
        fontFamily: "inherit",
        parentHeightOffset: 0,
        height: chartContainer.style.height,
        stacked: !!data.stacked,
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
        palette: "palette4",
      },
      dataLabels: {
        enabled: !!data.labels,
        dropShadow: {
          enabled: true,
          color: "#f6f8fb",
        },
        formatter:
          data.type === "rangeBar"
            ? (_val, { seriesIndex, w }) => w.config.series[seriesIndex].name
            : data.type === "pie"
              ? (value, { seriesIndex, w }) =>
                  `${w.config.labels[seriesIndex]}: ${value.toFixed()}%`
              : (value) => value.toLocaleString(),
      },
      fill: {
        type: data.type === "area" ? "gradient" : "solid",
      },
      stroke: {
        width: data.type === "area" ? 3 : 1,
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
          data.type === "bubble" || data.type === "scatter"
            ? bubbleTooltip
            : undefined,
        y: {
          formatter: (value) => {
            if (is_timeseries && data.type === "rangeBar") {
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
          horizontal: !!data.horizontal || data.type === "rangeBar",
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
    console.log("Rendering chart", options);
    const chart = new ApexCharts(chartContainer, options);
    chart.render();
    if (window.charts) window.charts.push(chart);
    else window.charts = [chart];
    c.removeAttribute("data-pre-init");
  }

  function bubbleTooltip({ series, seriesIndex, dataPointIndex, w }) {
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

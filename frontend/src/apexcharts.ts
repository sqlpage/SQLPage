import ApexCharts, { type ApexOptions } from "apexcharts";
import {
  align_series_for,
  type ChartSeries,
  x_is_text,
} from "./chart_series.ts";
import { add_init_fn } from "./init.ts";

const tblrColors: [name: string, dark: string, light: string][] = [
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

const colorNames: Record<string, string> = Object.fromEntries(
  tblrColors.flatMap(([name, dark, light]) => [
    [name, dark],
    [`${name}-lt`, light],
  ]),
);

const isDarkTheme = document.body?.dataset?.bsTheme === "dark";

type ChartType = NonNullable<NonNullable<ApexOptions["chart"]>["type"]>;

/** ApexCharts has no z axis; our bubble tooltip reads its title back off the config. */
type SqlpageChartOptions = ApexOptions & {
  zaxis: { title: { text?: string } };
};

const STACKABLE_CHART_TYPES = ["line", "area", "bar"];
const APEXCHARTS_TYPE_ALIASES: Record<string, ChartType> = { column: "bar" };

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

function build_sqlpage_chart(c: HTMLElement) {
  const [data_element] = c.getElementsByTagName("data");
  const data = JSON.parse(data_element.textContent ?? "");
  const chartContainer = c.querySelector<HTMLElement>(".chart");
  if (!chartContainer) return;
  chartContainer.innerHTML = "";
  const is_timeseries = !!data.time;
  const chart_type: ChartType =
    APEXCHARTS_TYPE_ALIASES[data.type] || data.type || "line";
  const is_stacked =
    !!data.stacked && STACKABLE_CHART_TYPES.includes(chart_type);
  const series_map: Record<string, ChartSeries> = {};
  for (const [name, old_x, old_y, z] of data.points) {
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
    ...data.colors.filter((c: string) => c).map((c: string) => colorNames[c]),
    ...tblrColors.map(([_, dark, light]) => (isDarkTheme ? dark : light)),
    ...tblrColors.map(([_, dark, light]) => (isDarkTheme ? light : dark)),
  ];

  const named_series = Object.values(series_map);
  const categories = x_is_text(named_series);
  const is_pie = chart_type === "pie";
  const series = is_pie
    ? data.points.map(([_name, _x, y]: [string, unknown, string]) =>
        Number.parseFloat(y),
      )
    : named_series.length > 1
      ? align_series_for(named_series, chart_type, is_stacked)
      : named_series;

  const options: SqlpageChartOptions = {
    ...(is_pie && {
      labels: data.points.map(([name, x]: [string, unknown]) => x || name),
    }),
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
          ? (_val: unknown, { seriesIndex, w }: Untyped) =>
              w.config.series[seriesIndex].name
          : is_pie
            ? (value: number, { seriesIndex, w }: Untyped) =>
                `${w.config.labels[seriesIndex]}: ${value.toFixed()}%`
            : (value: Untyped) => value?.toLocaleString?.() || value,
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
      // ApexCharts counts intervals here rather than ticks
      tickAmount: data.xticks || undefined,
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
        formatter: (value: number | null) => {
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
  const chart = new ApexCharts(chartContainer, options);
  chart.render();
  if (window.charts) window.charts.push(chart);
  else window.charts = [chart];
  c.removeAttribute("data-pre-init");
}

function bubbleTooltip({ seriesIndex, dataPointIndex, w }: Untyped) {
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

add_init_fn(sqlpage_chart);

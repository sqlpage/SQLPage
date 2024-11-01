/* !include https://cdn.jsdelivr.net/npm/apexcharts@4.0.0/dist/apexcharts.min.js */

sqlpage_chart = (function () {

    function sqlpage_chart() {
        for (const c of document.querySelectorAll("[data-pre-init=chart]")) {
            try { build_sqlpage_chart(c) }
            catch (e) { console.log(e) }
        }
    }


    const tblrColors = Object.fromEntries(['azure', 'red', 'lime', 'purple', 'yellow', 'gray-600', 'orange', 'pink', 'teal', 'indigo', 'cyan', 'green', 'blue-lt', 'yellow-lt', 'pink-lt', 'green-lt', 'orange-lt', 'blue', 'gray-500', 'gray-400', 'gray-300', 'gray-200', 'gray-100', 'gray-50', 'black']
        .map(c => [c, getComputedStyle(document.documentElement).getPropertyValue('--tblr-' + c)]));

    /** @typedef { { [name:string]: {data:{x:number,y:number}[], name:string} } } Series */

    /**
     * @param {Series} series
     * @returns {Series} */
    function align_categories(series) {
        const new_series = series.map(s => ({ name: s.name, data: [] }));
        do {
            var category = null;
            series.forEach((s, s_i) => {
                const point = s.data[0];
                let new_point = { x: category, y: 0 };
                if (point) {
                    if (category == null) category = point.x;
                    if (category === point.x) {
                        new_point = s.data.shift();
                    }
                }
                new_series[s_i].data.push(new_point);
            })
            new_series.forEach(s => s.data[s.data.length - 1].x = category);
        } while (category != null);
        new_series.forEach(s => s.data.pop());
        return new_series;
    }


    /** @param {HTMLElement} c */
    function build_sqlpage_chart(c) {
        const [data_element] = c.getElementsByTagName("data");
        const data = JSON.parse(data_element.textContent);
        const chartContainer = c.querySelector('.chart');
        chartContainer.innerHTML = "";
        const is_timeseries = !!data.time;
        /** @type { Series } */
        const series_map = {};
        data.points.forEach(([name, x, y, z]) => {
            series_map[name] = series_map[name] || { name, data: [] }
            if (is_timeseries) {
                if (typeof x === 'number') x = new Date(x * 1000) // databases use seconds; JS uses ms
                else if (data.type === 'rangeBar' && Array.isArray(y)) y = y.map(y => new Date(y).getTime()); // timerange charts
                else x = new Date(x);
            }
            series_map[name].data.push({ x, y, z });
        })
        if (data.xmin == null) data.xmin = undefined;
        if (data.xmax == null) data.xmax = undefined;
        if (data.ymin == null) data.ymin = undefined;
        if (data.ymax == null) data.ymax = undefined;

        const colors = [
            ...data.colors.filter(c => c).map(c => tblrColors[c]),
            ...Object.values(tblrColors)
        ];

        let series = Object.values(series_map);

        // tickamount is the number of intervals, not the number of ticks
        const tickAmount = data.xticks ||
            Math.min(30, Math.max(...series.map(s => s.data.length - 1)));

        let labels;
        const categories = series.length > 0 && typeof series[0].data[0].x === "string";
        if (data.type === "pie") {
            labels = data.points.map(([name, x, y]) => x || name);
            series = data.points.map(([name, x, y]) => y);
        } else if (categories && data.type === 'bar') series = align_categories(series);

        const options = {
            chart: {
                type: data.type || 'line',
                fontFamily: 'inherit',
                parentHeightOffset: 0,
                height: chartContainer.style.height,
                stacked: !!data.stacked,
                toolbar: {
                    show: !!data.toolbar,
                },
                animations: {
                    enabled: false
                },
                zoom: {
                    enabled: false
                }
            },
            theme: {
                palette: 'palette4',
            },
            dataLabels: {
                enabled: !!data.labels,
                dropShadow: {
                    enabled: true,
                    color: '#f6f8fb',
                },
                formatter: (data.type === 'rangeBar') ? (_val, { seriesIndex, w }) => w.config.series[seriesIndex].name :
                    (data.type === 'pie') ? (value, { seriesIndex, w }) => `${w.config.labels[seriesIndex]}: ${value.toFixed()}%`
                        : value => value.toLocaleString(),
            },
            fill: {
                type: data.type === 'area' ? 'gradient' : 'solid',
            },
            stroke: {
                width: data.type === 'area' ? 3 : 1,
                lineCap: "round",
                curve: "smooth",
            },
            xaxis: {
                tooltip: {
                    enabled: false
                },
                min: data.xmin,
                max: data.xmax,
                tickAmount,
                title: {
                    text: data.xtitle || undefined,
                },
                type: is_timeseries ? 'datetime' : categories ? 'category' : undefined,
                labels: {
                    datetimeUTC: false,
                }
            },
            yaxis: {
                logarithmic: !!data.logarithmic,
                min: data.ymin,
                max: data.ymax,
                stepSize: data.ystep,
                tickAmount: data.yticks,
                title: {
                    text: data.ytitle || undefined,
                }
            },
            zaxis: {
                title: {
                    text: data.ztitle || undefined,
                }
            },
            markers: {
                size: data.marker || 0,
                strokeWidth: 0,
                hover: {
                    sizeOffset: 5,
                }
            },
            tooltip: {
                fillSeriesColor: false,
                custom: (data.type === 'bubble' || data.type === 'scatter') ? bubbleTooltip : undefined,
                y: {
                    formatter: function (value) {
                        if (is_timeseries && data.type === 'rangeBar') {
                            const d = new Date(value);
                            if (d.getHours() === 0 && d.getMinutes() === 0) return d.toLocaleDateString();
                            return d.toLocaleString();
                        }
                        const str_val = value.toLocaleString();
                        if (str_val.length > 10 && value != value | 0) return value.toFixed(2);
                        return str_val;
                    }
                }
            },
            plotOptions: {
                bar: {
                    horizontal: !!data.horizontal || data.type === 'rangeBar',
                    borderRadius: 5,
                },
                bubble: { minBubbleRadius: 5, },
            },
            colors,
            series,
        };
        if (labels) options.labels = labels;
        const chart = new ApexCharts(chartContainer, options);
        chart.render();
        if (window.charts) window.charts.push(chart);
        else window.charts = [chart];
        c.removeAttribute("data-pre-init");
    }

    function bubbleTooltip({ series, seriesIndex, dataPointIndex, w }) {
        const { name, data } = w.config.series[seriesIndex];
        const point = data[dataPointIndex];

        const tooltip = document.createElement('div');
        tooltip.className = 'apexcharts-tooltip-text';
        tooltip.style.fontFamily = 'inherit';

        const seriesName = document.createElement('div');
        seriesName.className = 'apexcharts-tooltip-y-group';
        seriesName.style.fontWeight = 'bold';
        seriesName.innerText = name;
        tooltip.appendChild(seriesName);

        for (const axis of ['x', 'y', 'z']) {
            const value = point[axis];
            if (value == null) continue;
            const axisValue = document.createElement('div');
            axisValue.className = 'apexcharts-tooltip-y-group';
            let axis_conf = w.config[axis + 'axis'];
            if (axis_conf.length) axis_conf = axis_conf[0];
            const title = axis_conf.title.text || axis;
            const labelSpan = document.createElement('span');
            labelSpan.className = 'apexcharts-tooltip-text-y-label';
            labelSpan.innerText = title + ': ';
            axisValue.appendChild(labelSpan);
            const valueSpan = document.createElement('span');
            valueSpan.className = 'apexcharts-tooltip-text-y-value';
            valueSpan.innerText = value;
            axisValue.appendChild(valueSpan);
            tooltip.appendChild(axisValue);
        }
        return tooltip.outerHTML;
    }

    return sqlpage_chart;
})();

add_init_fn(sqlpage_chart);
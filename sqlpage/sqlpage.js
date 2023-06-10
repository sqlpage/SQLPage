/* !include https://cdn.jsdelivr.net/npm/@tabler/core@1.0.0-beta19/dist/js/tabler.min.js */
/* !include https://cdn.jsdelivr.net/npm/list.js@2.3.1/dist/list.min.js */
/* !include https://cdn.jsdelivr.net/npm/apexcharts@3.41.0/dist/apexcharts.min.js */

function sqlpage_chart() {
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
          let new_point = { x: category, y: NaN };
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

    for (const c of document.getElementsByClassName("chart")) {
      try {
        const data = JSON.parse(c.querySelector("data").innerText);
        /** @type { Series } */
        const series_map = {};
        data.points.forEach(([name, x, y]) => {
          series_map[name] = series_map[name] || { name, data: [] }
          series_map[name].data.push({ x, y });
        })
        if (data.xmin == null) data.xmin = undefined;
        if (data.xmax == null) data.xmax = undefined;
        if (data.ymin == null) data.ymin = 0;
        if (data.ymax == null) data.ymax = undefined;

        let series = Object.values(series_map);

        // tickamount is the number of intervals, not the number of ticks
        const tickAmount = data.xticks ||
          Math.min(30, Math.max(...series.map(s => s.data.length - 1)));

        let labels;
        const categories = typeof data.points[0][1] === "string";
        if (data.type === "pie") {
            labels = data.points.map(([name, x, y]) => x || name);
            series = data.points.map(([name, x, y]) => y);
        } else if (categories) series = align_categories(series);

        const options = {
          chart: {
            type: data.type || 'line',
            fontFamily: 'inherit',
            parentHeightOffset: 0,
            height: c.style.height,
            stacked: !!data.stacked,
            toolbar: {
              show: false,
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
          },
          fill: {
            opacity: .7,
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
            type: data.time ? 'datetime' : categories ? 'category' : undefined,
          },
          yaxis: {
            logarithmic: !!data.logarithmic,
            min: data.ymin,
            max: data.ymax,
            title: {
              text: data.ytitle || undefined,
            }
          },
          markers: {
            size: data.marker || 0,
            strokeWidth: 0,
            hover: {
              sizeOffset: 5,
            }
          },
          series,
        };
        if (labels) options.labels = labels;
        c.innerHTML = "";
        const chart = new ApexCharts(c, options);
        chart.render();
        if (window.charts) window.charts.push(chart);
        else window.charts = [chart];
      } catch (e) { console.log(e) }
    }
}

function sqlpage_table(){
    // Tables
    for (const r of document.getElementsByClassName("data-list")) {
        new List(r, {
            valueNames: [...r.getElementsByClassName("sort")]
                .map(t => t.dataset.sort),
            indexAsync: true
        });
    }
}

document.addEventListener('DOMContentLoaded', function () {
    sqlpage_table();
    sqlpage_chart();
})
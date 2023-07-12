/* !include https://cdn.jsdelivr.net/npm/@tabler/core@1.0.0-beta19/dist/js/tabler.min.js */
/* !include https://cdn.jsdelivr.net/npm/list.js@2.3.1/dist/list.min.js */
/* !include https://cdn.jsdelivr.net/npm/apexcharts@3.41.0/dist/apexcharts.min.js */

function sqlpage_chart() {

    const tblrColors = Object.fromEntries(['azure', 'red', 'lime', 'blue', 'pink', 'indigo', 'purple', 'yellow', 'cyan', 'green', 'orange', 'cyan']
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
          tooltip: {
            fillSeriesColor: false,
          },
          plotOptions: { bar: { horizontal: !!data.horizontal } },
          colors,
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

function sqlpage_map() {
    const maps = document.getElementsByClassName("leaflet");
    if (maps.length) {
      // Add the leaflet js and css to the page
      const leaflet_css = document.createElement("link");
      leaflet_css.rel = "stylesheet";
      leaflet_css.href = "https://cdn.jsdelivr.net/npm/leaflet@1.9.4/dist/leaflet.css";
      leaflet_css.integrity = "sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY=";
      leaflet_css.crossOrigin = "anonymous";
      document.head.appendChild(leaflet_css);
      const leaflet_js = document.createElement("script");
      leaflet_js.src = "https://cdn.jsdelivr.net/npm/leaflet@1.9.4/dist/leaflet.js";
      leaflet_js.integrity = "sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo=";
      leaflet_js.crossOrigin = "anonymous";
      leaflet_js.onload = onLeafletLoad;
      document.head.appendChild(leaflet_js);
    }
    function onLeafletLoad() {
      for (const m of maps) {
        const map = L.map(m);
        const center = m.dataset.center.split(",").map(c => parseFloat(c));
        map.setView(center, +m.dataset.zoom);
        L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
          attribution: '&copy; OpenStreetMap',
          maxZoom: 18,
        }).addTo(map);
        for (const marker_elem of m.getElementsByClassName("marker")) {
          setTimeout(addMarker, 0, marker_elem, map);
        }
      }
    }
    function addMarker(marker_elem, map) {
      const coords = marker_elem.dataset.coords.split(",").map(c => parseFloat(c));
      const marker = L.marker(coords).addTo(map);
      marker.bindPopup(marker_elem);
    }
}

document.addEventListener('DOMContentLoaded', function () {
    sqlpage_table();
    sqlpage_chart();
    sqlpage_map();
})
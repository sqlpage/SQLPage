/* !include https://cdn.jsdelivr.net/npm/@tabler/core@1.0.0-beta19/dist/js/tabler.min.js */
/* !include https://cdn.jsdelivr.net/npm/list.js-fixed@2.3.2/dist/list.min.js */

function sqlpage_chart() {
  let first_chart = document.querySelector("[data-js]");
  if (first_chart) {
    // Add the apexcharts js to the page
    const apexcharts_js = document.createElement("script");
    apexcharts_js.src = first_chart.dataset.js;
    document.head.appendChild(apexcharts_js);
  }
}

function sqlpage_table(){
    // Tables
    for (const r of document.getElementsByClassName("data-list")) {
        new List(r, {
            valueNames: [...r.getElementsByTagName("th")].map(t => t.textContent),
            searchDelay: 100,
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
/* !include https://cdn.jsdelivr.net/npm/@tabler/core@1.4.0/dist/js/tabler.min.js */
const nonce = document.currentScript.nonce;

function sqlpage_card() {
  for (const c of document.querySelectorAll("[data-pre-init=card]")) {
    c.removeAttribute("data-pre-init");
    const url = new URL(c.dataset.embed, window.location.href);
    url.searchParams.set("_sqlpage_embed", "1");
    fetch(url)
      .then((res) => res.text())
      .then((html) => {
        const body = c.querySelector(".card-content");
        body.innerHTML = html;
        const spinner = c.querySelector(".card-loading-placeholder");
        if (spinner) {
          spinner.parentNode.removeChild(spinner);
        }
        const fragLoadedEvt = new CustomEvent("fragment-loaded", {
          bubbles: true,
        });
        c.dispatchEvent(fragLoadedEvt);
      });
  }
}

/** @param {HTMLElement} root_el */
function setup_table(root_el) {
  /** @type {HTMLInputElement | null} */
  const search_input = root_el.querySelector("input.search");
  const table_el = root_el.querySelector("table");
  const sort_buttons = [...table_el.querySelectorAll("button.sort[data-sort]")];
  const item_parent = table_el.querySelector("tbody");
  const has_sort = sort_buttons.length > 0;

  if (search_input || has_sort) {
    const items = table_parse_data(table_el, sort_buttons);
    if (search_input) setup_table_search_behavior(search_input, items);
    if (has_sort) setup_sort_behavior(sort_buttons, items, item_parent);
  }

  // Change number format AFTER parsing and storing the sort keys
  apply_number_formatting(table_el);
}

/**
 * @param {HTMLInputElement} search_input
 * @param {Array<{el: HTMLElement, sort_keys: Array<{num: number, str: string}>}>} items
 */
function setup_table_search_behavior(search_input, items) {
  function onSearch() {
    const lower_search = search_input.value
      .toLowerCase()
      .split(/\s+/)
      .filter((s) => s);
    for (const item of items) {
      const show = lower_search.every((s) =>
        item.el.textContent.toLowerCase().includes(s),
      );
      item.el.style.display = show ? "" : "none";
    }
  }

  search_input.addEventListener("input", onSearch);
  onSearch();
}

/**@param {HTMLElement} table_el */
function apply_number_formatting(table_el) {
  const header_els = table_el.querySelectorAll("thead > tr > th");
  const col_types = [...header_els].map((el) => el.dataset.column_type);
  const col_rawnums = [...header_els].map((el) => !!el.dataset.raw_number);
  const col_money = [...header_els].map((el) => !!el.dataset.money);
  const number_format_locale = table_el.dataset.number_format_locale;
  const number_format_digits = table_el.dataset.number_format_digits;
  const currency = table_el.dataset.currency;

  for (const tr_el of table_el.querySelectorAll("tbody tr, tfoot tr")) {
    const cells = tr_el.getElementsByTagName("td");
    for (let idx = 0; idx < cells.length; idx++) {
      const column_type = col_types[idx];
      const is_raw_number = col_rawnums[idx];
      const cell_el = cells[idx];
      const text = cell_el.textContent;

      if (column_type === "number" && !is_raw_number && text) {
        const num = Number.parseFloat(text);
        const is_money = col_money[idx];
        cell_el.textContent = num.toLocaleString(number_format_locale, {
          maximumFractionDigits: number_format_digits,
          currency,
          style: is_money ? "currency" : undefined,
        });
      }
    }
  }
}

/** Prepare the table rows for sorting.
 * @param {HTMLElement} table_el
 * @param {HTMLElement[]} sort_buttons
 */
function table_parse_data(table_el, sort_buttons) {
  const is_num = [...sort_buttons].map(
    (btn_el) => btn_el.parentElement.dataset.column_type === "number",
  );
  return [...table_el.querySelectorAll("tbody tr")].map((tr_el) => {
    const cells = tr_el.getElementsByTagName("td");
    return {
      el: tr_el,
      sort_keys: sort_buttons.map((_btn_el, idx) => {
        const str = cells[idx]?.textContent;
        const num = is_num[idx] ? Number.parseFloat(str) : Number.NaN;
        return { num, str };
      }),
    };
  });
}

/**
 * Adds event listeners to the sort buttons to sort the table rows.
 * @param {HTMLElement[]} sort_buttons
 * @param {HTMLElement[]} items
 * @param {HTMLElement} item_parent
 */
function setup_sort_behavior(sort_buttons, items, item_parent) {
  sort_buttons.forEach((button, button_index) => {
    button.addEventListener("click", function sort_items() {
      const sort_desc = button.classList.contains("asc");
      for (const b of sort_buttons) {
        b.classList.remove("asc", "desc");
      }
      button.classList.add(sort_desc ? "desc" : "asc");
      const multiplier = sort_desc ? -1 : 1;
      items.sort((a, b) => {
        const a_key = a.sort_keys[button_index];
        const b_key = b.sort_keys[button_index];
        return (
          multiplier *
          (Number.isNaN(a_key.num) || Number.isNaN(b_key.num)
            ? a_key.str.localeCompare(b_key.str)
            : a_key.num - b_key.num)
        );
      });
      item_parent.append(...items.map((item) => item.el));
    });
  });
}

function sqlpage_table() {
  for (const r of document.querySelectorAll("[data-pre-init=table]")) {
    r.removeAttribute("data-pre-init");
    setup_table(r);
  }
}

let is_leaflet_injected = false;
let is_leaflet_loaded = false;

function sqlpage_map() {
  const first_map = document.querySelector("[data-pre-init=map]");
  const leaflet_base_url = "https://cdn.jsdelivr.net/npm/leaflet@1.9.4";
  if (first_map && !is_leaflet_injected) {
    // Add the leaflet js and css to the page
    const leaflet_css = document.createElement("link");
    leaflet_css.rel = "stylesheet";
    leaflet_css.href = `${leaflet_base_url}/dist/leaflet.css`;
    leaflet_css.integrity =
      "sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY=";
    leaflet_css.crossOrigin = "anonymous";
    document.head.appendChild(leaflet_css);
    const leaflet_js = document.createElement("script");
    leaflet_js.src = `${leaflet_base_url}/dist/leaflet.js`;
    leaflet_js.integrity =
      "sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo=";
    leaflet_js.crossOrigin = "anonymous";
    leaflet_js.nonce = nonce;
    leaflet_js.onload = onLeafletLoad;
    document.head.appendChild(leaflet_js);
    is_leaflet_injected = true;
  }
  if (first_map && is_leaflet_loaded) {
    onLeafletLoad();
  }
  /**
   *
   * @param {string|undefined} coords
   * @returns {[number, number] | undefined}
   */
  function parseCoords(coords) {
    return coords?.split(",", 2).map((c) => Number.parseFloat(c));
  }
  function onLeafletLoad() {
    is_leaflet_loaded = true;
    const maps = document.querySelectorAll("[data-pre-init=map]");
    for (const m of maps) {
      const tile_source = m.dataset.tile_source;
      const maxZoom = +m.dataset.max_zoom;
      const attribution = m.dataset.attribution;
      const map = L.map(m, { attributionControl: !!attribution });
      const zoom = m.dataset.zoom;
      const center = parseCoords(m.dataset.center);
      if (tile_source)
        L.tileLayer(tile_source, { attribution, maxZoom }).addTo(map);
      map._sqlpage_markers = [];
      for (const marker_elem of m.getElementsByClassName("marker")) {
        setTimeout(addMarker, 0, marker_elem, map);
      }
      setTimeout(() => {
        if (center) map.setView(center, +zoom);
        else {
          const markerBounds = (m) =>
            m.getLatLng ? m.getLatLng() : m.getBounds();
          const bounds = map._sqlpage_markers.map(markerBounds);
          if (bounds.length > 0) map.fitBounds(bounds);
          else map.setView([51.505, 10], +zoom);
          if (zoom != null) map.setZoom(+zoom);
        }
      }, 100);
      m.removeAttribute("data-pre-init");
      m.getElementsByClassName("spinner-border")[0]?.remove();
    }
  }

  function addMarker(marker_elem, map) {
    const { dataset } = marker_elem;
    const options = {
      color: marker_elem.dataset.color,
      title: marker_elem.getElementsByTagName("h3")[0].textContent.trim(),
    };
    const marker = dataset.coords
      ? createMarker(marker_elem, options)
      : createGeoJSONMarker(marker_elem, options);
    marker.addTo(map);
    map._sqlpage_markers.push(marker);
    if (marker_elem.textContent.trim()) marker.bindPopup(marker_elem);
    else if (marker_elem.dataset.link) {
      marker.on("click", () => {
        window.location.href = marker_elem.dataset.link;
      });
    }
  }
  function createMarker(marker_elem, options) {
    const coords = parseCoords(marker_elem.dataset.coords);
    const icon_obj = marker_elem.getElementsByClassName("mapicon")[0];
    if (icon_obj) {
      const size =
        1.5 *
        +(options.size || icon_obj.firstChild?.getAttribute("width") || 24);
      options.icon = L.divIcon({
        html: icon_obj,
        className: `border-0 bg-${options.color || "primary"} bg-gradient text-white rounded-circle shadow d-flex justify-content-center align-items-center`,
        iconSize: [size, size],
        iconAnchor: [size / 2, size / 2],
      });
    }
    return L.marker(coords, options);
  }
  function createGeoJSONMarker(marker_elem, options) {
    const geojson = JSON.parse(marker_elem.dataset.geojson);
    if (options.color) {
      options.color = get_tabler_color(options.color) || options.color;
    }
    function style({ properties }) {
      if (typeof properties !== "object") return options;
      return { ...options, ...properties };
    }
    function pointToLayer(feature, latlng) {
      marker_elem.dataset.coords = `${latlng.lat},${latlng.lng}`;
      return createMarker(marker_elem, { ...options, ...feature.properties });
    }
    return L.geoJSON(geojson, { style, pointToLayer });
  }
}

function sqlpage_form() {
  const file_inputs = document.querySelectorAll(
    "input[type=file][data-max-size]",
  );
  for (const input of file_inputs) {
    const max_size = +input.dataset.maxSize;
    input.addEventListener("change", function () {
      input.classList.remove("is-invalid");
      input.setCustomValidity("");
      for (const { size } of this.files) {
        if (size > max_size) {
          input.classList.add("is-invalid");
          return input.setCustomValidity(
            `File size must be less than ${max_size / 1000} kB.`,
          );
        }
      }
    });
  }

  const auto_submit_forms = document.querySelectorAll("form[data-auto-submit]");
  for (const form of auto_submit_forms) {
    form.addEventListener("change", () => form.submit());
  }
}

function get_tabler_color(name) {
  return getComputedStyle(document.documentElement).getPropertyValue(
    `--tblr-${name}`,
  );
}

function load_scripts() {
  const addjs = document.querySelectorAll("[data-sqlpage-js]");
  const existing_scripts = new Set(
    [...document.querySelectorAll("script")].map((s) => s.src),
  );
  for (const el of addjs) {
    const js = new URL(el.dataset.sqlpageJs, window.location.href).href;
    if (existing_scripts.has(js)) continue;
    existing_scripts.add(js);
    const script = document.createElement("script");
    script.src = js;
    document.head.appendChild(script);
  }
}

function add_init_fn(f) {
  document.addEventListener("DOMContentLoaded", f);
  document.addEventListener("fragment-loaded", f);
  if (document.readyState !== "loading") setTimeout(f, 0);
}

add_init_fn(sqlpage_table);
add_init_fn(sqlpage_map);
add_init_fn(sqlpage_card);
add_init_fn(sqlpage_form);
add_init_fn(load_scripts);

function init_bootstrap_components(event) {
  const bootstrap = window.bootstrap || window.tabler.bootstrap;
  const fragment = event.target;
  for (const el of fragment.querySelectorAll('[data-bs-toggle="tooltip"]')) {
    new bootstrap.Tooltip(el);
  }
  for (const el of fragment.querySelectorAll('[data-bs-toggle="popover"]')) {
    new bootstrap.Popover(el);
  }
  for (const el of fragment.querySelectorAll('[data-bs-toggle="dropdown"]')) {
    new bootstrap.Dropdown(el);
  }
  for (const el of fragment.querySelectorAll('[data-bs-ride="carousel"]')) {
    new bootstrap.Carousel(el);
  }
}

document.addEventListener("fragment-loaded", init_bootstrap_components);

function open_modal_for_hash() {
  const hash = window.location.hash.substring(1);
  if (!hash) return;
  const modal = document.getElementById(hash);
  const classes = modal.classList;
  if (!modal || !classes.contains("modal")) return;
  const bootstrap_modal =
    window.tabler.bootstrap.Modal.getOrCreateInstance(modal);
  bootstrap_modal.show();
  modal.addEventListener("hidden.bs.modal", () => {
    window.history.replaceState(null, "", "#");
  });
}

window.addEventListener("hashchange", open_modal_for_hash);
window.addEventListener("DOMContentLoaded", open_modal_for_hash);

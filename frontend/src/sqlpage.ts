import { bootstrap as bundled_bootstrap } from "@tabler/core";
import type * as Leaflet from "leaflet";
import { add_init_fn } from "./init.ts";

// A page may load its own Bootstrap; prefer it over the bundled copy.
const page_bootstrap = () => window.bootstrap ?? bundled_bootstrap;

/**
 * Bootstrap declares getOrCreateInstance on the base class, which returns a
 * BaseComponent and so loses show().
 */
type ToastWidget = InstanceType<typeof bundled_bootstrap.Toast>;
type ModalWidget = InstanceType<typeof bundled_bootstrap.Modal>;

const nonce = (document.currentScript as HTMLScriptElement).nonce;

function sqlpage_card() {
  const cards = document.querySelectorAll<HTMLElement>("[data-pre-init=card]");
  for (const c of cards) {
    c.removeAttribute("data-pre-init");
    if (!c.dataset.embed) continue;
    const url = new URL(c.dataset.embed, window.location.href);
    url.searchParams.set("_sqlpage_embed", "1");
    fetch(url)
      .then((res) => res.text())
      .then((html) => {
        const body = c.querySelector(".card-content");
        if (body) body.innerHTML = html;
        const spinner = c.querySelector(".card-loading-placeholder");
        spinner?.remove();
        const fragLoadedEvt = new CustomEvent("fragment-loaded", {
          bubbles: true,
        });
        c.dispatchEvent(fragLoadedEvt);
      })
      .catch((e) => {
        console.error(e);
        c.querySelector(".card-loading-placeholder")?.remove();
      });
  }
}

function setup_table(root_el: HTMLElement) {
  const search_input = root_el.querySelector<HTMLInputElement>("input.search");
  const table_el = root_el.querySelector("table");
  if (!table_el) return;
  const sort_button_els = table_el.querySelectorAll<HTMLElement>(
    "button.sort[data-sort]",
  );
  const sort_buttons = [...sort_button_els];
  const item_parent = table_el.querySelector("tbody");
  const has_sort = sort_buttons.length > 0;

  if (search_input || has_sort) {
    const items = table_parse_data(table_el, sort_buttons);
    if (search_input) setup_table_search_behavior(search_input, items);
    if (has_sort && item_parent)
      setup_sort_behavior(sort_buttons, items, item_parent);
  }

  // Change number format AFTER parsing and storing the sort keys
  apply_number_formatting(table_el);
}

/**
 */
function setup_table_search_behavior(
  search_input: HTMLInputElement,
  items: TableRow[],
) {
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

function apply_number_formatting(table_el: HTMLElement) {
  const header_els = table_el.querySelectorAll<HTMLElement>("thead > tr > th");
  const col_types = [...header_els].map((el) => el.dataset.column_type);
  const col_rawnums = [...header_els].map((el) => !!el.dataset.raw_number);
  const col_money = [...header_els].map((el) => !!el.dataset.money);
  const number_format_locale = table_el.dataset.number_format_locale;
  const number_format_digits = table_el.dataset.number_format_digits
    ? Number(table_el.dataset.number_format_digits)
    : undefined;
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

type TableRow = { el: HTMLElement; sort_keys: { num: number; str: string }[] };

/** Prepare the table rows for sorting.
 */
function table_parse_data(
  table_el: HTMLElement,
  sort_buttons: HTMLElement[],
): TableRow[] {
  const is_num = [...sort_buttons].map(
    (btn_el) => btn_el.parentElement?.dataset.column_type === "number",
  );
  const row_els = table_el.querySelectorAll<HTMLElement>("tbody tr");
  return [...row_els].map((tr_el) => {
    const cells = tr_el.getElementsByTagName("td");
    return {
      el: tr_el,
      sort_keys: sort_buttons.map((_btn_el, idx) => {
        const str = cells[idx]?.textContent ?? "";
        const num = is_num[idx] ? Number.parseFloat(str) : Number.NaN;
        return { num, str };
      }),
    };
  });
}

/**
 * Adds event listeners to the sort buttons to sort the table rows.
 */
function setup_sort_behavior(
  sort_buttons: HTMLElement[],
  items: TableRow[],
  item_parent: HTMLElement,
) {
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
  const tables = document.querySelectorAll<HTMLElement>(
    "[data-pre-init=table]",
  );
  for (const r of tables) {
    r.removeAttribute("data-pre-init");
    try {
      setup_table(r);
    } catch (e) {
      console.error(e);
    }
  }
}

// Leaflet is loaded from a CDN by sqlpage_map, so it is a global, not an import.
declare const L: typeof Leaflet;

type MarkerStyle = Leaflet.MarkerOptions &
  Leaflet.PathOptions & {
    /** A GeoJSON feature may size its own icon, overriding the SVG's width. */
    size?: number | string;
  };

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
  function parseCoords(
    coords: string | undefined,
  ): Leaflet.LatLngTuple | undefined {
    if (!coords) return undefined;
    const parsed = coords.split(",", 2).map((c) => Number.parseFloat(c));
    if (parsed.length !== 2 || !parsed.every(Number.isFinite)) {
      console.error(
        `Invalid map coordinates: ${JSON.stringify(coords)}. Expected a "latitude,longitude" pair of numbers.`,
      );
      return undefined;
    }
    return [parsed[0], parsed[1]];
  }
  function onLeafletLoad() {
    is_leaflet_loaded = true;
    const maps = document.querySelectorAll<HTMLElement>("[data-pre-init=map]");
    for (const m of maps) {
      const tile_source = m.dataset.tile_source;
      const maxZoom = Number(m.dataset.max_zoom);
      const attribution = m.dataset.attribution;
      const map = L.map(m, { attributionControl: !!attribution });
      const zoom = Number(m.dataset.zoom);
      const center = parseCoords(m.dataset.center);
      if (tile_source)
        L.tileLayer(tile_source, { attribution, maxZoom }).addTo(map);
      const markers: (Leaflet.Marker | Leaflet.GeoJSON)[] = [];
      for (const marker_elem of m.querySelectorAll<HTMLElement>(".marker")) {
        setTimeout(() => {
          const marker = addMarker(marker_elem, map);
          if (marker) markers.push(marker);
        }, 0);
      }
      setTimeout(() => {
        if (center) map.setView(center, zoom);
        else {
          const bounds = L.latLngBounds([]);
          for (const marker of markers)
            bounds.extend(
              marker instanceof L.Marker
                ? marker.getLatLng()
                : marker.getBounds(),
            );
          if (markers.length > 0) map.fitBounds(bounds);
          else map.setView([51.505, 10], zoom);
          if (!Number.isNaN(zoom)) map.setZoom(zoom);
        }
      }, 100);
      m.removeAttribute("data-pre-init");
      m.querySelector(".spinner-border")?.remove();
    }
  }

  function addMarker(marker_elem: HTMLElement, map: Leaflet.Map) {
    const { color, coords, geojson, link } = marker_elem.dataset;
    const options: MarkerStyle = {
      color,
      title: marker_elem.querySelector("h3")?.textContent?.trim(),
    };
    const marker = coords
      ? createMarker(marker_elem, options)
      : geojson && createGeoJSONMarker(marker_elem, geojson, options);
    if (!marker) return undefined;
    marker.addTo(map);
    if (marker_elem.textContent?.trim()) marker.bindPopup(marker_elem);
    else if (link) {
      marker.on("click", () => {
        window.location.href = link;
      });
    }
    return marker;
  }
  function createMarker(marker_elem: HTMLElement, options: MarkerStyle) {
    const coords = parseCoords(marker_elem.dataset.coords);
    return coords && createMarkerAt(marker_elem, coords, options);
  }
  function createMarkerAt(
    marker_elem: HTMLElement,
    coords: Leaflet.LatLngTuple,
    options: MarkerStyle,
  ) {
    const icon_obj = marker_elem.querySelector<HTMLElement>(".mapicon");
    if (!icon_obj) return L.marker(coords, options);
    const size =
      1.5 *
      +(
        options.size ||
        icon_obj.firstElementChild?.getAttribute("width") ||
        24
      );
    const icon = L.divIcon({
      html: icon_obj,
      className: `border-0 bg-${options.color || "primary"} bg-gradient text-white rounded-circle shadow d-flex justify-content-center align-items-center`,
      iconSize: [size, size],
      iconAnchor: [size / 2, size / 2],
    });
    return L.marker(coords, { ...options, icon });
  }
  function createGeoJSONMarker(
    marker_elem: HTMLElement,
    geojson: string,
    options: MarkerStyle,
  ) {
    if (options.color) {
      options.color = get_tabler_color(options.color) || options.color;
    }
    return L.geoJSON(JSON.parse(geojson), {
      style: (feature) => {
        const properties = feature?.properties;
        if (typeof properties !== "object") return options;
        return { ...options, ...properties };
      },
      pointToLayer: (feature, latlng) =>
        createMarkerAt(marker_elem, [latlng.lat, latlng.lng], {
          ...options,
          ...feature.properties,
        }),
    });
  }
}

function sqlpage_form() {
  const file_inputs = document.querySelectorAll<HTMLInputElement>(
    "input[type=file][data-max-size]",
  );
  for (const input of file_inputs) {
    const max_size = Number(input.dataset.maxSize);
    input.addEventListener("change", () => {
      input.classList.remove("is-invalid");
      input.setCustomValidity("");
      for (const { size } of input.files ?? []) {
        if (size > max_size) {
          input.classList.add("is-invalid");
          return input.setCustomValidity(
            `File size must be less than ${max_size / 1000} kB.`,
          );
        }
      }
    });
  }

  const auto_submit_forms = document.querySelectorAll<HTMLFormElement>(
    "form[data-auto-submit]",
  );
  for (const form of auto_submit_forms) {
    form.addEventListener("change", () => form.submit());
  }
}

function get_tabler_color(name: string) {
  return getComputedStyle(document.documentElement).getPropertyValue(
    `--tblr-${name}`,
  );
}

function load_scripts() {
  const addjs = document.querySelectorAll<HTMLElement>("[data-sqlpage-js]");
  const existing_scripts = new Set(
    [...document.querySelectorAll("script")].map((s) => s.src),
  );
  for (const el of addjs) {
    if (!el.dataset.sqlpageJs) continue;
    const js = new URL(el.dataset.sqlpageJs, window.location.href).href;
    if (existing_scripts.has(js)) continue;
    existing_scripts.add(js);
    const script = document.createElement("script");
    script.src = js;
    document.head.appendChild(script);
  }
}

function normalize_hash(hash: string | undefined) {
  const normalized = hash?.replace(/^#/, "") ?? "";
  try {
    return decodeURIComponent(normalized);
  } catch {
    return normalized;
  }
}

function open_toasts_for_hash(toasts: Iterable<HTMLElement>) {
  const Toast = page_bootstrap().Toast;
  const hash = normalize_hash(window.location.hash);
  if (!hash) return;
  for (const toast of toasts) {
    if (normalize_hash(toast.dataset.toastTrigger) === hash) {
      (Toast.getOrCreateInstance(toast) as ToastWidget).show();
    }
  }
}

function restore_focus_after_toast(toast: HTMLElement, container: HTMLElement) {
  if (!toast.contains(document.activeElement)) return;
  const next_close = container.querySelector<HTMLElement>(
    '.toast.show [data-bs-dismiss="toast"]',
  );
  if (next_close) {
    next_close.focus();
    return;
  }
  const main = document.querySelector("main");
  if (!main) return;
  if (!main.hasAttribute("tabindex")) main.tabIndex = -1;
  main.focus({ preventScroll: true });
}

function sqlpage_toast() {
  const Toast = page_bootstrap().Toast;

  const initialized_toasts: HTMLElement[] = [];
  const toasts = document.querySelectorAll<HTMLElement>(
    '[data-pre-init="toast"]',
  );
  for (const toast of toasts) {
    const source_container = toast.parentElement;
    if (!source_container) continue;
    const position = source_container.dataset.sqlpageToastPosition;
    let container: HTMLElement | null = document.querySelector(
      `.toast-container[data-sqlpage-toast-position="${position}"]:not([data-pre-init])`,
    );
    if (!container) {
      container = source_container;
      container.removeAttribute("data-pre-init");
      document.body.appendChild(container);
    } else {
      container.appendChild(toast);
      source_container.remove();
    }

    toast.removeAttribute("data-pre-init");
    const instance = Toast.getOrCreateInstance(toast) as ToastWidget;
    initialized_toasts.push(toast);
    toast.addEventListener("hidden.bs.toast", () => {
      restore_focus_after_toast(toast, container);
      if (toast.dataset.toastTrigger) {
        if (
          normalize_hash(window.location.hash) ===
          normalize_hash(toast.dataset.toastTrigger)
        ) {
          window.history.replaceState(window.history.state, "", "#");
        }
        return;
      }
      instance.dispose();
      toast.remove();
      if (!container.querySelector(".toast")) {
        container.remove();
      }
    });
    if (!toast.dataset.toastTrigger) {
      instance.show();
    }
  }
  open_toasts_for_hash(initialized_toasts);
}

function sqlpage_modal() {
  // Bootstrap modals use position: fixed and are documented to live as
  // direct children of <body>
  // (https://getbootstrap.com/docs/5.3/components/modal/#how-it-works).
  // SQLPage renders them inside .page, but Tabler 1.5 sets
  // `contain: layout` on .page, which makes fixed positioning relative to
  // .page instead of the viewport. The modal then scrolls with the page
  // content and ends up behind its own backdrop, so its buttons cannot be
  // clicked. Moving modals to <body> keeps them viewport-fixed.
  for (const modal of document.querySelectorAll("body .page .modal")) {
    document.body.appendChild(modal);
  }
}

add_init_fn(sqlpage_table);
add_init_fn(sqlpage_map);
add_init_fn(sqlpage_card);
add_init_fn(sqlpage_form);
add_init_fn(sqlpage_modal);
add_init_fn(load_scripts);
add_init_fn(sqlpage_toast);
window.addEventListener("hashchange", () =>
  open_toasts_for_hash(
    document.querySelectorAll<HTMLElement>("[data-toast-trigger]"),
  ),
);

function init_bootstrap_components(fragment: Element | Document) {
  const bootstrap = page_bootstrap();
  for (const el of fragment.querySelectorAll<HTMLElement>(
    '[data-bs-toggle="tooltip"]',
  )) {
    new bootstrap.Tooltip(el);
  }
  for (const el of fragment.querySelectorAll<HTMLElement>(
    '[data-bs-toggle="popover"]',
  )) {
    new bootstrap.Popover(el);
  }
  for (const el of fragment.querySelectorAll<HTMLElement>(
    '[data-bs-toggle="dropdown"]',
  )) {
    new bootstrap.Dropdown(el);
  }
  for (const el of fragment.querySelectorAll<HTMLElement>(
    '[data-bs-ride="carousel"]',
  )) {
    new bootstrap.Carousel(el);
  }
}

document.addEventListener("fragment-loaded", ({ target }) => {
  if (target instanceof Element || target instanceof Document)
    init_bootstrap_components(target);
});

function open_modal_for_hash() {
  const hash = window.location.hash.substring(1);
  if (!hash) return;
  const modal = document.getElementById(hash);
  if (!modal?.classList.contains("modal")) return;
  const bootstrap_modal = page_bootstrap().Modal.getOrCreateInstance(
    modal,
  ) as ModalWidget;
  bootstrap_modal.show();
  modal.addEventListener(
    "hidden.bs.modal",
    () => {
      window.history.replaceState(null, "", "#");
    },
    { once: true },
  );
}

window.addEventListener("hashchange", open_modal_for_hash);
window.addEventListener("DOMContentLoaded", open_modal_for_hash);

/* !include https://cdn.jsdelivr.net/npm/tom-select@2.4.1/dist/js/tom-select.popular.min.js */

function sqlpage_select_dropdown() {
  for (const s of document.querySelectorAll(
    "[data-pre-init=select-dropdown]",
  )) {
    s.removeAttribute("data-pre-init");
    // See: https://github.com/orchidjs/tom-select/issues/716
    // By default, TomSelect will not retain the focus if s is already focused
    // This is a workaround to fix that
    const is_focused = s === document.activeElement;

    const tom = new TomSelect(s, {
      load: sqlpage_load_options_source(s.dataset.options_source),
      valueField: "value",
      labelField: "label",
      searchField: "label",
      create: s.dataset.create_new,
      maxOptions: null,
      onItemAdd: function () {
        this.setTextboxValue("");
        this.refreshOptions();
      },
    });
    if (is_focused) tom.focus();
    s.form?.addEventListener("reset", async () => {
      // The reset event is fired before the form is reset, so we need to wait for the next event loop
      await new Promise((resolve) => setTimeout(resolve, 0));
      // Sync the options with the new reset value
      tom.sync();
      // Wait for the options to be updated
      await new Promise((resolve) => setTimeout(resolve, 0));
      // "sync" also focuses the input, so we need to blur it to remove the focus
      tom.blur();
      tom.close();
    });
  }
}

function sqlpage_load_options_source(options_source) {
  if (!options_source) return;
  return async (query, callback) => {
    const err = (label) =>
      callback([{ label, value: "" }]);
    const resp = await fetch(
      `${options_source}?search=${encodeURIComponent(query)}`,
    );
    if (!resp.ok) {
      return err(
        `Error loading options from "${options_source}": ${resp.status} ${resp.statusText}`,
      );
    }
    const resp_type = resp.headers.get("content-type");
    if (resp_type !== "application/json") {
      return err(
        `Invalid response type: ${resp_type} from "${options_source}". Make sure to use the 'json' component in the SQL file that generates the options.`,
      );
    }
    const results = await resp.json();
    if (!Array.isArray(results)) {
      return err(
        `Invalid response from "${options_source}". The response must be an array of objects with a 'label' and a 'value' property.`,
      );
    }
    if (results.length === 1 && results[0].error) {
      return err(results[0].error);
    }
    if (results.length > 0) {
      const keys = Object.keys(results[0]);
      if (keys.length !== 2 || !keys.includes("label") || !keys.includes("value")) {
        return err(
          `Invalid response from "${options_source}". The response must be an array of objects with a 'label' and a 'value' property. Got: ${JSON.stringify(results[0])} in the first object instead.`,
        );
      }
    }
    callback(results);
  };
}
add_init_fn(sqlpage_select_dropdown);

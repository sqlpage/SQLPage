/* !include https://cdn.jsdelivr.net/npm/tom-select@2.3.1/dist/js/tom-select.popular.min.js */

function sqlpage_select_dropdown() {
  for (const s of document.querySelectorAll(
    "[data-pre-init=select-dropdown]",
  )) {
    // See: https://github.com/orchidjs/tom-select/issues/716
    // By default, TomSelect will not retain the focus if s is already focused
    // This is a workaround to fix that
    const is_focused = s === document.activeElement;
    const tom = new TomSelect(s, {
      create: s.dataset.create_new,
      maxOptions: null,
    });
    if (is_focused) tom.focus();
    s.form?.addEventListener("reset", () => setTimeout(tom.sync.bind(tom), 0));
  }
}

add_init_fn(sqlpage_select_dropdown);

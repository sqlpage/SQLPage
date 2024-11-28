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

add_init_fn(sqlpage_select_dropdown);

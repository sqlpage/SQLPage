/* !include https://cdn.jsdelivr.net/npm/tom-select@2.3.1/dist/js/tom-select.popular.min.js */

function sqlpage_select_dropdown() {
    for (const s of document.querySelectorAll("[data-pre-init=select-dropdown]")) {
        new TomSelect(s, {
          create: s.dataset.create_new
        });
    }
}

add_init_function(sqlpage_select_dropdown);
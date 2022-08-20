// Tables
for (const r of document.getElementsByClassName("data-list")) {
    new List(r, {
        valueNames: [...r.getElementsByClassName("sort")]
            .map(t => t.dataset.sort),
        indexAsync: true
    });
}

/** A deterministic 30-second edit. Capture renderVideo(frame / 30) at any speed. */
(() => {
  const $ = (id) => document.getElementById(id);
  const clamp = (value) => Math.max(0, Math.min(1, value));
  const phase = (t, start, duration) => clamp((t - start) / duration);
  const ease = (value) => 1 - (1 - clamp(value)) ** 3;
  const smooth = (value) => {
    const v = clamp(value);
    return v * v * (3 - 2 * v);
  };
  const mix = (a, b, progress) => a + (b - a) * progress;
  const full = { x: 84, y: 160, w: 1752, h: 840 };
  const split = { x: 916, y: 180, w: 920, h: 800 };
  const form = { x: 460, y: 160, w: 1000, h: 810 };
  const interpolate = (a, b, progress) => ({
    x: mix(a.x, b.x, progress),
    y: mix(a.y, b.y, progress),
    w: mix(a.w, b.w, progress),
    h: mix(a.h, b.h, progress),
  });
  function show(element, opacity, x = 0, y = 0, scale = 1) {
    element.style.opacity = clamp(opacity);
    element.style.transform = `translate(${x}px,${y}px) scale(${scale})`;
  }
  const declaration = [...$("declaration").children];
  const columns = [...$("columns").children];
  let manualPointer = null;
  window.setPointer = (x, y, click = 0) => {
    manualPointer = { x, y, click };
  };
  window.renderVideo = (t, suppliedPointer) => {
    let box = full;
    if (t >= 4 && t < 11)
      box = interpolate(full, split, ease(phase(t, 4, 0.7)));
    else if (t >= 11 && t < 12)
      box = interpolate(split, full, ease(phase(t, 11, 0.85)));
    else if (t >= 18 && t < 21)
      box = interpolate(full, form, ease(phase(t, 18, 0.45)));
    else if (t >= 21 && t < 24)
      box = interpolate(form, full, ease(phase(t, 21, 0.55)));
    // Tighten the frame around the real filtered row, then reopen for sorting.
    if (t >= 13.2 && t < 14.8)
      box = { ...full, h: mix(840, 500, ease(phase(t, 13.2, 0.45))) };
    else if (t >= 14.8 && t < 15.4)
      box = { ...full, h: mix(500, 840, ease(phase(t, 14.8, 0.6))) };
    const stage = $("app-stage");
    Object.assign(stage.style, {
      left: `${box.x}px`,
      top: `${box.y}px`,
      width: `${box.w}px`,
      height: `${box.h}px`,
      opacity: `${1 - phase(t, 24, 0.5)}`,
      transform: `translateY(${mix(0, 120, ease(phase(t, 24, 0.7)))}px)`,
      borderRadius: `${mix(16, 300, ease(phase(t, 24, 0.7)))}px`,
    });
    const sqlVisible = ease(phase(t, 4, 0.6)) * (1 - phase(t, 10.65, 0.35));
    show($("code-stage"), sqlVisible, mix(-95, 0, ease(phase(t, 4, 0.7))), 0);
    $("code-stage").style.setProperty("--rule", `${ease(phase(t, 4.7, 0.7))}`);
    show($("filename"), sqlVisible, 0, mix(30, 0, ease(phase(t, 4.15, 0.4))));
    const outgoing = 1 - phase(t, 7.8, 0.45);
    declaration.forEach((line, index) => {
      const arrival = ease(phase(t, 4.12 + index * 0.13, 0.45));
      show(
        line,
        arrival * outgoing,
        0,
        mix(45, 0, arrival) - ease(phase(t, 7.8, 0.45)) * 72,
      );
      let focus =
        index < 2 ? 1 - smooth(phase(t, 6.1, 0.4)) : smooth(phase(t, 6.1, 0.4));
      focus *= ease(phase(t, 4.75, 0.4));
      line.style.setProperty("--emphasis", `${focus * 0.8}`);
    });
    columns.forEach((line, index) => {
      const arrival = ease(phase(t, 8.08 + index * 0.13, 0.45));
      show(line, arrival, 0, mix(64, 0, arrival));
      line.style.setProperty(
        "--emphasis",
        `${(index === 0 && t >= 8.7 && t < 10.1) || (index === 1 && t >= 10.1) ? 0.8 : 0}`,
      );
    });
    for (const [name, start, end] of [
      ["name", 8.7, 9.4],
      ["plan", 9.4, 10.1],
      ["seats", 10.1, 11.1],
    ]) {
      const token = $(`column-${name}`);
      const active = t >= start && t < end;
      const reveal = ease(phase(t, start, 0.25));
      token.style.color = active ? "#58cce0" : "#f4f7f8";
      token.style.transform = `translateY(${-6 * Math.sin(Math.PI * phase(t, start, 0.35))}px)`;
      token.style.setProperty("--underline", `${active ? reveal : 0}`);
    }
    const label =
      t < 4
        ? "Your app."
        : t < 11
          ? "Just SQL."
          : t < 14.4
            ? "Search."
            : t < 18
              ? "Sort."
              : t < 21
                ? "Add."
                : "Saved.";
    $("label").textContent = label;
    const labelStarts = [0, 4, 11, 14.4, 18, 21];
    const currentStart = labelStarts.filter((value) => value <= t).at(-1);
    const labelArrival = ease(phase(t, currentStart, 0.35));
    show(
      $("label"),
      labelArrival * (1 - phase(t, 24, 0.35)),
      0,
      mix(20, 0, labelArrival),
    );
    const filesIn = ease(phase(t, 18.3, 0.5));
    const filesOut = 1 - phase(t, 22.4, 0.4);
    show($("file-connector"), filesIn * filesOut, 0, mix(30, 0, filesIn));
    $("form-file").style.color = t < 21 ? "#f4f7f8" : "#9caeb5";
    $("save-file").style.color = t >= 21 ? "#58cce0" : "#9caeb5";
    const pointer = suppliedPointer ?? manualPointer;
    const pointerVisible = t >= 11.5 && t < 23.8 && pointer;
    if (pointerVisible) {
      show(
        $("pointer"),
        1,
        pointer.x,
        pointer.y,
        1 - clamp(pointer.click ?? 0) * 0.12,
      );
      const click = clamp(pointer.click ?? 0);
      show(
        $("click-ring"),
        click * 0.85,
        pointer.x + 5,
        pointer.y + 5,
        mix(1.25, 0.4, click),
      );
    } else {
      show($("pointer"), 0);
      show($("click-ring"), 0);
    }
    const ending = ease(phase(t, 24, 0.55));
    show($("ending"), ending);
    const mascotArrival = ease(phase(t, 24.15, 1.2));
    show(
      $("end-mascot"),
      mascotArrival,
      mix(130, 0, mascotArrival),
      mix(35, 0, mascotArrival),
      mix(0.94, 1, mascotArrival),
    );
    const titleArrival = ease(phase(t, 24.3, 0.75));
    show($("end-title"), titleArrival, 0, mix(60, 0, titleArrival));
    const actionArrival = ease(phase(t, 25.1, 0.6));
    show($("end-action"), actionArrival, 0, mix(35, 0, actionArrival));
    const urlArrival = ease(phase(t, 25.3, 0.6));
    show($("end-url"), urlArrival, 0, mix(24, 0, urlArrival));
  };
  window.renderVideo(0);
})();

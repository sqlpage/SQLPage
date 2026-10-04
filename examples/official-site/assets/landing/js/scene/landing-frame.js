import { portraitFlow } from "./portrait-flow.js";

// Normalized projection of the actual sculpture at DEFAULT_TILT, time zero,
// with the square reference camera. The WebP and live camera share this frame.
const reference = {
  shoulder: { x: 0.3499337213692189, y: 0.41625745331853664 },
  bodyWidth: 0.3936780897606217,
  bounds: {
    left: 0.291365946694466,
    right: 0.6850440364550877,
    top: 0.22428359203511622,
    bottom: 0.8034857349574394,
    bottomX: 0.4723918976663531,
  },
};
const clamp = (x, min, max) => Math.min(max, Math.max(min, x));

function frameAt(width, height, anchor, portrait, bodyWidth = width * 0.48) {
  const compositionWidth = width < 680 ? width : Math.min(width, height * 1.95);
  const compositionLeft = (width - compositionWidth) / 2;
  const desiredWidth = portrait
    ? bodyWidth
    : Math.min(compositionWidth * 0.335, height * 0.59);
  let size = desiredWidth / reference.bodyWidth;
  const align = () => {
    const left = anchor.x - reference.shoulder.x * size;
    const top = anchor.y - reference.shoulder.y * size;
    const bounds = Object.fromEntries(
      Object.entries(reference.bounds).map(([key, value]) => [
        key,
        value * size + (key === "top" || key === "bottom" ? top : left),
      ]),
    );
    return { left, top, size, bounds };
  };
  for (let step = 0; step < 3; step++) {
    const { bounds } = align();
    const ribbonHeight = portrait ? 49 : clamp(height * 0.056, 54, 76);
    const ribbonTop =
      height -
      height * (portrait ? 0.075 : 0.045) -
      ribbonHeight -
      (bounds.bottomX - width * 0.51) *
        Math.tan(((portrait ? 5 : 4) * Math.PI) / 180);
    const floor = ribbonTop - (portrait ? 24 : 14);
    const rightEdge = compositionLeft + compositionWidth * 0.95;
    let fit = 1;
    if (!portrait && bounds.bottom > floor && floor > anchor.y)
      fit = Math.min(fit, (floor - anchor.y) / (bounds.bottom - anchor.y));
    if (bounds.right > rightEdge && rightEdge > anchor.x)
      fit = Math.min(fit, (rightEdge - anchor.x) / (bounds.right - anchor.x));
    if (fit >= 0.999) break;
    size *= fit;
  }
  return align();
}

// This small module runs before Three.js downloads, so even the loading layout
// uses the same measured letter, portrait flow, and projection as the live scene.
export function layoutLandingFrame(mount, anchor, progress = 0) {
  const width = Math.max(1, mount.clientWidth);
  const screenHeight = window.innerHeight;
  const portrait = width < 900 && screenHeight >= width;
  const shortLandscape = !portrait && screenHeight < 560;
  const viewport = mount.parentElement;
  const section = viewport.parentElement;
  const heading = anchor?.closest(".hero-copy")?.querySelector(".title-from");
  const intro = viewport.querySelector(".intro-row");
  const ribbon = viewport.querySelector(".sql-ribbon");
  const footer = viewport.querySelector(".viewport-footer");
  const comment = viewport.querySelector(".engraved-comment");
  const baseHeroTop = viewport.querySelector(".site-header").offsetHeight + 14;
  let height;
  let portraitBodyWidth = width * 0.435;
  let portraitShoulderY;
  const setStage = (value) => {
    section.style.setProperty("--stage-height", `${value}px`);
    section.style.setProperty(
      "--sticky-top",
      `${-Math.max(0, value - screenHeight)}px`,
    );
    height =
      portrait || shortLandscape ? value : Math.max(1, mount.clientHeight);
  };
  const measureLetter = () => {
    if (portrait && heading) {
      const surface = mount.getBoundingClientRect();
      return {
        x:
          width * 0.95 -
          ((reference.bounds.right - reference.shoulder.x) *
            portraitBodyWidth) /
            reference.bodyWidth,
        y:
          portraitShoulderY ??
          heading.getBoundingClientRect().bottom - surface.top + width * 0.19,
      };
    }
    if (!anchor) return { x: width * 0.55, y: height * 0.48 };
    const rect = anchor.getBoundingClientRect();
    const surface = mount.getBoundingClientRect();
    const style = getComputedStyle(anchor);
    const fontSize = parseFloat(style.fontSize);
    const context = document.createElement("canvas").getContext("2d");
    context.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
    const ink = context.measureText("e");
    return {
      x:
        rect.left -
        surface.left +
        progress * 90 +
        (ink.actualBoundingBoxRight || rect.width) -
        fontSize * 0.045,
      y:
        rect.bottom -
        surface.top -
        (ink.fontBoundingBoxDescent ?? fontSize * 0.2) +
        ink.actualBoundingBoxDescent -
        fontSize * 0.025,
    };
  };
  if (portrait)
    section.style.setProperty("--portrait-hero-top", `${baseHeroTop}px`);
  setStage(shortLandscape ? Math.max(screenHeight, 640) : screenHeight);
  if (portrait && intro && heading && ribbon && footer && comment) {
    const headingBottom =
      heading.getBoundingClientRect().bottom -
      mount.getBoundingClientRect().top;
    const commentTop = headingBottom + 18;
    section.style.setProperty("--portrait-comment-top", `${commentTop}px`);
    const introTop = commentTop + comment.offsetHeight + 20;
    const minimumArtBottom =
      headingBottom +
      width * 0.19 +
      ((reference.bounds.bottom - reference.shoulder.y) * portraitBodyWidth) /
        reference.bodyWidth;
    const availableArtBottom =
      screenHeight - ribbon.offsetHeight - footer.offsetHeight - 80;
    const spareHeight = Math.max(0, availableArtBottom - minimumArtBottom);
    // Grow the sculpture into the spare height, keeping its right edge fixed.
    portraitBodyWidth = Math.min(
      width * 0.68,
      portraitBodyWidth + spareHeight * 0.65,
    );
    portraitShoulderY = Math.max(
      headingBottom + width * 0.19,
      availableArtBottom -
        ((reference.bounds.bottom - reference.shoulder.y) * portraitBodyWidth) /
          reference.bodyWidth,
    );
    const pose = frameAt(
      width,
      height,
      measureLetter(),
      true,
      portraitBodyWidth,
    );
    const flow = portraitFlow(
      screenHeight,
      Math.max(pose.bounds.bottom, introTop + intro.offsetHeight),
      ribbon.offsetHeight,
      footer.offsetHeight,
    );
    setStage(flow.stageHeight);
    section.style.setProperty("--portrait-ribbon-top", `${flow.ribbonTop}px`);
    section.style.setProperty("--portrait-footer-top", `${flow.footerTop}px`);
    intro.style.top = `${introTop}px`;
    intro.style.left = `${width * 0.04}px`;
  } else if (heading && intro) {
    const rect = heading.getBoundingClientRect();
    const surface = mount.getBoundingClientRect();
    intro.style.top = `${Math.max(0, Math.min(rect.bottom - surface.top + 32, height * 0.87 - intro.offsetHeight - 20))}px`;
    intro.style.left = `${rect.left - surface.left + progress * 90}px`;
  }
  const frame = {
    ...frameAt(width, height, measureLetter(), portrait, portraitBodyWidth),
    width,
    height,
  };
  const preview = viewport.querySelector(".scene-preview");
  preview.style.width = preview.style.height = `${frame.size}px`;
  preview.style.left = `${frame.left}px`;
  preview.style.top = `${frame.top}px`;
  return frame;
}

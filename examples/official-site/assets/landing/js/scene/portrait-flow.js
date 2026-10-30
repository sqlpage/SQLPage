/** Finish the compact, two-column mobile composition without stretching gaps. */
export function portraitFlow(
  screenHeight,
  contentBottom,
  ribbonHeight,
  footerHeight,
) {
  const ribbonTop = contentBottom + 32;
  const footerTop = ribbonTop + ribbonHeight + 24;
  return {
    stageHeight: Math.max(
      screenHeight,
      Math.ceil(footerTop + footerHeight + 24),
    ),
    ribbonTop,
    footerTop,
  };
}

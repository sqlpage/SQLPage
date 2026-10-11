import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile, stat } from "node:fs/promises";

const brand = "examples/official-site/assets/brand";
for (const [name, width, height] of [
  ["social-og", 1200, 630],
  ["social-github", 1280, 640],
  ["social-x", 1500, 500],
  ["social-youtube", 2560, 1440],
  ["avatar", 512, 512],
  ["apple-touch-icon", 180, 180],
]) {
  const size = execFileSync(
    "identify",
    ["-format", "%wx%h", `${brand}/${name}.png`],
    { encoding: "utf8" },
  );
  assert.equal(size, `${width}x${height}`, name);
}
for (const [file, budget, duration, fps] of [
  ["docs/sqlpage.mp4", 8_000_000, 30, "30/1"],
  ["docs/sqlpage.webm", 6_000_000, 30, "30/1"],
  ["docs/sqlpage.gif", 3_000_000, 8, null],
]) {
  assert.ok(
    (await stat(file)).size <= budget,
    `${file} exceeds the media budget`,
  );
  const info = JSON.parse(
    execFileSync(
      "ffprobe",
      [
        "-v",
        "error",
        "-show_entries",
        "stream=width,height,r_frame_rate,codec_type",
        "-show_entries",
        "format=duration",
        "-of",
        "json",
        file,
      ],
      { encoding: "utf8" },
    ),
  );
  assert.ok(
    Math.abs(Number(info.format.duration) - duration) < 0.05,
    `${file} duration`,
  );
  assert.ok(
    info.streams.every((stream) => stream.codec_type !== "audio"),
    "silent video",
  );
  if (fps) {
    assert.equal(info.streams[0].width, 1920);
    assert.equal(info.streams[0].height, 1080);
    assert.equal(info.streams[0].r_frame_rate, fps);
  }
}
for (const variant of ["light", "dark", "mono-light", "mono-dark"]) {
  for (const kind of ["logo-horizontal", "logo-stacked", "symbol"]) {
    const source = await readFile(`${brand}/${kind}-${variant}.svg`, "utf8");
    assert.ok(source.includes("<path"));
    assert.ok(!source.includes("<text"), "outlined lettering");
    await stat(`${brand}/${kind}-${variant}.png`);
  }
}
function luminance(hex) {
  const rgb = hex
    .match(/[0-9a-f]{2}/gi)
    .map((value) => Number.parseInt(value, 16) / 255)
    .map((value) =>
      value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4,
    );
  return rgb[0] * 0.2126 + rgb[1] * 0.7152 + rgb[2] * 0.0722;
}
for (const [foreground, background] of [
  ["f4f7f8", "080e11"],
  ["9caeb5", "080e11"],
  ["9caeb5", "18262d"],
  ["58cce0", "080e11"],
  ["080e11", "58cce0"],
  ["146575", "f4f7f8"],
]) {
  const values = [luminance(foreground), luminance(background)].sort(
    (a, b) => b - a,
  );
  const ratio = (values[0] + 0.05) / (values[1] + 0.05);
  assert.ok(ratio >= 4.5, `#${foreground} on #${background}: ${ratio}`);
  console.log(`#${foreground} on #${background}: ${ratio.toFixed(2)}:1`);
}
console.log(
  "Brand dimensions, outlines, contrast, duration, 1080p/30fps, and media budgets pass.",
);

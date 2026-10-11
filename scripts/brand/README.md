# SQLPage identity assets

The landing sculpture is the reference. The render uses the existing GLB, studio environment, custom materials, jewel edges, shaders, lights, camera, and selective bloom in `examples/official-site/assets/landing/js/scene`. No replacement model or generated illustration is involved.

Outfit is vendored from [Google Fonts](https://github.com/google/fonts/tree/main/ofl/outfit), under the retained SIL Open Font License. `fonts/Outfit.ttf` is the variable source (100–900); `Outfit.woff2` retains its full Latin/extended-Latin character map. Wordmark SVGs contain outlined Outfit 600 glyphs at natural widths with optical spacing. The favicon retains the existing cylinder path and rounded-square proportions, with the sculpture’s dark palette and a crisp cyan band. Companion symbols preserve the broad database and shallow jewel proportions of the existing logo; the legacy raster logo remains a shaded render of the actual sculpture.

## Reproduce assets

Prerequisites: repository npm dependencies, Playwright Chromium, Python with Pillow, fontTools and Brotli, ImageMagick, FFmpeg, and a built SQLPage binary. Run commands from the repository root. The sculpture renderer requires access to the existing Three.js CDN dependencies.

Start the official site in one terminal:

```sh
SQLPAGE_LISTEN_ON=127.0.0.1:8091 target/debug/sqlpage \
  --web-root examples/official-site --config-dir examples/official-site/sqlpage
```

Export the actual model and regenerate vector/raster artwork:

```sh
SQLPAGE_BRAND_URL=http://127.0.0.1:8091 node scripts/brand/export-sculpture.mjs
python scripts/brand/generate-assets.py --render /tmp/sqlpage-brand-render/mascot-transparent.png
```

The exporter filters the surrounding floor and orbit decoration in its isolated capture, without changing the landing renderer. It fixes camera, frame, viewport, and pixel ratio; software WebGL makes the capture repeatable. The generator checks alpha coverage, outlines the variable font at weight 600, draws the symbol and dedicated favicon, and exports every lockup and social size. Source social SVGs reference `mascot-transparent.png` in the same directory, and are editable without an installed font. The YouTube layout uses the central 1546×423 safe strip; the X layout keeps text above the lower-left avatar overlap. Legacy logo and favicon URLs retain their existing character. The site’s authored social preview composition is preserved; the additional platform exports are supplied as separate artwork.

To rebuild WOFF2 from the retained TTF:

```sh
python -c 'from fontTools.ttLib import TTFont; f=TTFont("examples/official-site/assets/brand/fonts/Outfit.ttf"); f.flavor="woff2"; f.save("examples/official-site/assets/brand/fonts/Outfit.woff2")'
```

## Reproduce the finished demonstration

Start an isolated in-memory fixture server in a second terminal:

```sh
SQLPAGE_LISTEN_ON=127.0.0.1:8092 target/debug/sqlpage \
  --web-root tests/end-to-end/fixtures --config-dir tests/end-to-end/fixture-server
```

Then record and encode:

```sh
SQLPAGE_FIXTURE_URL=http://127.0.0.1:8092 node scripts/brand/record-video.mjs
```

The fixture resets only its dedicated customer table, seeds fixed records, and uses real SQLPage table/form rendering and SQLite persistence. Never point the recorder at a production application. The Playwright fixture test imports the shared SQL fixture harness and verifies search, ordering, saving, and persistence after reload.

`video.html` and `video-timeline.js` define the editable 1920×1080 composition and its directed motion. The recorder renders all 900 frames against an explicit 30fps timeline; rendering speed never changes playback timing. Search typing, table sorting, form submission, and reload run against the real iframe. SQL clauses enter and leave with their corresponding result, and the composition expands for interaction. FFmpeg encodes an exact 30-second, silent H.264 MP4 and VP9 WebM, then creates an eight-second 960px GIF and poster. Captures stay in `/tmp/sqlpage-brand-video` by default. `SQLPAGE_VIDEO_TEMP` can change that directory. Output targets are:

- `docs/sqlpage.mp4`, `docs/sqlpage.webm`, `docs/sqlpage.gif` — README media.
- `docs/sqlpage-poster.png` and `docs/sqlpage-poster.webp` — poster.

For a quick framing review without encoding, set `SQLPAGE_VIDEO_REVIEW=1`; it runs the same interactions and saves twelve representative frames. Review them at 640px wide, where a README normally displays the video.

The five scenes occupy 0–4s (customer list), 4–12s (SQL), 12–18s (search/sort), 18–24s (form/save/reload), and 24–30s (sculpture and tutorial invitation). Captions use Outfit; technical filenames and SQL remain monospace. The demonstration belongs to the README; it is not added to the landing page.

## Review and validation

`node scripts/brand/check-assets.mjs` checks dimensions, duration, frame rate, media budgets, SVG structure, and color contrast. `node scripts/brand/review-site.mjs` captures the hero, demonstration, documentation, tutorial, identity guide, small icons, social crops, and video samples under `/tmp/sqlpage-brand-review`.

```sh
npm run format -- --staged
npm test
LOG_LEVEL=info SQLPAGE_BINARY="$(pwd)/target/debug/sqlpage" \
  npm test --workspace tests/end-to-end -- landing.spec.ts landing-demos.spec.ts landing-touch.spec.ts fixtures/brand-demo/test.ts
scripts/test-examples-hurl.sh examples/official-site
```

The frontend formatter is restricted to staged task files. Browser tests cover 390/768/1440px layouts, live examples, keyboard navigation, mouse/touch manipulation, reduced motion, blocked WebGL dependencies, no JavaScript, and font fallback. Uploading social assets or deploying the site is a separate operation.

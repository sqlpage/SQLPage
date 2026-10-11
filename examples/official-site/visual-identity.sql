select 'http_header' as component,
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";
select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', 'Visual Identity - SQLPage',
    'css', '/assets/highlightjs-and-tabler-theme.css',
    'theme', 'dark'
)) as properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, 'SQLPage visual identity' as title, '
SQLPage turns data into useful applications. Its identity pairs the round geometry of **Outfit** with the landing sculpture: three stacked metal sections, two luminous bands, and a detached jewel.

The sculpture is the reference mascot. Use the supplied render or reproduce it from the existing model, materials, lighting, and camera. Do not substitute a generic database illustration.
' as contents_md;
select 'html' as component, '<div style="display:flex;flex-wrap:wrap;align-items:center;gap:32px;margin-block:32px"><img src="/assets/brand/logo-horizontal-light.svg" alt="SQLPage horizontal wordmark" width="320" style="max-width:100%"><img src="/assets/brand/mascot-dark.webp" alt="The reference SQLPage sculpture" width="340" style="max-width:100%"></div>' as html;

select 'text' as component, 'Wordmark and symbol' as title, '
The outlined wordmark uses Outfit 600, with optical spacing and natural letter widths. Use the horizontal lockup in broad placements and the stacked lockup where space is square. Official-site navigation pairs the shaded sculpture artwork with its Outfit wordmark. The full symbol follows the broad sculpture silhouette and its detached jewel. The dedicated favicon preserves the familiar broad cylinder and a crisp cyan band at tiny sizes.

Leave at least half the symbol width clear on every side. Minimum horizontal lockup width: 150px; minimum standalone symbol height: 32px. At 16px, use the favicon. Choose the light artwork on metal black or graphite, and dark artwork on porcelain. Monochrome variants are provided for single-color reproduction.

[Horizontal SVG — light](/assets/brand/logo-horizontal-light.svg) · [dark](/assets/brand/logo-horizontal-dark.svg) · [PNG](/assets/brand/logo-horizontal-light.png)

[Stacked SVG — light](/assets/brand/logo-stacked-light.svg) · [dark](/assets/brand/logo-stacked-dark.svg) · [PNG](/assets/brand/logo-stacked-light.png)

[Symbol SVG — light](/assets/brand/symbol-light.svg) · [dark](/assets/brand/symbol-dark.svg) · [monochrome light](/assets/brand/symbol-mono-light.svg) · [monochrome dark](/assets/brand/symbol-mono-dark.svg)

[Monochrome horizontal — light](/assets/brand/logo-horizontal-mono-light.svg) · [dark](/assets/brand/logo-horizontal-mono-dark.svg)

[Favicon SVG](/assets/brand/favicon.svg) · [ICO](/favicon.ico) · [Apple touch icon](/assets/brand/apple-touch-icon.png)
' as contents_md;

select 'text' as component, 'Six core colors' as title;
select 'color_swatch' as component;
select 'Metal black' as name, '#080E11' as hex, 'Primary dark background and text on light surfaces' as description;
select 'Graphite surface' as name, '#18262D' as hex, 'Code panels and secondary dark surfaces' as description;
select 'Porcelain' as name, '#F4F7F8' as hex, 'Reading surfaces and text on dark backgrounds' as description;
select 'Cyan interaction accent' as name, '#58CCE0' as hex, 'Links on dark backgrounds; buttons with metal black text' as description;
select 'Links on light backgrounds' as name, '#146575' as hex, 'Readable teal links on porcelain' as description;
select 'Muted steel text' as name, '#9CAEB5' as hex, 'Supporting text on dark surfaces, never body text on porcelain' as description;

select 'text' as component, 'Typography' as title, '
**Outfit** is self-hosted, with Latin and extended-Latin coverage and variable weights 100–900. The fallback is `system-ui, sans-serif`. Keep SQL, filenames, and technical values in monospace.

Brand headlines: 600, normal width, tracking −0.025em, line height 1.05. The homepage retains its authored oversized SQL headline and responsive composition. Reading pages keep their established, quieter scale.

Brand copy: 400, 18px/1.6, about 68 characters per line. Use Outfit for site copy and controls while preserving the existing homepage layout, metal finishes, and navigation.

[Font WOFF2](/assets/brand/fonts/Outfit.woff2) · [Source TTF](/assets/brand/fonts/Outfit.ttf) · [SIL Open Font License](/assets/brand/fonts/OFL.txt) · [Google Fonts source](https://github.com/google/fonts/tree/main/ofl/outfit)
' as contents_md;
select 'typography_sample' as component;
select 'Headlines' as title, 'Outfit, system-ui, sans-serif' as font_family, '48px' as font_size,
    '600' as font_weight, '1.05' as line_height, '#F4F7F8' as text_color,
    '-0.025em' as letter_spacing, 'Turn your data into an app.' as sample_text,
    'Brand captions and headings; preserve the homepage’s existing weight and scale' as usage;
select 'typography_sample' as component;
select 'Body' as title, 'Outfit, system-ui, sans-serif' as font_family, '18px' as font_size,
    '400' as font_weight, '1.6' as line_height, '#F4F7F8' as text_color,
    'normal' as letter_spacing, 'Tables, forms and charts, written in SQL.' as sample_text,
    'Descriptions, tutorials, and documentation' as usage;

select 'text' as component, 'Composition and motion' as title, '
The homepage pairs its oversized SQL composition with the actual sculpture. Its tuned opening flight, scroll rotations, layered waves, and section transitions form the existing motion language. Keep their timing and layout together.

Allow direct sculpture manipulation and preserve keyboard and touch controls. Respect reduced motion and retain the static fallback when WebGL is unavailable. Reading pages stay still and quieter than the homepage. Component previews use actual SQLPage rendering.
' as contents_md;

select 'text' as component, 'Mascot and social artwork' as title, '
[Transparent PNG](/assets/brand/mascot-transparent.png) · [Transparent WebP](/assets/brand/mascot-transparent.webp) · [Dark composition](/assets/brand/mascot-dark.webp) · [Light composition](/assets/brand/mascot-light.webp)

Social artwork keeps the jewel and both bands visible alongside the wordmark and “Turn your data into an app.” Platform crops have different layouts; use their dedicated files.

[Open Graph — 1200×630](/assets/brand/social-og.png) · [editable SVG](/assets/brand/social-og.svg)

[GitHub — 1280×640](/assets/brand/social-github.png) · [editable SVG](/assets/brand/social-github.svg)

[X — 1500×500](/assets/brand/social-x.png) · [editable SVG](/assets/brand/social-x.svg)

[YouTube — 2560×1440](/assets/brand/social-youtube.png) · [editable SVG](/assets/brand/social-youtube.svg)

[Square avatar — 512×512](/assets/brand/avatar.png)
' as contents_md;
select 'text' as component, 'Demonstration and reproducible sources' as title, '
The silent 30-second README demo records an isolated SQLPage customer application with real search, sorting, forms, and database persistence. The repository README contains its media.

[README video MP4](https://github.com/sqlpage/SQLPage/blob/main/docs/sqlpage.mp4) · [WebM](https://github.com/sqlpage/SQLPage/blob/main/docs/sqlpage.webm) · [Poster](https://github.com/sqlpage/SQLPage/blob/main/docs/sqlpage-poster.png)

The repository includes the original model, sculpture export script, outlined vector generator, editable social compositions, Playwright recorder, and FFmpeg encoding commands in [scripts/brand](https://github.com/sqlpage/SQLPage/tree/main/scripts/brand). Outfit retains its font license.
' as contents_md;

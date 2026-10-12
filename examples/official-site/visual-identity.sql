select 'http_header' as component,
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";
select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', 'Visual Identity - SQLPage',
    'css', '/assets/highlightjs-and-tabler-theme.css',
    'theme', 'dark'
)) as properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, 'SQLPage visual identity' as title, '
SQLPage turns data into useful applications. Its identity pairs the round geometry of **Outfit** with the landing sculpture: three stacked metal sections, two luminous bands, and a detached jewel.

The landing sculpture is the reference mascot. Use its existing model, faceted jewel, metal stack, and luminous bands for all 3D artwork. The navigation icon is a static render of this same model, with studio fill lighting to keep its cylinder readable at small sizes.
' as contents_md;
select 'html' as component, '<div style="display:flex;flex-wrap:wrap;align-items:center;gap:32px;margin-block:32px"><img src="/assets/brand/logo-horizontal-light.svg" alt="SQLPage horizontal wordmark" width="320" style="max-width:100%"><img src="/assets/icon.webp" alt="The reference SQLPage sculpture" width="340" style="max-width:100%"></div>' as html;

select 'text' as component, 'Wordmark and symbol' as title, '
The outlined wordmark uses Outfit 600, with optical spacing and natural letter widths. Official-site navigation pairs the landing sculpture artwork with its Outfit wordmark. The dedicated favicon preserves the familiar broad cylinder and a crisp cyan band at tiny sizes.

Leave at least half the symbol width clear on every side. Minimum horizontal lockup width: 150px. At 16px, use the favicon. Choose the light artwork on metal black or graphite, and dark artwork on porcelain.

[Horizontal SVG — light](/assets/brand/logo-horizontal-light.svg) · [dark](/assets/brand/logo-horizontal-dark.svg)

[Favicon ICO](/favicon.ico) · [Shaded sculpture icon](/assets/icon.webp)
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

[Font WOFF2](/assets/brand/fonts/Outfit.woff2) · [SIL Open Font License](/assets/brand/fonts/OFL.txt) · [Google Fonts source](https://github.com/google/fonts/tree/main/ofl/outfit)
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
Use the static landing sculpture render for small static placements. The navigation icon and social preview use the same model as the interactive homepage. Keep its faceted jewel, tilted metal stack, and luminous bands together.

[Shaded sculpture icon](/assets/icon.webp) · [Social preview](/sqlpage_social_preview.webp)
' as contents_md;
select 'text' as component, 'README demonstration' as title, '
The silent 30-second README demo shows a SQLPage customer application with search, sorting, forms, and database persistence.

[README video MP4](/sqlpage_introduction_video.mp4) · [Animated preview](https://github.com/sqlpage/SQLPage/blob/main/docs/sqlpage.gif)

The horizontal SVGs and licensed WOFF2 font in the official-site assets are the canonical identity files, shared by the site and repository README.
' as contents_md;

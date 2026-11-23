# Official SQLPage site

The SQLPage website is of course built with SQLPage !

Things that you may be interested to look at:
 - The [custom component we use for our stylish home page](./sqlpage/templates/landing-page.handlebars)
 - The [component reference](./content/components/) and its live examples
 - The [advanced multistep form example](./examples/multistep-form/)

It is hosted as a simple Docker container on [sql-page.com](https://sql-page.com).

Feel free to [open a pull request](https://github.com/lovasoa/sqlpage/pulls) if you would like to add or change anything !

## Preview and deployment

Run `npm ci --ignore-scripts` from the repository root, then `npm run site`.
The preview uses the same build as CI and deployment. Rerun the command after editing the site.

After `npm run build`, run `npm run site:build` to prepare the site under
`target/official-site/`.
Run SQLPage from that directory, or run
`docker compose -f examples/official-site/docker-compose.yml up --build` from
the repository root.

## Editing documentation

- Edit [component](./content/components/form.md) and [function](./content/functions/link.md) descriptions in Markdown. The filename is the public name. Put the summary, category, and parameters in the YAML header.
- Add [component examples](./content/components/form/examples/) beside the component page. The numbered filename determines display order. The Markdown body explains the example. When supplied, the `rows` in the header provide both the rendered preview and the displayed SQL.
- Edit [blog posts](./content/blog/) in Markdown and the [guides](./content/guides.yaml) and [example project cards](./content/projects.yaml) in YAML.

The site build compiles these sources into a startup migration. The checked-in SQL migrations supply the site’s demos and installation options.

## Landing page

The home page is rendered by `index.sql` and the `landing-page` custom template.
Its styles, ES modules, model, lighting map, and static preview are under
`assets/landing/`; no separate HTML entry point or frontend build is required.
Internal navigation uses site-relative URLs so it also works in local previews.

A small nonce-protected marker in the template reserves the animated opening
before first paint; it does not run when JavaScript is disabled. The static
preview appears once its page anchor is measured, and WebGL readiness changes
only the renderer, so slow or failed 3D loading does not shift the page.

The landing page runs as native ES modules. `main.js` starts independent
navigation, selectors, streaming timeline, section transitions, waves, and scene
controllers; each returns its cleanup function. `sections.js` shares keyboard tab
behavior through `tabs.js` and delegates iframe/source switching to
`component-demos.js`. Each live preview uses the normal SQLPage shell and built-in
components in `landing-demos/`, with `demo.css` sizing the rendered content.

Demo customer, ticket, feedback and profile defaults come from
`78_landing_demos.sql`. The source endpoint reads the same SQL files from a fixed
allowlist, never an arbitrary request path. Profile submissions are validated on
the server and rendered in that POST response only. They do not change the shared
database, create visitor storage, or put submitted values in URLs. A new GET of
the form restores its seeded defaults.

`section-motion.js` measures authored content before applying sticky positioning
and element entrance/outro progress. A short section holds once it fits; a long
section scrolls through its full content before its tail holds beneath the next
section's wave. `wave-motion.js` combines slow idle waves with scroll-driven wind
impulses that decay after scrolling stops. Visual poses and responsive layout
remain in the CSS modules rather than the JavaScript controllers.

`experience.js` isolates the optional WebGL lifecycle. `scroll-progress.js` owns
page geometry and scroll state; `scene/database-scene.js` owns rendering and
interaction. One shared canvas flies from the opening composition to the demo,
then stays at each section's `data-scene-stop` page anchor while rotating. Later
handoffs fade between anchors without interpolating position. Anchor measurements
refresh after content and viewport changes, including iframe resizing.

Scene modules use pinned Three.js 0.186.1 ES modules from jsDelivr. Keep their core
module URLs identical when upgrading. Shaders are ordinary `.glsl` files in
`js/scene/shaders/`, fetched relative to `shader-sources.js`; successful loads are
shared, while failed loads can be retried. The model, lighting map, and static
preview are versioned local assets. The official site's `sqlpage.yaml` allows
jsDelivr scripts while preserving SQLPage's per-request nonce.

The studio is a textured horizontal floor with a 512 × 512 planar reflection,
filtered to 256 × 256 with the same Gaussian shader as the emission halo.
Reflection RGB and coverage are blurred together before tone mapping. Its
contact point follows the actual metal surface as the sculpture rotates.
Materials stay in `sculpture-materials.js`, lighting in `database-scene.js`,
and floor shading in the `studio-*.glsl` snippets. After changing the scene,
run the official site locally, then regenerate both fallback assets with
`node scripts/render-landing-sculpture.mjs http://127.0.0.1:8080` from the
repository root. This uses the production renderer at time zero and all 64
turns; it requires Playwright Chromium and ImageMagick with WebP/AVIF support.
The low-power handoff retains the last live frame while its atlas decodes and
attaches the replacement only after drawing it, without showing the preview again.

Without JavaScript the document retains its content, initial live example, and
navigation. CDN or WebGL failure leaves the static sculpture preview and working
controls, and a retry is available for the optional scene. Reduced motion disables
the pinned opening, covering transitions, and idle animations. SQL source uses a
pinned SQL-only Highlight.js grammar with readable plain text if its CDN fails.

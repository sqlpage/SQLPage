# Official SQLPage site

The SQLPage website is of course built with SQLPage !

Things that you may be interested to look at:
 - The [custom component we use for our stylish home page](./sqlpage/templates/landing-page.handlebars)
 - The [migrations](./sqlpage/migrations) that populate the database with the documentation for all components
 - The [advanced multistep form example](./examples/multistep-form/)

It is hosted as a simple Docker container on [sql-page.com](https://sql-page.com).

Feel free to [open a pull request](https://github.com/lovasoa/sqlpage/pulls) if you would like to add or change anything !

The home page is rendered by `index.sql` and the `landing-page` custom template.
Its styles, ES modules, model, lighting map, and static preview are under
`assets/landing/`; no separate HTML entry point or frontend build is required.
Internal navigation uses site-relative URLs so it also works in local previews.

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

Without JavaScript the document retains its content, initial live example, and
navigation. CDN or WebGL failure leaves the static sculpture preview and working
controls, and a retry is available for the optional scene. Reduced motion disables
the pinned opening, covering transitions, and idle animations. SQL source uses a
pinned SQL-only Highlight.js grammar with readable plain text if its CDN fails.

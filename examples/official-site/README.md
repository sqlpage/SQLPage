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

The optional 3D scene loads pinned Three.js 0.186.1 ES modules from jsDelivr.
`index.sql` allows that CDN in the home page's script policy; other pages keep
SQLPage's default policy. Use the same canonical core module URL in every scene
module when upgrading Three.js. The static preview and ordinary navigation work
without WebGL or CDN access, and animation respects reduced-motion preferences.

The sections live in `landing-page.handlebars` with `sections.css` and a small
`sections.js` controller for keyboard-accessible component, stack and deployment
selectors. Component previews use the normal SQLPage shell and built-in components
in `landing-demos/`, styled by `demo.css`. The source endpoint reads those same
SQL files from a fixed allowlist; it never accepts arbitrary file paths. The form
submits to SQLPage for a request-local preview without changing the site database.

One shared canvas follows the `data-scene-stop` rectangles in the sections.
`scroll-progress.js` measures them after responsive layout changes and interpolates
the reference frame; the original GLB, shaders and lighting are unchanged.

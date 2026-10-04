# Official SQLPage site

The SQLPage website is of course built with SQLPage !

Things that you may be interested to look at:
 - The [custom component we use for our stylish home page](./sqlpage/templates/shell-home.handlebars)
 - The [migrations](./sqlpage/migrations) that populate the database with the documentation for all components
 - The [advanced multistep form example](./examples/multistep-form/)

It is hosted as a simple Docker container on [sql-page.com](https://sql-page.com).

Feel free to [open a pull request](https://github.com/lovasoa/sqlpage/pulls) if you would like to add or change anything !

The home page is rendered by `index.sql` and the `shell-home` custom template.
Its styles, ES modules, icons, model, lighting map, and static preview are under
`assets/landing/`; no separate HTML entry point or frontend build is required.
Internal navigation uses site-relative URLs so it also works in local previews.

The optional 3D scene loads pinned Three.js 0.186.1 ES modules from jsDelivr.
`index.sql` allows that CDN in the home page's script policy; other pages keep
SQLPage's default policy. Use the same canonical core module URL in every scene
module when upgrading Three.js. The static preview and ordinary navigation work
without WebGL or CDN access, and animation respects reduced-motion preferences.

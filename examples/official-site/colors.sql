set theme = coalesce($theme, 'custom');

select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', $theme || ' SQLPage Colors',
    'css', case $theme when 'custom' then '/assets/highlightjs-and-tabler-theme.css' end,
    'theme', case $theme when 'default' then 'light' else 'dark' end
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

select 'tab' as component;
select 'Default theme' as title, '?theme=default' as link, 'Default theme' as description, case $theme when 'default' then 'primary' end as color, $theme = 'default' as disabled;
select 'Custom theme' as title, '?theme=custom' as link, 'Custom theme' as description, case $theme when 'custom' then 'primary' end as color, $theme = 'custom' as disabled;


select 'title' as component, 'Colors' as contents, 2 as level;
select 'color_swatch' as component;
select column1 as name, 'tblr-' || column1 as css_var from (values
    ('blue'), ('azure'), ('indigo'), ('purple'), ('pink'), ('red'), ('orange'), ('yellow'), ('lime'), ('green'), ('teal'), ('cyan'),
    ('blue-lt'), ('azure-lt'), ('indigo-lt'), ('purple-lt'), ('pink-lt'), ('red-lt'), ('orange-lt'), ('yellow-lt'), ('lime-lt'), ('green-lt'), ('teal-lt'), ('cyan-lt'),
    ('gray-50'), ('gray-100'), ('gray-200'), ('gray-300'), ('gray-400'), ('gray-500'), ('gray-600'), ('gray-700'), ('gray-800'), ('gray-900'),
    ('facebook'), ('twitter'), ('linkedin'), ('google'), ('youtube'), ('vimeo'), ('dribbble'), ('github'), ('instagram'), ('pinterest'), ('vk'), ('rss'), ('flickr'), ('bitbucket'), ('tabler'),
    ('black'), ('white'), ('gray'), ('primary'), ('secondary'), ('success'), ('info'), ('warning'), ('danger'), ('light'), ('dark')
);

select 'text' as component, '
The colors above are from the [official site custom theme](https://github.com/sqlpage/SQLPage/blob/main/examples/official-site/assets/brand/brand.css).
View [this page with the default theme](?theme=default) to see the colors that are used by default.
' as contents_md where $theme = 'custom';

select 'text' as component, '
### Customization and theming

SQLPage is designed to be easily customizable and themable.
You cannot pass arbitrary color codes to components from your SQL queries,
but you can customize which exact color is associated to each color name.
When you change a fill color, set its matching `-fg` variable to keep text readable.

#### Creating a custom theme

To create a custom theme, you can create a CSS file and use the [shell component](/component.sql?component=shell) to include it.

##### `index.sql`

```sql
select ''shell'' as component, ''custom_theme.css'' as css, ''custom_theme'' as theme;
```

##### `custom_theme.css`

```css
:root,
.layout-boxed[data-bs-theme="custom_theme"] {
  color-scheme: dark;

  /* Base text colors */
  --tblr-body-color: #cfd5e6;
  --tblr-text-secondary-rgb: 204, 209, 217;
  --tblr-secondary-color: #cccccc;
  --tblr-muted-color: rgba(191, 191, 191, 0.8);

  /* Background colors */
  --tblr-body-bg: #0f1426;
  --tblr-bg-surface: #111629;
  --tblr-bg-surface-secondary: #151a2e;
  --tblr-bg-surface-tertiary: #191f33;

  /* Primary and secondary colors */
  --tblr-primary-rgb: 139, 184, 232;
  --tblr-primary: rgb(var(--tblr-primary-rgb));
  --tblr-primary-fg: var(--tblr-body-bg);
  --tblr-secondary-rgb: 235, 232, 255;
  --tblr-secondary: rgb(var(--tblr-secondary-rgb));

  /* Border colors */
  --tblr-border-color: #151926;
  --tblr-border-color-translucent: #404d73b3;

  /* Theme colors. All sqlpage colors can be customized in the same way. */
  --tblr-blue-rgb: 84, 151, 213; /* To convert between #RRGGBB color codes to decimal RGB values, you can use https://www.rapidtables.com/web/color/RGB_Color.html */
  --tblr-blue: rgb(var(--tblr-blue-rgb));

  --tblr-red-rgb: 229, 62, 62;
  --tblr-red: rgb(var(--tblr-red-rgb));

  --tblr-green-rgb: 72, 187, 120;
  --tblr-green: rgb(var(--tblr-green-rgb));
}

/* Pagination has its own active label color in Tabler. */
.pagination {
  --tblr-pagination-active-color: var(--tblr-primary-fg);
}
```
' as contents_md;


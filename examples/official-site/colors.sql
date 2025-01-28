set theme = coalesce($theme, 'custom');

select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', $theme || ' SQLPage Colors',
    'css', case $theme when 'custom' then '/assets/highlightjs-and-tabler-theme.css' end,
    'theme', case $theme when 'default' then 'light' else 'dark' end
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

create temporary table if not exists colors as select column1 as color, column2 as hex from (values
    ('blue', '#0054a6'), ('azure', '#4299e1'), ('indigo', '#4263eb'), ('purple', '#ae3ec9'), ('pink', '#d6336c'), ('red', '#d63939'), ('orange', '#f76707'), ('yellow', '#f59f00'), ('lime', '#74b816'), ('green', '#2fb344'), ('teal', '#0ca678'), ('cyan', '#17a2b8'),
    ('blue-lt', '#e9f0f9'), ('azure-lt', '#ecf5fc'), ('indigo-lt', '#eceffd'), ('purple-lt', '#f7ecfa'), ('pink-lt', '#fbebf0'), ('red-lt', '#fbebeb'), ('orange-lt', '#fef0e6'), ('yellow-lt', '#fef5e6'), ('lime-lt', '#f1f8e8'), ('green-lt', '#eaf7ec'), ('teal-lt', '#e7f6f2'), ('cyan-lt', '#e8f6f8'),
    ('gray-50', '#f8fafc'), ('gray-100', '#f1f5f9'), ('gray-200', '#e2e8f0'), ('gray-300', '#c8d3e1'), ('gray-400', '#9ba9be'), ('gray-500', '#6c7a91'), ('gray-600', '#49566c'), ('gray-700', '#313c52'), ('gray-800', '#1d273b'), ('gray-900', '#0f172a'),
    ('facebook', '#1877F2'), ('twitter', '#1da1f2'), ('linkedin', '#0a66c2'), ('google', '#dc4e41'), ('youtube', '#ff0000'), ('vimeo', '#1ab7ea'), ('dribbble', '#ea4c89'), ('github', '#181717'), ('instagram', '#e4405f'), ('pinterest', '#bd081c'), ('vk', '#6383a8'), ('rss', '#ffa500'), ('flickr', '#0063dc'), ('bitbucket', '#0052cc'), ('tabler', '#0054a6'),
    ('black', '#000000'), ('white', '#ffffff'), ('gray', '#808080'),
    ('primary', '#0054a6'), ('secondary', '#49566c'), ('success', '#2fb344'), ('info', '#17a2b8'), ('warning', '#f59f00'), ('danger', '#d63939'), ('light', '#f1f5f9'), ('dark', '#0f172a')
);

select 'tab' as component;
select 'Default theme' as title, '?theme=default' as link, 'Default theme' as description, case $theme when 'default' then 'primary' end as color, $theme = 'default' as disabled;
select 'Custom theme' as title, '?theme=custom' as link, 'Custom theme' as description, case $theme when 'custom' then 'primary' end as color, $theme = 'custom' as disabled;


select 'card' as component, 'Colors' as title;
select color as title, hex as description, color as background_color
from colors;


select 'text' as component, '
The colors above are from the [official site custom theme](https://github.com/sqlpage/SQLPage/blob/main/examples/official-site/assets/highlightjs-and-tabler-theme.css).
View [this page with the default theme](?theme=default) to see the colors that are used by default.
' as contents_md where $theme = 'custom';

select 'text' as component, '
### Customization and theming

SQLPage is designed to be easily customizable and themable.
You cannot pass arbitrary color codes to components from your SQL queries,
but you can customize which exact color is associated to each color name.

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
  color-scheme: light;

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
  --tblr-primary-rgb: 95, 132, 169;
  --tblr-primary: rgb(var(--tblr-primary-rgb));
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
```
' as contents_md;


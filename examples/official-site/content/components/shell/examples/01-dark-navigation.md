---
rows:
  - component: shell
    title: 'SQLPage: SQL websites'
    navbar_title: SQLPage
    link: /
    menu_item:
      - title: About
        submenu:
          - link: /safety.sql
            title: Security
            icon: lock
          - link: /performance.sql
            title: Performance
            icon: bolt
          - link: //github.com/sqlpage/SQLPage/blob/main/LICENSE.txt
            title: License
            icon: file-text
          - link: /blog.sql
            title: Articles
            icon: book
      - title: Examples
        submenu:
          - link: /examples/tabs/
            title: Tabs
            icon: layout-navbar
          - link: /examples/layouts.sql
            title: Layouts
            icon: layout
          - link: /examples/multistep-form
            title: Forms
            icon: edit
          - link: /examples/handle_picture_upload.sql
            title: File uploads
            icon: upload
          - link: /examples/authentication/
            title: Password protection
            icon: password-user
          - link: //github.com/sqlpage/SQLPage/blob/main/examples/
            title: All examples & demos
            icon: code
      - title: Community
        submenu:
          - link: /blog.sql
            title: Blog
            icon: book
          - link: //github.com/sqlpage/SQLPage/issues
            title: Report a bug
            icon: bug
          - link: //github.com/sqlpage/SQLPage/discussions
            title: Discussions
            icon: message
          - link: //github.com/sqlpage/SQLPage
            title: Github
            icon: brand-github
      - title: Documentation
        submenu:
          - link: /your-first-sql-website
            title: Getting started
            icon: book
          - link: /components.sql
            title: All Components
            icon: list-details
          - link: /functions.sql
            title: SQLPage Functions
            icon: math-function
          - link: /extensions-to-sql
            title: Extensions to SQL
            icon: cube-plus
          - link: /custom_components.sql
            title: Custom Components
            icon: puzzle
          - link: https://sql-page.com/configuration
            title: Configuration
            icon: settings
      - title: Search
        link: /search
    layout: boxed
    language: en-US
    description: Go from SQL queries to web applications in an instant.
    preview_image: https://sql-page.com/sqlpage_social_preview.webp
    theme: dark
    font_size: 19
    image: /assets/icon.webp
    favicon: /favicon.ico
    javascript_module: /assets/highlightjs-launch.js
    css: /assets/highlightjs-and-tabler-theme.css
    footer: '[Built with SQLPage](https://github.com/sqlpage/SQLPage/tree/main/examples/official-site)'
---

This example configures a dark shell with menus linking to pages on this site.

Each `menu_item` object defines a link or a dropdown. The "Community" menu links to the blog, issue tracker, discussions, and source code.

The object form can be used directly only on database engines that have a native JSON type.
On other engines (such as SQLite), you can use the [`dynamic`](?component=dynamic#component) component to generate the same result.


You see the [page layouts demo](./examples/layouts.sql) for a live example of the different layouts.

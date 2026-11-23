---
rows:
  - component: list
    wrap: true
  - title: SQL Websites
    image_url: /favicon.ico
    description_md: Write SQL, get a website. SQLPage is a **SQL**-based **site** generator for **PostgreSQL**,
      **MySQL**, **SQLite** and **SQL Server**.
    link: /
  - title: SQL Forms
    image_url: https://upload.wikimedia.org/wikipedia/commons/b/b6/FileStack_retouched.jpg
    description_md: Easily collect data **from users to your database** using the *form* component. Handle
      the data in SQL with `INSERT` or `UPDATE` queries.
    link: ?component=form
  - title: SQL Maps
    image_url: https://upload.wikimedia.org/wikipedia/commons/1/15/Vatican_City_map_EN.png
    description_md: Show database contents on a map using the *map* component. Works well with *PostGIS* and
      *SpatiaLite*.
    link: ?component=map
  - title: Advanced features
    icon: settings
    description_md: '[Authenticate users](?component=authentication), [edit data](?component=form), [generate
      an API](?component=json), [maintain your database schema](/your-first-sql-website/migrations.sql), and
      more.'
---

### A list with rich text descriptions

This example illustrates creating a nice list where each item has a title, a description, an image, and a link to another page.

> Be careful, nested links are not supported. If you use the list's `link` property, then your markdown `description_md` should not contain any link.

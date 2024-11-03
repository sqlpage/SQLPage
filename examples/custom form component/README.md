# Custom form component

This example shows how to create a simple custom component in handlebars, and call it from SQL.

It uses MySQL, but it should be easy to adapt to other databases.
The only MySQL-specific features used here are 
 - `json_table`, which is supported by MariaDB and MySQL 8.0 and later.
 - MySQL's `json_merge` function.
Both [have analogs in other databases](https://sql.datapage.app/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide).

![screenshot](screenshot.png)


## Key features illustrated in this example

- How to create a custom component in handlebars, with dynamic behavior implemented in JavaScript
- How to manage multiple-option select boxes, with pre-selected items, and multiple choices
- Including a common menu between different pages using a `shell.sql` file, the dynamic component, and the `sqlpage.run_sql` function.
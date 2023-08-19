# CHANGELOG.md

## unreleased

 - Added a dark theme ! You can now choose between a light and a dark theme for your SQLPage website.
   Select the dark theme using the `theme` parameter of the `shell` component:
   ```sql
   SELECT 'shell' AS component, 'dark' AS theme;
   ```
   See https://github.com/lovasoa/SQLpage/issues/50
 - Fixed a bug where the default index page would be displayed when `index.sql` could not be loaded,
   instead of displaying an error page explaining the issue.

## 0.9.5 (2023-08-12)

 - New `tab` component to create tabbed interfaces. See [the documentation](https://sql.ophir.dev/documentation.sql?component=tab#component).
 - Many improvements in database drivers.
   - performance and numeric precision improvements,
   - multiple fixes around passing NUMERIC, DECIMAL, and JSON values to SQLPage.

## 0.9.4 (2023-08-04)

Small bugfix release

 - Fix a bug with simple queries (ones with only static values) that contained multiple repeated columns
   (such as `SELECT 'hello' AS menu_item, 'world' AS menu_item`). Only the last column would be taken into account.
   This could manifest as a bug where
     - only the last menu item in the shell component would be displayed,
     - only the last markdown column in a table would be interpreted as markdown,
     - only the last icon column in a table would be displayed as an icon.

## 0.9.3 (2023-08-03)

 - Icons are now loaded directly from the sqlpage binary instead of loading them from a CDN.
  This allows pages to load faster, and to get a better score on google's performance audits, potentially improving your position in search results.
    - This also makes it possible to host a SQLPage website on an intranet without access to the internet.
    - Fixes https://github.com/lovasoa/SQLpage/issues/37
 - store compressed frontend assets in the SQLPage binary:
    - smaller SQLPage binary
    - Faster page loads, less work on the server
 - Fix a bug where table search would fail to find a row if the search term contained some special characters.
    - Fixes https://github.com/lovasoa/SQLpage/issues/46
 - Split the charts javascript code from the rest of the frontend code, and load it only when necessary.
   This greatly diminishes the amount of js loaded by default, and achieves very good performance scores by default.
   SQLPage websites now load even faster, een on slow mobile connections.

## 0.9.2 (2023-08-01)

 - Added support for more SQL data types. This notably fixes an issue with the display of datetime columns in tables.
    - See: https://github.com/lovasoa/SQLpage/issues/41
 - Updated dependencies, better SQL drivers

## 0.9.1 (2023-07-30)

 - Fix issues with the new experimental mssql driver.

## 0.9.0 (2023-07-30)

 - Added a new `json` component, which allows building a JSON API entirely in SQL with SQLPage !
   Now creating an api over your database is as simple as `SELECT 'json' AS component, JSON_OBJECT('hello', 'world') AS contents`.
 - `SELECT` statements that contain only static values are now interpreted directly by SQLPage, and do not result in a database query. This greatly improves the performance of pages that contain many static elements.
 - Redirect index pages without a trailing slash to the same page with the trailing slash. This ensures that relative links work correctly, and gives each page a unique canonical URL. (For instance, if you have a file in `myfolder/index.sql`, then it will be accessible at `mysite.com/myfolder/` and `mysite.com/myfolder` will redirect to `mysite.com/myfolder/`).
 - Update the database drivers to the latest version, and switch to a fork of `sqlx`. This also updates the embedded version of SQLite to 3.41.2, with [many cool new features](https://www.sqlite.org/changes.html) such as:
   - better json support
   - better performance
 - Add experimental support for *Microsoft SQL Server*. If you have a SQL Server database lying around, please test it and report any issue you might encounter !

## 0.8.0 (2023-07-17)

 - Added a new [`sqlite_extensions` configuration parameter](./configuration.md) to load SQLite extensions. This allows many interesting use cases, such as 
      - [using spatialite to build a geographic data application](./examples/make%20a%20geographic%20data%20application%20using%20sqlite%20extensions/),
      - querying CSV data from SQLPage with [vsv](https://github.com/nalgeon/sqlean/blob/main/docs/vsv.md),
      - or building a search engine for your data with [FTS5](https://www.sqlite.org/fts5.html).
 - Breaking: change the order of priority for loading configuration parameters: the environment variables have priority over the configuration file. This makes it easier to tweak the configuration of a SQLPage website when deploying it.
 - Fix the default index page in MySQL. Fixes [#23](https://github.com/lovasoa/SQLpage/issues/23).
 - Add a new [map](https://sql.ophir.dev/documentation.sql?component=map#component) component to display a map with markers on it. Useful to display geographic data from PostGIS or Spatialite.
 - Add a new `icon` attribute to the [table](https://sql.ophir.dev/documentation.sql?component=table#component) component to display icons in the table.
 - Fix `textarea` fields in the [form](https://sql.ophir.dev/documentation.sql?component=table#component) component to display the provided `value` attribute. Thanks Frank for the contribution !
 - SQLPage now guarantees that a single web request will be handled by a single database connection. Previously, connections were repeatedly taken and put back to the connection pool between each statement, preventing the use of temporary tables, transactions, and other connection-specific features such as [`last_insert_rowid`](https://www.sqlite.org/lang_corefunc.html#last_insert_rowid). This makes it much easier to keep state between SQL statements in a single `.sql` file. Please report any performance regression you might encounter. See [the many-to-many relationship example](./examples/modeling%20a%20many%20to%20many%20relationship%20with%20a%20form/).
 - The [table](https://sql.ophir.dev/documentation.sql?component=table#component) component now supports setting a custom background color, and a custom CSS class on a given table line.
 - New `checked` attribute for checkboxes and radio buttons. 

## 0.7.2 (2023-07-10)

### [SQL components](https://sql.ophir.dev/documentation.sql)

 - New [authentication](https://sql.ophir.dev/documentation.sql?component=authentication#component) component to handle user authentication, and password checking
 - New [redirect](https://sql.ophir.dev/documentation.sql?component=redirect#component) component to stop rendering the current page and redirect the user to another page.
 - The [debug](https://sql.ophir.dev/documentation.sql?component=debug#component) component is now documented
 - Added properties to the [shell](https://sql.ophir.dev/documentation.sql?component=shell#component) component:
    - `css` to add custom CSS to the page
    - `javascript` to add custom Javascript to the page. An example of [how to use it to integrate a react component](https://github.com/lovasoa/SQLpage/tree/main/examples/using%20react%20and%20other%20custom%20scripts%20and%20styles) is available.
    - `footer` to set a message in the footer of the page

### [SQLPage functions](https://sql.ophir.dev/functions.sql)

 - New [`sqlpage.basic_auth_username`](https://sql.ophir.dev/functions.sql?function=basic_auth_username#function) function to get the name of the user logged in with HTTP basic authentication
 - New [`sqlpage.basic_auth_password`](https://sql.ophir.dev/functions.sql?function=basic_auth_password#function) function to get the password of the user logged in with HTTP basic authentication.
 - New [`sqlpage.hash_password`](https://sql.ophir.dev/functions.sql?function=hash_password#function) function to hash a password with the same algorithm as the [authentication](https://sql.ophir.dev/documentation.sql?component=authentication#component) component uses.
 - New [`sqlpage.header`](https://sql.ophir.dev/functions.sql?function=header#function) function to read an HTTP header from the request.
 - New [`sqlpage.random_string`](https://sql.ophir.dev/functions.sql?function=random_string#function) function to generate a random string. Useful to generate session ids.


### Bug fixes

 - Fix a bug where the page style would not load in pages that were not in the root directory: https://github.com/lovasoa/SQLpage/issues/19
 - Fix resources being served with the wrong content type
 - Fix compilation of SQLPage as an AWS lambda function
 - Fixed logging and display of errors, to make them more useful

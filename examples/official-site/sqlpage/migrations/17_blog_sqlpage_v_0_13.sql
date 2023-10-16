INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES (
        'SQLPage v0.13',
        'SQLPage v0.13 brings easier debugging and a beautiful new timeline component.',
        'refresh',
        '2023-10-16',
        '
# v0.13.0: beautiful timelines and scatter plots

 - New [timeline](https://sql.ophir.dev/documentation.sql?component=timeline#component) component to display a timeline of events.
 - Add support for scatter and bubble plots in the chart component. See [the chart documentation](https://sql.ophir.dev/documentation.sql?component=chart#component).
 - further improve debuggability with more precise error messages. In particular, it used to be hard to debug errors in long migration scripts, because the line number and position was not displayed. This is now fixed.
 - Better logs on 404 errors. SQLPage used to log a message without the path of the file that was not found. This made it hard to debug 404 errors. This is now fixed.
 - Add a new `top_image` attribute to the [card](https://sql.ophir.dev/documentation.sql?component=card#component) component to display an image at the top of the card. This makes it possible to create beautiful image galleries with SQLPage.
 - Updated dependencies, for bug fixes and performance improvements.
 - New icons (see https://tabler-icons.io/changelog)
 - When `NULL` is passed as an icon name, display no icon instead of raising an error.
 - Official docker image folder structure changed. The docker image now expects 
   - the SQLPage website (`.sql` files) to be in `/var/www/`, and
   - the SQLPage configuration folder to be in `/etc/sqlpage/`
    - the configuration file should be in `/etc/sqlpage/sqlpage.json`
    - the database file should be in `/etc/sqlpage/sqlpage.db`
    - custom templates should be in `/etc/sqlpage/templates/`
   - This configuration change concerns only the docker image. If you are using the sqlpage binary directly, nothing changes.

**Full Changelog**: https://github.com/lovasoa/SQLpage/compare/v0.12.0...v0.13.0
');
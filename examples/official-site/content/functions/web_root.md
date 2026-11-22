---
icon: folder-code
since: 0.42.0
category: server-and-runtime
summary: Locate the directory that holds the site's public files.
---

Returns the web root directory where SQLPage serves `.sql` files from.

### Example

```sql
SELECT 'text' AS component;
SELECT 'SQL files are served from ' AS contents;
SELECT sqlpage.web_root() as contents, true as code;
```

#### Result

SQL files are served from `/home/user/my_sqlpage_website`

#### Notes

The web root is the directory from which `.sql` files are loaded and served.
By default, it is the current working directory, but it can be changed using:
 - the `--web-root` command line argument
 - the `web_root` [configuration option](/configuration#web_root) in `sqlpage.json`
 - the `WEB_ROOT` environment variable

This is more reliable than `sqlpage.current_working_directory()` when you need to reference the location of your SQL files.

---
icon: folder-cog
since: 0.42.0
category: server-and-runtime
summary: Locate the directory for SQLPage configuration and templates.
---

Returns the configuration directory where SQLPage looks for `sqlpage.json`, templates, and migrations.

### Example

```sql
SELECT 'text' AS component;
SELECT 'Configuration files are in ' AS contents;
SELECT sqlpage.configuration_directory() as contents, true as code;
```

#### Result

Configuration files are in `/home/user/my_sqlpage_website/sqlpage`

#### Notes

The configuration directory is where SQLPage looks for:
 - `sqlpage.json` (the configuration file)
 - `templates/` (custom component templates)
 - `migrations/` (database migration files)

By default, it is `./sqlpage` relative to the current working directory, but it can be changed using:
 - the `--config-dir` command line argument
 - the `SQLPAGE_CONFIGURATION_DIRECTORY` or `CONFIGURATION_DIRECTORY` environment variable

This function is useful when you need to reference configuration-related files in your SQL code.

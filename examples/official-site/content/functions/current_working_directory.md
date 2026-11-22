---
icon: folder-question
since: 0.11.0
category: server-and-runtime
summary: Get the working directory of the SQLPage server process.
---

Returns the [current working directory](https://en.wikipedia.org/wiki/Working_directory) of the SQLPage server process.

### Example

```sql
SELECT 'text' AS component;
SELECT 'Currently running from ' AS contents;
SELECT sqlpage.current_working_directory() as contents, true as code;
```

#### Result

Currently running from `/home/user/my_sqlpage_website`

#### Notes

The current working directory is the directory from which the SQLPage server process was started.
By default, this is also the directory from which `.sql` files are loaded and served.
However, this can be changed by setting the `web_root` [configuration option](/configuration#web_root).

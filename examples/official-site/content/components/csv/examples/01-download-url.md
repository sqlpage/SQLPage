---
rows:
  - component: button
  - title: Download my data
    link: /examples/csv_download.sql
---

### Header component: creating a CSV download URL

You can create a page that will trigger a download of the CSV file when the user visits it.
The contents will be streamed efficiently from the database to the browser, without being fully loaded in memory.
This makes it possible to download even very large files without overloading the database server, the web server, or the client's browser.

#### `csv_download.sql`

```sql
select 'csv' as component, 'example.csv' as filename;
SELECT * FROM my_large_table;
```

#### `index.sql`

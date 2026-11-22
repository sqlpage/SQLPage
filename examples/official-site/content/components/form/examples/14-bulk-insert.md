---
rows:
  - component: form
    title: CSV import
    validate: Load data
    action: examples/handle_csv_upload.sql
  - name: product_data_input
    type: file
    accept: text/csv
    label: Products
    description: Upload a CSV with a name, description, and price columns
    required: true
---

## Bulk data insertion

You can use the `file` type to allow the user to upload a [CSV](https://en.wikipedia.org/wiki/Comma-separated_values)
file containing data to insert in a table.

SQLPage can load data from a CSV file and insert it into a database table.
SQLPage re-uses PostgreSQL's [`COPY` syntax](https://www.postgresql.org/docs/current/sql-copy.html)
to specify the format of the CSV file, but makes it work with all supported databases.

> When connected to a PostgreSQL database, SQLPage will use the native `COPY` statement,
> for super fast and efficient on-database CSV parsing.
> But it will also work transparently with other databases, by
> parsing the CSV locally and emulating the same behavior with simple `INSERT` statements.

Here is how you could easily copy data from a CSV to a table in the database:

```sql
copy product(name, description) from 'product_data_input'
with (header true, delimiter ',', quote '"');
```

If you want to pre-process the data before inserting it into the final table,
you can use a temporary table to store the data, and then insert it into the final table:

```sql
-- temporarily store the data in a table with text columns
create temporary table if not exists product_tmp(name text, description text, price text);
delete from product_tmp;

-- copy the data from the CSV file into the temporary table
copy product_tmp(name, description, price) from 'product_data_input';

-- insert the data into the final table, converting the price column to an integer
insert into product(name, description, price)
select name, description, CAST(price AS integer) from product_tmp
where price is not null and description is not null and length(description) > 10;
```

This will load the processed CSV into the product table, provided it has the following structure:

```csv
name,description,price
"SQLPage","A tool to create websites using SQL",0
"PostgreSQL","A powerful open-source relational database",0
"SQLite","A lightweight relational database",0
"MySQL","A popular open-source relational database",0
```

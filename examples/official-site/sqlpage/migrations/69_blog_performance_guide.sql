
INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'Performance Guide',
        'Concrete advice on how to make your SQLPage webapp fast',
        'bolt',
        '2025-10-31',
        '
# Performance Guide

SQLPage is [optimized](/performance)
to allow you to create web pages that feel snappy.
This guide contains advice on how to ensure your users never wait
behind a blank screen waiting for your pages to load.

A lot of the advice here is not specific to SQLPage, but applies
to making SQL queries fast in general.
If you are already comfortable with SQL performance optimization, feel free to jump right to
the second part of the quide: *SQLPage-specific advice*.

## Make your queries fast

The best way to ensure your SQLPage webapp is fast is to ensure your
database is well managed and your SQL queries are well written.
We''ll go over the most common database performance pitfalls so that you know how to avoid them.

### Choose the right database schema

#### Normalize (but not too much)

Your database schema should be [normalized](https://en.wikipedia.org/wiki/Database_normalization):
one piece of information should be stored in only one place in the database.
This is a good practice that will not only make your queries faster,
but also make it impossible to store incoherent data.
You should use meaningful natural [primary keys](https://en.wikipedia.org/wiki/Primary_key) for your tables
and resort to surrogate keys (such as auto-incremented integer ids) only when the data is not naturally keyed.
Relationships between tables should be explicitly represented by [foreign keys](https://en.wikipedia.org/wiki/Foreign_key).

```sql
-- Products table, naturally keyed by catalog_number
CREATE TABLE product (
    catalog_number VARCHAR(20) PRIMARY KEY,
    name TEXT NOT NULL,
    price DECIMAL(10,2) NOT NULL
);

-- Sales table: natural key = (sale_date, store_id, transaction_number)
-- composite primary key used since no single natural attribute alone uniquely identifies a sale
CREATE TABLE sale (
    sale_date DATE NOT NULL,
    store_id   VARCHAR(10) NOT NULL,
    transaction_number INT NOT NULL,
    product_catalog_number VARCHAR(20) NOT NULL,
    quantity   INT NOT NULL CHECK (quantity > 0),
    PRIMARY KEY (sale_date, store_id, transaction_number),
    FOREIGN KEY (product_catalog_number) REFERENCES product(catalog_number),
    FOREIGN KEY (store_id) REFERENCES store(store_id)
);
```

Always use foreign keys instead of trying to store redundant data such as store names in the sales table.

This way, when you need to display the list of stores in your application, you don''t have to
run a slow `select distinct store from sales`, that would have to go through your millions of sales
(*even if you have an index on the store column*), you just query the tiny `stores` table directly.

You also need to use the right [data types](https://en.wikipedia.org/wiki/Data_type) for your columns,
otherwise you will waste a lot of space and time converting data at query time.
See [postgreSQL data types](https://www.postgresql.org/docs/current/datatype.html),
[MySQL data types](https://dev.mysql.com/doc/refman/8.0/en/data-types.html),
[Microsoft SQL Server data types](https://learn.microsoft.com/en-us/sql/t-sql/data-types/data-types-transact-sql?view=sql-server-ver16),
[SQLite data types](https://www.sqlite.org/datatype3.html).

[Denormalization](https://en.wikipedia.org/wiki/Denormalization) can be introduced
only after you have already normalized your data, and is often not required at all.

### Use views

Querying normalized views can be cumbersome.
`select store_name, sum(paid_eur) from sale group by store_name`
is more readable than

```sql
select store.name, sum(sale.paid_eur)
from sales
  inner join stores on sale.store_id = store.store_id
group by store_name
```

To work around that, you can create views that contain
useful table joins so that you do not have to duplicate them in all your queries:

```sql
create view enriched_sales as
select sales.sales_eur, sales.client_id, store.store_name
from sales
inner join store
```

#### Materialized views

Some analytical queries just have to compute aggregated statistics over large quantities of data.
For instance, you might want to compute the total sales per store, or the total sales per product.
These queries are slow to compute when there are many rows, and you might not want to run them on every request.
You can use [materialized views](https://en.wikipedia.org/wiki/Materialized_view) to cache the results of these queries.
Materialized views are views that are stored as regular tables in the database.

Depending on the database, you might have to refresh the materialized view manually.
You can either refresh the view manually from inside your sql pages when you detect they are outdated,
or write an external script to refresh the view periodically.

```sql
create materialized view total_sales_per_store as
select store_name, sum(sales_eur) as total_sales
from sales
group by store_name;
```

### Use database indices

When a query on a large table uses non-primary column in a `WHERE`, `GROUP BY`, `ORDER BY`, or `JOIN`,
you should create an [index](https://en.wikipedia.org/wiki/Database_index) on that column.
When multiple columns are used in the query, you should create a composite index on those columns.
When creating a composite index, the order of the columns is important.
The most frequently used columns should be first.

```sql
create index idx_sales_store_date on sale (store_id, sale_date); -- useful for queries that filter by "store" or by "store and date"
create index idx_sales_product_date on sale (product_id, sale_date);
create index idx_sales_store_product_date on sale (store_id, product_id, sale_date);
```

Indexes are updated automatically when the table is modified.
They slow down the insertion and deletion of rows in the table,
but speed up the retrieval of rows in queries that use the indexed columns.

### Query performance debugging

When a query is slow, you can use the `EXPLAIN` keyword to see how the database will execute the query.
Just add `EXPLAIN` before the query you want to analyze.

On PostgreSQL, you can use a tool like [explain.dalibo.com](https://explain.dalibo.com/) to visualize the query plan.

What to look for:
 - Are indexes used? You should see references to the indices you created.
 - Are full table scans used? Large tables should never be scanned.
 - Are expensive operations used? Such as sorting, hashing, bitmap index scans, etc.
 - Are operations happening in the order you expected them to? Filtering large tables should come first.

### Vacuum your database regularly

On PostgreSQL, you can use the [`VACUUM`](https://www.postgresql.org/docs/current/sql-vacuum.html) command to garbage-collect and analyze a database.

On MySQL, you can use the [`OPTIMIZE TABLE`](https://dev.mysql.com/doc/refman/8.0/en/optimize-table.html) command to reorganize it on disk and make it faster.
On Microsoft SQL Server, you can use the [`DBCC DBREINDEX`](https://learn.microsoft.com/en-us/sql/t-sql/database-console-commands/dbcc-dbreindex-transact-sql?view=sql-server-ver17) command to rebuild the indexes.
On SQLite, you can use the [`VACUUM`](https://www.sqlite.org/lang_vacuum.html) command to garbage-collect and analyze the database.

### Use the right database engine

If the amount of data you are working with is very large, does not change frequently, and you need to run complex queries on it,
you could use a specialized analytical database such as [ClickHouse](https://clickhouse.com/) or [DuckDB](https://duckdb.org/).
Such databases can be used with SQLPage by using their [ODBC](https://en.wikipedia.org/wiki/Open_Database_Connectivity) drivers.

### Database-specific performance recommendations

 - [PostgreSQL "Performance Tips"](https://www.postgresql.org/docs/current/performance-tips.html)
 - [MySQL optimization guide](https://dev.mysql.com/doc/refman/8.0/en/optimization.html)
 - [Microsoft SQL Server "Monitor and Tune for Performance"](https://learn.microsoft.com/en-us/sql/relational-databases/performance/monitor-and-tune-for-performance?view=sql-server-ver17)
 - [SQLite query optimizer overview](https://www.sqlite.org/optoverview.html)

## SQLPage-specific advice

The best way to make your SQLPage webapp fast is to make your queries fast.
Sometimes, you just don''t have control over the database, and have to run slow queries.
This section will help you minimize the impact to your users.

### Order matters

SQLPage executes the queries in your `.sql` files in order.
It does not start executing a query before the previous one has returned all its results.
So, if you have to execute a slow query, put it as far down in the page as possible.

#### No heavy computation before the shell

Every user-facing page in a SQLPage site has a [shell](/components?component=shell).

The first queries in any sql file (all the ones that come before the [shell](/components?component=shell))
are executed before any data has been sent to the user''s browser.
During that time, the user will see a blank screen.
So, ensure your shell comes as early as possible, and does not require any heavy computation.
If you can make your shell entirely static (independent of the database), do so,
and it will be rendered before SQLPage even finishes acquiring a database connection.

#### Set variables just above their first usage

For the reasons explained above, you should avoid defining all variables at the top of your sql file.
Instead, define them just above their first usage.

### Avoid recomputing the same data multiple times

Often, a single page will require the same pieces of data in multiple places.
In this case, avoid recomputing it on every use inside the page.

#### Reusing a single database record

When that data is small, store it in a sqlpage variable as JSON and then 
extract the data you need using [json operations](/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide).

```sql
set product = (
  select json_object(''name'', name, ''price'', price) -- in postgres, you can simply use row_to_json(product)
  from products where id = $product_id
);

select ''alert'' as component, ''Product'' as title, $product->>''name'' as description;
```

#### Reusing a large query result set

You may have a page that lets the user filter a large dataset by many different criteria,
and then displays multiple charts and tables based on the filtered data.

In this case, store the filtered data in a temporary table and then reuse it in multiple places.

```sql
drop table if exists filtered_products;
create temporary table filtered_products as
select * from products where 
  ($category is null or category = $category) and
  ($manufacturer is null or manufacturer = $manufacturer);

select ''alert'' as component, count(*) || '' products'' as title
from filtered_products;

select ''list'' as component;
select name as title from filtered_products;
```

### Reduce the number of queries

Each query you execute has an overhead of at least the time it takes to send a packet back and forth
between SQLPage and the database.
When it''s possible, combine multiple queries into a single one, possibly using
[`UNION ALL`](https://en.wikipedia.org/wiki/Set_operations_(SQL)#UNION_operator).

```sql
select ''big_number'' as component;

with stats as (
    select count(*) as total, avg(price) as average_price from filtered_products
)
select ''count'' as title, stats.total as value from stats
union all
select ''average price'' as title, stats.average_price as value from stats;
```

### Lazy loading

Use the [card](/component?component=card) and [modal](/component?component=modal) components 
with the `embed` attribute to load data lazily.
Lazy loaded content is not sent to the user''s browser when the page initially loads,
so it does not block the initial rendering of the page and provides a better experience for
data that might be slow to load.

### Database connections

SQLPage uses connection pooling: it keeps multiple database connections opened,
and reuses them for consecutive requests. When it does not receive requests for a long time,
it closes idle connection. When it receives many requests, it opens new connection,
but never more than the value specified by `max_database_pool_connections` in its
[configuration](https://github.com/sqlpage/SQLPage/blob/main/configuration.md).
You can increase the value of that parameter if your website has many concurrent users and your
database is configured to allow opening many simultaneous connections.

### SQLPage performance debugging

When `environment` is set to `development` in its [configuration](https://github.com/sqlpage/SQLPage/blob/main/configuration.md),
SQLPage will include precise measurement of the time it spends in each of the steps it has to go through before starting to send data
back to the user''s browser. You can visualize that performance data in your browser''s network inspector.

You can set the `RUST_LOG` environment variable to `sqlpage=debug` to make SQLPage
print detailed messages associated with precise timing for everything it does.
');

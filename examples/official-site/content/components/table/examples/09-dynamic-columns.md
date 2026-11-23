#### Dynamic column names in a table

In all the previous examples, the column names were hardcoded in the SQL query.
This makes it very easy to quickly visualize the results of a query as a table,
but it can be limiting if you want to include columns that are not known in advance.
In situations when the number and names of the columns depend on the data, or on variables,
you can use the `dynamic` component to generate the table columns dynamically.

For that, you will need to return JSON objects from your SQL query, where the keys are the column names,
and the values are the cell contents.

Databases [offer utilities to generate JSON objects from query results](/blog.sql?post=JSON%20in%20SQL%3A%20A%20Comprehensive%20Guide)
 - In PostgreSQL, you can use the [`json_build_object`](https://www.postgresql.org/docs/current/functions-json.html#FUNCTIONS-JSON-PROCESSING)
function for a fixed number of columns, or [`json_object_agg`](https://www.postgresql.org/docs/current/functions-aggregate.html#FUNCTIONS-AGGREGATE) for a dynamic number of columns.
 - In SQLite, you can use the [`json_object`](https://www.sqlite.org/json1.html) function for a fixed number of columns,
or the `json_group_object` function for a dynamic number of columns.
 - In MySQL, you can use the [`JSON_OBJECT`](https://dev.mysql.com/doc/refman/8.0/en/json-creation-functions.html#function_json-object) function for a fixed number of columns,
or the [`JSON_OBJECTAGG`](https://dev.mysql.com/doc/refman/8.4/en/aggregate-functions.html#function_json-objectagg) function for a dynamic number of columns.
 - In Microsoft SQL Server, you can use the [`FOR JSON PATH`](https://docs.microsoft.com/en-us/sql/relational-databases/json/format-query-results-as-json-with-for-json-sql-server?view=sql-server-ver15) clause.

For instance, let's say we have a table with three columns: store, item, and quantity_sold.
We want to create a pivot table where each row is a store, and each column is an item.
We will return a set of json objects that look like this: `{"store":"Madrid", "Item1": 42, "Item2": 7, "Item3": 0}`

```sql
SELECT 'table' AS component;
with filled_data as (
  select
    stores.store, items.item,
    (select coalesce(sum(quantity_sold), 0) from store_sales where store=stores.store and item=items.item) as quantity
  from (select distinct store from store_sales) as stores
  cross join (select distinct item from store_sales) as items
  order by stores.store, items.item
)
SELECT
    'dynamic' AS component,
    JSON_PATCH( -- SQLite-specific, refer to your database documentation for the equivalent JSON functions
        JSON_OBJECT('store', store),
        JSON_GROUP_OBJECT(item, quantity)
    ) AS properties
FROM
    filled_data
GROUP BY
    store;
```

This will generate a table with the stores in the first column, and the items in the following columns, with the quantity sold in each store for each item.

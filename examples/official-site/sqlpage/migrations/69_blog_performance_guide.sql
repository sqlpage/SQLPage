
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

Your database schema should be made so that one piece of information
is stored in only one place in the database.
For instance, if you are modelling sales that happen in stores, the sales table should 
contain a foreign key to another table named stores.
This way, when you need to display the list of stores in your application, you don''t have to 
run a slow `select distinct store from sales`, that would have to go through your millions of sales
(*even if you have an index on the store column*),

### Use database indices

### Use (materialized) views

### Query performance debugging

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

The first queries in any sql file (all the ones that come before the [])

#### Set variables just above their first usage

### Avoid recomputing the same data multiple times

### Reduce the number of queries

### Lazy loading

Use the card and modal components to load data lazily.

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

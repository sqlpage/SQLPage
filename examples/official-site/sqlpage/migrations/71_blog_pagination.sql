
INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'How to use the pagination component',
        'Concrete advice on how to make your SQLPage webapp fast',
        'sailboat-2',
        '2025-11-10',
        '
# How to use the pagination component

To display a large number of records from a database, it is often practical to split these data into pages. The user can thus navigate from one page to another, as well as directly to the first or last page. With SQLPage, it is possible to perform these operations using the pagination component.

This component offers many options, and I recommend consulting its documentation before proceeding with the rest of this tutorial.

Of course, this component only handles its display and does not implement any logic for data processing or state changes. In this tutorial, we will implement a complete example of using the pagination component with a SQLite database, but the code should work without modification (or with very little modification) with any relational database management system (RDBMS).

## Initialization

We first need to define two constants that indicate the maximum number of rows per page and the maximum number of pages that the component should display.

```
SET MAX_RECORD_PER_PAGE = 10;
SET MAX_PAGES = 10;
```

Now, we need to know the number of rows present in the table to be displayed. We can then calculate the number of pages required.

```
SET records_count = (SELECT COUNT(*) FROM album);
SET pages_count = (CAST($records_count AS INTEGER) / CAST($MAX_RECORD_PER_PAGE AS INTEGER));
```

It is possible that the number of rows in the table is greater than the estimated number of pages multiplied by the number of rows per page. In this case, it is necessary to add an additional page.
 
```
SET pages_count = (
    CASE 
        WHEN (CAST($pages_count AS INTEGER) * CAST($MAX_RECORD_PER_PAGE AS INTEGER)) = CAST($records_count AS INTEGER) THEN $pages_count 
        ELSE (CAST($pages_count AS INTEGER) + 1) 
    END
);
```

We will need to transmit the page number to be displayed in the URL using the `page` parameter. We do the same for the number of the first page (`idx_page`) appearing at the left end of the pagination component. 

![Meaning of URL parameters](blog/pagination.png)


If the page number or index is not present in the URL, the value of 1 is applied by default.

```
SET page = COALESCE($page,1);
SET idx_page = COALESCE($idx_page,1);
```

## Read the data

We can now read and display the data based on the active page. To do this, we simply use a table component. 

```
SELECT 
    ''table'' as component
SELECT
    AlbumId AS id, 
    Title AS title 
FROM
    album
LIMIT CAST($MAX_RECORD_PER_PAGE AS INTEGER)
OFFSET (CAST($page AS INTEGER) - 1) * CAST($MAX_RECORD_PER_PAGE AS INTEGER);
```

The SQL LIMIT clause allows us to not read more rows than the maximum allowed for a page. With the SQL OFFSET clause, we specify from which row the data is selected.

On each HTML page load, the table content will be updated based on the `page` and `idx_page` variables, whose values will be extracted from the URL

## Set up the pagination component

Now, we need to set up the parameters that will be included in the URL for the buttons to navigate to the previous or next page.

If the user wants to view the previous page and the current page is not the first one, the value of the `page` variable is decremented. The same applies to `idx_page`, which is decremented if its value does not correspond to the first page.

```
SET previous_parameters = (
    CASE
        WHEN CAST($page AS INTEGER) > 1 THEN
            json_object(
                ''page'', (CAST($page AS INTEGER) - 1),
                ''idx_page'', (CASE 
                    WHEN CAST($idx_page AS INTEGER) > 1 THEN (CAST($idx_page AS INTEGER) - 1)
                    ELSE $idx_page
                END) 
            )
        ELSE json_object() END  
);
```

The logic is quite similar for the URL to view the next page. First, it is necessary to verify that the user is not already on the last page. Then, the `page` variable can be incremented and the `idx_page` variable updated.

```
SET next_parameters = (
    CASE
        WHEN CAST($page AS INTEGER) < CAST($pages_count AS INTEGER) THEN
            json_object(
                ''page'', (CAST($page AS INTEGER) + 1),
                ''idx_page'', (CASE 
                    WHEN CAST($idx_page AS INTEGER) < (CAST($pages_count AS INTEGER) - CAST($MAX_PAGES AS INTEGER) + 1) THEN (CAST($idx_page AS INTEGER) + 1)
                    ELSE $idx_page
                END) 
            )
        ELSE json_object() END
);
```

We can now add the pagination component, which is placed below the table displaying the data. All the logic for managing the buttons is entirely handled in SQL:
- the buttons to access the first or last page,
- the buttons to view the previous or next page,
- the enabling or disabling of these buttons based on the context.

```
SELECT
    ''pagination'' AS component,
    (CAST($page AS INTEGER) = 1) AS first_disabled,
    (CAST($page AS INTEGER) = 1) AS previous_disabled,
    (CAST($page AS INTEGER) = CAST($pages_count AS INTEGER)) AS next_disabled,
    (CAST($page AS INTEGER) = CAST($pages_count AS INTEGER)) AS last_disabled,
    sqlpage.link(sqlpage.path(), json_object(''page'', 1, ''idx_page'', 1)) as first_link,
    sqlpage.link(sqlpage.path(), $previous_parameters) AS previous_link,
    sqlpage.link(sqlpage.path(), $next_parameters) AS next_link,
    sqlpage.link(
        sqlpage.path(),
        json_object(''page'', $pages_count, ''idx_page'', (
            CASE
                WHEN (CAST($pages_count AS INTEGER) <= CAST($MAX_PAGES AS INTEGER)) THEN 1
                ELSE (CAST($pages_count AS INTEGER) - CAST($MAX_PAGES AS INTEGER) + 1) 
            END)
        )
    ) AS last_link,
    TRUE AS outline;
```

The final step is to generate the page numbers based on the number of pages and the index of the first page displayed to the left of the component. To do this, we use a recursive CTE query. 

```
WITH RECURSIVE page_numbers AS (
    SELECT $idx_page AS number
    UNION ALL
    SELECT number + 1
    FROM page_numbers
    LIMIT CAST($MAX_PAGES AS INTEGER)
)
SELECT 
    number AS contents,
    sqlpage.link(sqlpage.path(), json_object(''page'', number, ''idx_page'', $idx_page)) as link,
    (number = CAST($page AS INTEGER)) AS active 
FROM page_numbers;
```

If the added page matches the content of the `page` variable, the `active` option is set to `TRUE` so that the user knows it is the current page.
');
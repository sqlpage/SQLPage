-- Welcome to SQLPage ! This is a short demonstration of a few things you can do with SQLPage
-- Using the 'shell' component at the top allows you to customize your web page, giving it a title and a description
select 'shell' as component,
    'SQLpage' as title,
    '/' as link,
    'en' as lang,
    'Welcome to SQLPage' as description;
-- Making a web page with SQLPage works by using a set of predefined "components"
--  and filling them with contents from the results of your SQL queries
select 'hero' as component, -- We select a component. The documentation for each component can be found on https://sql.ophir.dev/documentation.sql
    'It works !' as title, -- 'title' is top-level parameter of the 'hero' component
    'If you can see this, then SQLPage is running correctly on your server. Congratulations! ' as description;
-- Properties can be textual, numeric, or booleans

-- Let's start with the text component
SELECT 'text' as component, -- We can switch to another component at any time just with a select statement.
    'Get started' as title;
-- We are now inside the text component. Each row that will be returned by our SELECT queries will be a span of text
-- The text component has a property called "contents" that can be  that we use to set the contents of our block of text
-- and a property called "center" that we use to center the text
SELECT 'In order to get started ' as contents;
select 'visit SQLPage''s website' as contents,
    'https://sql.ophir.dev/' as link,
    true as italics;
SELECT '. You can replace this page''s contents by creating a file named ' as contents;
SELECT 'index.sql' as contents, true as italics;
SELECT ' in the folder where sqlpage is running. ' as contents;
SELECT 'Alternatively, you can create a table called sqlpage_files in your database with the following columns: path, contents, and last_modified.' as contents;

select 'text' as component,
    'Demo' as title;
-- We can switch to another component at any time just with a select statement.
-- Let's draw a chart
select 'chart' as component,
    'Revenue per country' as title,
    'bar' as type,
    'time' as xtitle,
    'price' as ytitle,
    true as stacked;
-- Inside the chart component, we have access to the "series", "label", and "value" properties
-- Here, we are selecting static data, but you can also use a query to a real database
select 'Russia' as series,
    '2022-01' as label,
    2 as value
union
select 'Russia',
    '2022-02',
    4
union
select 'Russia',
    '2022-03',
    2;
select 'Brasil' as series,
    '2022-01' as label,
    4 as value
union
select 'Brasil',
    '2022-03',
    1
union
select 'Brasil',
    '2022-04',
    1;
-- Let's make a new chart, this time generating the data with a more complex query
select 'chart' as component,
    'Collatz conjecture' as title,
    'area' as type;

SELECT 'syracuse' as series, x, y
FROM (
      SELECT 0 AS x, 15 AS y UNION SELECT 1, 46 UNION SELECT 2, 23 UNION SELECT 3, 70 UNION SELECT 4, 35 UNION SELECT 5, 106 UNION SELECT 6, 53 UNION SELECT 7, 160 UNION SELECT 8, 80 UNION SELECT 9, 40 UNION SELECT 10, 20 UNION SELECT 11, 10 UNION SELECT 12, 5
) AS syracuse ORDER BY x;

select 'table' as component,
    true as sort,
    true as search;
-- The table component lets you just select your data as it is, without imposing a particular set of properties
select 'John' as "First Name",
    'Doe' as "Last Name",
    1994 as "Birth Date"
union
select 'Jane',
    'Smith',
    1989;
-- Here, things get a little more interesting. We are making a small app to learn our times table
-- We will display a set of cards, each one displaying the result of the multiplication a * b
select 'card' as component,
    5 as columns;

WITH nums(x) AS (
    SELECT 1 UNION SELECT 2 UNION SELECT 3 UNION SELECT 4 UNION SELECT 5 UNION SELECT 6 UNION SELECT 7 UNION SELECT 8 UNION SELECT 9 UNION SELECT 10
)
SELECT a.x || ' times ' || b.x as title,
    CASE
        a.x % 4
        WHEN 0 THEN 'red'
        WHEN 1 THEN 'green'
        WHEN 3 THEN 'yellow'
        ELSE 'blue'
    END as color,
    a.x || ' x ' || b.x || ' = ' || (a.x * b.x) as description,
    'This is basic math' as footer,
    '?x=' || a.x as link -- This is the interesting part. Each card has a link. When you click the card, the current page is reloaded with '?x=a' appended to the end of the URL
FROM nums as a, nums as b
WHERE -- The powerful thing is here
    $x IS NULL
    OR -- The syntax $x allows us to extract the value 'a' when the URL ends with '?x=a'. It will be null if the URL does not contain '?x='
    b.x = $x::DECIMAL
ORDER BY a.x, b.x;
-- So when we click the card for "a times b", we will reload the page, and display only the multiplication table of a
---------------------------
-- FORMS --
-- Until now, we have only read data. Let's see how we can write new data to our database
-- You can use an existing table in your database
-- or create the table by just creating a file at 'sqlpage/migrations/00_create_users.sql'
-- containing the SQL query to create the table. For this example, we will use:
-- CREATE TABLE IF NOT EXISTS users(name TEXT);

-- Displaying a form is as easy as displaying a table; we use the "form" component
-- Let's display a form to our users
select 'form' as component,
    'Create' as validate,
    'New User' as title;
select 'number' as type,
    'age' as placeholder;
select 'First Name' as name,
    true as autocomplete,
    true as required,
    'We need your name for legal reasons.' as description;
select 'Last name' as name,
    true as autocomplete;
select 'radio' as type,
    'favorite_food' as name,
    'banana' as value,
    'I like bananas the most' as label;
select 'radio' as type,
    'favorite_food' as name,
    'cake' as value,
    'I like cake more' as label,
    'Bananas are okay, but I prefer cake' as description;
select 'checkbox' as type,
    'checks[]' as name,
    1 as value,
    'Accept the terms and conditions' as label;
select 'checkbox' as type,
    'checks[]' as name,
    2 as value,
    'Subscribe to the newsletter' as label;

-- We can access the values entered in the form using the syntax :xxx where 'xxx' is the name of one of the fields in the form
-- insert into users select :"First Name" where :"First Name" IS NOT NULL;
-- We don't want to add a line in the database if the page was loaded without entering a value in the form, so we add a WHERE clause
-- Let's show the users we have in our database
-- select 'list' as component, 'Users' as title;
-- select name as title from users;
-- The debug component displays the raw results returned by a query
select 'debug' as component;
select $x as x,
    :"First Name" as firstName,
    :checks as checks;
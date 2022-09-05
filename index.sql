-- Welcome to SQLPage ! This is a short demonstration of a few things you can do with SQLPage

-- The first SELECT in your page allow you to customize your web page, giving it a title and a description
select 'sqlpage' as title, '/' as link, 'en' as lang, 'My cool app' as description;

-- Making a web page with SQLPage works by using a set of predefined "components"
--  and filling them with contents from the results of your SQL queries

-- Let's start with the text component :
select 'text' as component,
       'Welcome to SQLPage' as title; -- The text component has a property called "title" that we use to set the title of our block of text
-- We are now inside the text component. Each row that will be returned by our SELECT queries will be a span of text
select 'I can''t believe I am writing a website with nothing but SQL ! ' as contents,
        true as italics; -- The text component has a property called "contents" and another called "italics".
select 'This is a normal SQL query. This text is hardcoded, but it could as well come from a database.' as contents;

select 'default' as component;
select 'Welcome to sqlpage!', 'Look at how easy it is to write a website !';

-- We can switch to another component at any time just with a select statement.
-- Let's draw a chart
select 'chart' as component,
       'Revenue per country' as title,
       'bar'             as type,
       'time'            as xtitle,
       'price'           as ytitle,
        true             as stacked;
-- Inside the chart component, we have access to the "series", "label", and "value" properties
select 'Russia' as series, '2022-01' as label, 2 as value
union select 'Russia', '2022-02',4
union select 'Russia', '2022-03',2;
select 'Brasil' as series, '2022-01' as label, 4 as value
union select 'Brasil', '2022-03',1
union select 'Brasil', '2022-04',1;

-- Let's make a new chart, this time generating the data with a more complex query
select 'chart' as component, 'Collatz conjecture' as title, 'area' as type;
WITH RECURSIVE cnt(x,y) AS (
    VALUES(0,15) UNION ALL
    SELECT
        x+1,
        CASE y%2 WHEN 0 THEN y/2 ELSE 3*y+1 END
    FROM cnt WHERE x<12
) SELECT 'syracuse' as series, x, y from cnt;

select 'table' as component, true as sort, true as search;
-- The table component lets you just select your data as it is, without imposing a particular set of properties
select 'John' as "First Name", 'Doe' as "Last Name", 1994 as "Birth Date"
union select 'Jane', 'Smith', 1989;

-- Here, things get a little more interesting. We are making a small app to learn our times table
-- We will display a set of cards, each one displaying the result of the multiplication a * b
select 'card' as component, 5 as columns;
WITH RECURSIVE cnt(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM cnt WHERE x<10)
SELECT
    a.x || ' times ' || b.x as title,
     CASE a.x % 4 WHEN 0 THEN 'red' WHEN 1 THEN 'green' WHEN 3 THEN 'yellow' ELSE 'blue' END as color,
    a.x || ' x ' || b.x || ' = ' || (a.x*b.x) as description,
    'This is basic math' as footer,
    '?x=' || a.x as link -- This is the interesting part. Each card has a link. When you click the card, the current page is reloaded with '?x=a' appended to the end of the URL
FROM cnt as a, cnt as b
WHERE -- The powerful thing is here
    $x IS NULL OR -- The syntax $x allows us to extract the value 'a' when the URL ends with '?x=a'. It will be null if the URL does not contain '?x='
    b.x = $x::INTEGER; -- So when we click the card for "a times b", we will reload the page, and display only the multiplication table of a

-- Until now, we have only read data. Let's see how we can write new data to our database
-- I am creating a table directly for this example, but you would normally use an existing table in your database
-- or create the table in a migration file in 'sqlpage/migrations/00_create_users.sql'
create table if not exists users(name text);

-- Let's display a form to our users
select 'form' as component, 'Create' as validate, 'New User' as title;
select 'number' as type, 'age' as placeholder;
select 'First Name' as name, true as autocomplete, true as required, 'We need your name for legal reasons.' as description;
select 'Last name' as name, true as autocomplete;
select 'radio' as type, 'favorite_food' as name, 'banana' as value, 'I like bananas the most' as label;
select 'radio' as type, 'favorite_food' as name, 'cake' as value, 'I like cake more' as label, 'Bananas are okay, but I prefer cake' as description;
select 'checkbox' as type, 'checks[]' as name, 1 as value, 'Accept the terms and conditions' as label;
select 'checkbox' as type, 'checks[]' as name, 2 as value, 'Subscribe to the newsletter' as label;

-- We can access the values entered in the form using the syntax :xxx where 'xxx' is the name of one of the fields in the form
insert into users select :"First Name"
where :"First Name" IS NOT NULL; -- We don't want to add a line in the database if the page was loaded without entering a value in the form, so we add a WHERE clause

-- Let's show the users we have in our database
select 'list' as component, 'Users' as title;
select name as title from users;

-- The debug component displays the raw results returned by a query
select 'debug' as component;
select $x as x, :"First Name" as firstName, :checks as checks;
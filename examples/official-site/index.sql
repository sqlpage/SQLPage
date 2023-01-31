select 'http_header' as component, 'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";
-- Using the 'shell' component at the top allows you to customize your web page, giving it a title and a description
select 'shell' as component,
    'SQLPage' as title,
    'file-database' as icon,
    '/' as link,
    'en-US' as lang,
    'Official SQLPage website: write web applications in SQL !' as description,
    'documentation' as menu_item;

select 'text' as component, 'SQLPage: turn any database into a webapp' as title;
SELECT 'SQLPage is a powerful tool for creating websites using only SQL. With SQLPage, '||
    'you can create dynamic and functional websites without the need for complex programming languages. '||
    'Simply define your website using a set of predefined components and fill them with the results of your SQL queries. '||
    'This allows you to quickly and easily create a website that is tailored to your specific needs and requirements.' as contents, 5 as size;

select 'text' as component, 'SQL-only' as title;
SELECT 'One of the key benefits of SQLPage is that it allows you to use the SQL skills you and your team already have ' ||
    'to create a website. ' ||
    'If you are already familiar with SQL, you can create a SQL Page in a few minutes right now ' ||
    'without the need to learn additional programming languages. ' ||
    'If you are not, start learning SQL now, and you will have the level it takes to build a website in a week, versus months to learn a full-fledged programming language. ' ||
    'This makes SQLPage an ideal tool for people who are not professional programmers, but still need to create a website with a database: ' ||
     'that means data scientists, engineers, analysts, marketers, and others, who already have access to databases, ' ||
     'but are currently limited to creating dashboards and static reports can now create full-fledged dynamic web applications.' as contents, 4 as size;

select 'text' as component, 'Pre-defined components for everything' as title;
SELECT 'In addition to its ease of use, SQLPage offers a wide range of features and capabilities. '||
    'You can create a wide variety of components, including text blocks, charts, lists, and forms, and customise them with a variety of properties. '||
    'This allows you to create a website that is tailored to your specific needs and requirements.' as contents, 4 as size;

select 'text' as component, 'Get started!' as title;
SELECT 'Overall, SQLPage is a powerful and innovative tool for creating websites using only SQL. With its ease of use and wide range of features, it offers a unique and valuable way to create dynamic and functional websites without the need for complex programming languages. If you want to create a website with a database, SQLPage is the perfect tool for you.' as contents, 4 as size;

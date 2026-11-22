select
    'http_header' as component,
    'text/markdown; charset=utf-8' as "Content-Type",
    'inline; filename="llms.txt"' as "Content-Disposition";

with parameter_sections as (
    select kind, entry, section_order, '#### ' || title || char(10, 10) || group_concat(
        format('- `%s` (%s)%s: %s', name, type, case when not optional then ' **REQUIRED**' else '' end,
            replace(trim(description_md, char(13, 10)), char(10), char(10) || '  ')),
        char(10) order by position, name
    ) as contents
    from reference_parameter
    group by kind, entry, section_order, title
), parameters as (
    select kind, entry, group_concat(contents, char(10, 10) order by section_order) as contents
    from parameter_sections group by kind, entry
), entries as (
    select e.kind, e.name,
        '### [' || case e.kind when 'function' then 'sqlpage.' || e.name || '()' else e.name end || '](' || e.link || ')' ||
        coalesce(char(10, 10) || 'Since SQLPage ' || e.introduced_in_version || '.', '') ||
        coalesce(char(10, 10) || 'Returns `' || e.return_type || '`.', '') ||
        -- Quote the original Markdown so its headings and code fences stay inside the entry.
        char(10, 10) || '> ' || replace(trim(e.description_md, char(13, 10)), char(10), char(10) || '> ') ||
        coalesce(char(10, 10) || p.contents, '') as contents
    from reference_entry as e left join parameters as p on p.kind = e.kind and p.entry = e.name
), chapters as (
    select kind, group_concat(contents, char(10, 10) order by name) as contents
    from entries group by kind
)
select
    'shell-empty' as component,
    '# SQLPage

> SQLPage is a SQL-only web application framework. It lets you build entire websites and web applications using nothing but SQL queries. Write `.sql` files, and SQLPage executes them, maps results to UI components (handlebars templates), and streams HTML to the browser.

SQLPage is designed for developers who are comfortable with SQL but want to avoid the complexity of traditional web frameworks. It works with SQLite, PostgreSQL, MySQL, and Microsoft SQL Server, and through ODBC with any other database that has an ODBC driver installed.

Key features:
- No backend code needed: Your SQL files are your backend
- Component-based UI: Built-in components for forms, tables, charts, maps, and more
- Database-first: Every HTTP request triggers a sequence of SQL queries from a .sql file, the results are rendered with built-in or custom components, defined as .handlebars files in the sqlpage/templates folder.
- Simple deployment: Single binary with no runtime dependencies
- Secure by default: Parameterized queries prevent SQL injection

## Getting Started

- [Introduction to SQLPage: installation, guiding principles, and a first example](/your-first-sql-website/tutorial.md): Complete beginner tutorial covering setup, database connections, forms, and deployment

## Core Documentation

- [Components reference](/components.sql): List of all ' || (select count(*) from component) || ' built-in UI components with parameters and examples
- [Functions reference](/functions.sql): SQLPage built-in functions for handling requests, encoding data, and more
- [Configuration guide](https://github.com/sqlpage/SQLPage/blob/main/configuration.md): Complete list of configuration options in sqlpage.json

## Components

' || (select contents from chapters where kind = 'component') || '

## Functions

' || (select contents from chapters where kind = 'function') || '

## Examples

- [Authentication example](https://github.com/sqlpage/SQLPage/tree/main/examples/user-authentication): Complete user registration and login system
- [CRUD application](https://github.com/sqlpage/SQLPage/tree/main/examples/CRUD%20-%20Authentication): Create, read, update, delete with authentication
- [Image gallery](https://github.com/sqlpage/SQLPage/tree/main/examples/image%20gallery%20with%20user%20uploads): File upload and image display
- [Todo application](https://github.com/sqlpage/SQLPage/tree/main/examples/todo%20application): Simple CRUD app
- [Master-detail forms](https://github.com/sqlpage/SQLPage/tree/main/examples/master-detail-forms): Working with related data
- [Charts example](https://github.com/sqlpage/SQLPage/tree/main/examples/plots%20tables%20and%20forms): Data visualization

## Optional

- [Custom components guide](/custom_components.sql): Create your own handlebars components
- [Safety and security](/safety.sql): Understanding SQL injection prevention
- [Docker deployment](https://github.com/sqlpage/SQLPage#with-docker): Running SQLPage in containers
- [Systemd service](https://github.com/sqlpage/SQLPage/blob/main/sqlpage.service): Production deployment setup
- [Repository structure](https://github.com/sqlpage/SQLPage/blob/main/CONTRIBUTING.md): Project organization and contribution guide
' as html;

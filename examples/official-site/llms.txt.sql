select
    'http_header' as component,
    'text/markdown; charset=utf-8' as "Content-Type",
    'inline; filename="llms.txt"' as "Content-Disposition";

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

- [Components reference](/documentation.sql): List of all ' || (
        select
            count(*)
        from
            component
    ) || ' built-in UI components with parameters and examples
- [Functions reference](/functions.sql): SQLPage built-in functions for handling requests, encoding data, and more
- [Configuration guide](https://github.com/sqlpage/SQLPage/blob/main/configuration.md): Complete list of configuration options in sqlpage.json

## Components

' || (
        select
            group_concat (
                '### [' || c.name || '](/component.sql?component=' || c.name || ')

' || c.description || '

' || (
                    select
                        case when exists (
                                select
                                    1
                                from
                                    parameter
                                where
                                    component = c.name
                                    and top_level
                            ) then '#### Top-level parameters

' || group_concat (
                                    '- `' || name || '` (' || type || ')' || case when not optional then ' **REQUIRED**' else '' end || ': ' || description,
                                    char(10)
                                )
                        else
                            ''
                        end
                    from
                        parameter
                    where
                        component = c.name
                        and top_level
                ) || '

' || (
                    select
                        case when exists (
                                select
                                    1
                                from
                                    parameter
                                where
                                    component = c.name
                                    and not top_level
                            ) then '#### Row-level parameters

' || group_concat (
                                    '- `' || name || '` (' || type || ')' || case when not optional then ' **REQUIRED**' else '' end || ': ' || description,
                                    char(10)
                                )
                        else
                            ''
                        end
                    from
                        parameter
                    where
                        component = c.name
                        and not top_level
                ) || '

',
                ''
            )
        from
            component c
        order by
            c.name
    ) || '

## Functions

' || (
        select
            group_concat (
                '### [sqlpage.' || name || '()](/functions.sql?function=' || name || ')
' || replace (
                    replace (
                        description_md,
                        char(10) || '#',
                        char(10) || '###'
                    ),
                    '  ',
                    ' '
                ),
                char(10)
            )
        from
            sqlpage_functions
        order by
            name
    ) || '

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
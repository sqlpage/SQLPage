# **SQLPage** : Build a web application with a few SQL queries

SQLPage is an open-source tool that empowers database people to quickly build beautiful dynamic web applications *entirely in SQL*. 

Designed to seamlessly integrate with PostgreSQL, SQLPage enables data practitioners to leverage their SQL skills to create robust, data-centric web apps without the need for traditional web programming languages, thanks to its [rich library of built-in web components](https://sql.ophir.dev/documentation.sql) that can be invoked directly from basic SQL queries.

It lets you create complex dynamic webapps for data analysis, visualization, data ingestion, internal tooling, administration panels, prototyping, and more just by writing simple standard `.sql` files. 

## Introduction

SQLPage opens the world of easy web application development to database specialists. Built with the new capabilities of modern database software in mind, SQLPage makes it quick and easy to build user interfaces on top of databases, without locking you into a proprietary system: it's all plain old _standard_ SQL. Say goodbye to complex frameworks and proprietary ecosystems – SQLPage offers a refreshing approach that unlocks the full potential of PostgreSQL.


## Key Features

- **Database-Centric Approach**: SQLPage keeps your database at the center of your application, preserving data integrity and leveraging PostgreSQL's rich functionality.
- **Rapid Prototyping**: Develop a minimum viable product (MVP) in a matter of hours, allowing you to quickly validate your ideas and iterate on them.
- **Full SQL support**: SQLPage is not limited to building read-only web views of your database; it supports inserting, updating and deleting data easily. 
- **Seamless Integration**: SQLPage seamlessly connects to your PostgreSQL database, leveraging its robustness, performance, and scalability.
- **Component-Based UI**: Create beautiful and interactive user interfaces without torturing yourself with CSS using pre-built web components, providing a professional look and feel.


## Use Cases

- **Internal Dashboards**: Build data-driven dashboards and reporting tools that empower teams to make informed decisions.
- **Business Intelligence Apps**: Develop powerful business intelligence applications with intuitive interfaces for data exploration and analysis.
- **Rapid Prototyping**: Validate and iterate on your ideas quickly by rapidly creating functional _minimum viable products_.
- **Admin Interfaces**: Construct efficient and user-friendly administrative interfaces for managing and interacting with your PostgreSQL data.

## Example

Here are the exact two SQL queries that builds the list of components of the documentation page on [SQLPage's official website](https://sql.ophir.dev)

```
SELECT 'list' AS component, 'components' AS title;
```

```
SELECT
    name AS title,
    description,
    icon,
    '?component='||name||'#component' AS link,
    $component = name AS active
from component
order by name;
```

## Get Started

To explore the possibilities and limitations of SQLPage, visit [the official website](https://sql.ophir.dev) and read the [SQL website building tutorial](https://sql.ophir.dev/get%20started.sql). Join the [SQLPage community](https://github.com/lovasoa/SQLpage/discussions) to discuss your PostgreSQL-powered web applications.

## Contributing

SQLPage is an open-source project, and contributions from the PostgreSQL community are highly encouraged. Visit [the GitHub repository](https://github.com/lovasoa/sqlpage) to contribute, report issues, or submit feature requests.

Discover the power of SQL-driven web application development with SQLPage and take your PostgreSQL experience to new heights!
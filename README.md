<h1 align="center">
SQLpage
</h1>

[![A short video explaining the concept of sqlpage](./docs/sqlpage.gif)](./docs/sqlpage.mp4)

[SQLpage](https://sql-page.com) is an **SQL**-only webapp builder.
It allows building powerful data-centric user interfaces quickly,
by tranforming simple database queries into interactive websites.

With SQLPage, you write simple `.sql` files containing queries to your database
to select, group, update, insert, and delete your data, and you get good-looking clean webpages
displaying your data as text, lists, grids, plots, and forms.

## Examples

<table>
<thead>
<tr><td>Code<td>Result</tr>
</thead>
<tbody>
<tr>
<td>

```sql
SELECT 
    'list' as component,
    'Popular websites' as title;
SELECT 
    name as title,
    url as link,
    CASE type
      WHEN 1 THEN 'blue'
      ELSE 'red'
    END as color,
    description, icon, active
FROM website;
```

<td>
    
![SQLPage list component](./docs/demo-list.png)

</tr>
<tr>
<td>

```sql
SELECT
  'chart' as component,
  'Quarterly Revenue' as title,
  'area' as type;

SELECT
    quarter AS x,
    SUM(revenue) AS y
FROM finances
GROUP BY quarter
```

<td>

![SQLPage list component](./docs/demo-graph.png)

</tr>
<tr>
<td>

```sql
SELECT
    'form' as component,
    'User' as title,
    'Create new user' as validate;

SELECT
    name, type, placeholder,
    required, description
FROM user_form;

INSERT INTO user
SELECT $first_name, $last_name, $birth_date
WHERE $first_name IS NOT NULL;
```

<td>

![SQLPage list component](./docs/demo-form.png)

</tr>
<tr>
<td>

```sql
select 'tab' as component, true as center;
select 'Show all cards' as title, '?' as link,
  $tab is null as active;
select
  format('Show %s cards', color) as title,
  format('?tab=%s', color) as link,
  $tab=color as active
from tab_example_cards
group by color; 


select 'card' as component;
select
  title, description, color
  image_url as top_image, link
from tab_example_cards
where $tab is null or $tab = color;

select
  'text' as component,
  sqlpage.read_file_as_text('footer.md') as contents_md
```

<td>

![card component sql example](./docs/cards.png)

</tr>
</tbody>
</table>

## Supported databases

- [SQLite](https://www.sqlite.org/index.html), including the ability to [load extensions](./configuration.md) such as *Spatialite*.
- [PostgreSQL](https://www.postgresql.org/), and other compatible databases such as *YugabyteDB*, *CockroachDB* and *Aurora*.
- [MySQL](https://www.mysql.com/), and other compatible databases such as *MariaDB* and *TiDB*.
- [Microsoft SQL Server](https://www.microsoft.com/en-us/sql-server), and all compatible databases and providers such as *Azure SQL* and *Amazon RDS*.
- **ODBC-compatible databases** such as *ClickHouse*, *MongoDB*, *DuckDB*, *Oracle*, *Snowflake*, *BigQuery*, *IBM DB2*, and many others through ODBC drivers.

## Get started

[Read the official *get started* guide on SQLPage's website](https://sql-page.com/get_started.sql).

### Using executables

The easiest way to get started is to download the latest release from the
[releases page](https://github.com/sqlpage/SQLPage/releases).

- Download the binary that corresponds to your operating system (linux, macos, or windows).
- Uncompress it: `tar -xzf sqlpage-*.tgz`
- Run it: `./sqlpage.bin`

### With docker

To run on a server, you can use [the docker image](https://hub.docker.com/r/lovasoa/SQLPage):

- [Install docker](https://docs.docker.com/get-docker/)
- In a terminal, run the following command:
  - `docker run -it --name sqlpage -p 8080:8080 --volume "$(pwd):/var/www" --rm lovasoa/sqlpage`
  - (`"$(pwd):/var/www"` allows sqlpage to run sql files from your current working directory)
- Create a file called index.sql with the contents from [this example](./index.sql)
- Open https://localhost:8080 in your browser
- Optionally, you can also mount a directory containing sqlpage's configuration file,
  custom components, and migrations
  (see [configuration.md](./configuration.md)) to `/etc/sqlpage` in the container.
     - For instance, you can use:
       - `docker run -it --name sqlpage -p 8080:8080 --volume "$(pwd)/source:/var/www" --volume "$(pwd)/configuration:/etc/sqlpage:ro" --rm sqlpage/SQLPage`
     - And place your website in a folder named `source` and your `sqlpage.json` in a folder named `configuration`.
- If you want to build your own docker image, taking the raw sqlpage image as a base is not recommended, since it is extremely stripped down and probably won't contain the dependencies you need. Instead, you can take debian as a base and simply copy the sqlpage binary from the official image to your own image:
  - ```Dockerfile
    FROM debian:stable-slim
    COPY --from=sqlpage/SQLPage:main /usr/local/bin/sqlpage /usr/local/bin/sqlpage
    ``` 

We provide compiled binaries only for the x86_64 architecture, but provide docker images for other architectures, including arm64 and armv7. If you want to run SQLPage on a Raspberry Pi or 
a cheaper ARM cloud instance, using the docker image is the easiest way to do it.

### On Mac OS, with homebrew

An alternative for Mac OS users is to use [SQLPage's homebrew package](https://formulae.brew.sh/formula/sqlpage).

- [Install homebrew](https://brew.sh/)
- In a terminal, run the following commands:
  - `brew install sqlpage`


### ODBC Setup

You can skip this section if you want to use one of the built-in database drivers (SQLite, PostgreSQL, MySQL, Microsoft SQL Server).

SQLPage supports ODBC connections to connect to databases that don't have native drivers, such as Oracle, Snowflake, BigQuery, IBM DB2, and many others.

ODBC support requires an ODBC driver manager and appropriate database drivers to be installed on your system.

#### Install ODBC

 - On windows, it's installed by default.
 - On linux: `sudo apt-get install -y unixodbc odbcinst unixodbc-common libodbcinst2`
 - On mac: `brew install unixodbc`


#### Install your ODBC database driver
  - [DuckDB](https://duckdb.org/docs/stable/clients/odbc/overview.html)
  - [Snowflake](https://docs.snowflake.com/en/developer-guide/odbc/odbc)
  - [BigQuery](https://cloud.google.com/bigquery/docs/reference/odbc-jdbc-drivers)
  - For other databases, follow your database's official odbc install instructions.

#### Connect to your database

 - Find your [connection string](https://www.connectionstrings.com/). It will look like this: `Driver={SnowflakeDSIIDriver};Server=xyz.snowflakecomputing.com;Database=MY_DB;Schema=PUBLIC;UID=my_user;PWD=my_password`
 - Use it in the [DATABASE_URL configuration option](./configuration.md)


## How it works

![architecture diagram](./docs/architecture-detailed.png)

SQLPage is a [web server](https://en.wikipedia.org/wiki/Web_server) written in
[rust](https://en.wikipedia.org/wiki/Rust_(programming_language))
and distributed as a single executable file.
When it receives a request to a URL ending in `.sql`, it finds the corresponding
SQL file, runs it on the database,
passing it information from the web request as SQL statement parameters.
When the database starts returning rows for the query,
SQLPage maps each piece of information in the row to a parameter
in one of its pre-defined components' templates, and streams the result back
to the user's browser.

## Examples

- [TODO list](./examples/todo%20application/): a simple todo list application, illustrating how to create a basic CRUD application with SQLPage.
- [Plots, Tables, forms, and interactivity](./examples/plots%20tables%20and%20forms/): a short well-commented demo showing how to use plots, tables, forms, and interactivity to filter data based on an URL parameter.
- [Tiny splitwise clone](./examples/splitwise): a shared expense tracker app
- [Corporate Conundrum](./examples/corporate-conundrum/): a board game implemented in SQL
- [Master-Detail Forms](./examples/master-detail-forms/): shows how to implement a simple set of forms to insert data into database tables that have a one-to-many relationship.
- [SQLPage's own official website and documentation](./examples/official-site/): The SQL source code for the project's official site, https://sql-page.com
- [Image gallery](./examples/image%20gallery%20with%20user%20uploads/): An image gallery where users can log in and upload images. Illustrates the implementation of a user authentication system using session cookies, and the handling of file uploads.
- [User Management](./examples/user-authentication/): An authentication demo with user registration, log in, log out, and confidential pages. Uses PostgreSQL.
- [Making a JSON API and integrating React components in the frontend](./examples/using%20react%20and%20other%20custom%20scripts%20and%20styles/): Shows how to integrate a react component in a SQLPage website, and how to easily build a REST API with SQLPage.
- [Handling file uploads](./examples/image%20gallery%20with%20user%20uploads): An image gallery where authenticated users can publish new images via an upload form.
- [Bulk data import from CSV files](./examples/official-site/examples/handle_csv_upload.sql) : A simple form letting users import CSV files to fill a database table.
- [Advanced authentication example using PostgreSQL stored procedures](https://github.com/mnesarco/sqlpage_auth_example)
- [Complex web application in SQLite with user management, file uploads, plots, maps, tables, menus, ...](https://github.com/DSMejantel/Ecole_inclusive)
- [Single sign-on](./examples/single%20sign%20on): An example of how to implement OAuth and OpenID Connect (OIDC) authentication in SQLPage. The demo also includes a CAS (Central Authentication Service) client.
- [Dark theme](./examples/light-dark-toggle/) : demonstrates how to let the user toggle between a light theme and a dark theme, and store the user's preference.

You can try all the examples online without installing anything on your computer using [SQLPage's online demo on replit](https://replit.com/@pimaj62145/SQLPage).

## Configuration

SQLPage can be configured through either a configuration file placed in `sqlpage/sqlpage.json`
or environment variables such as `DATABASE_URL` or `LISTEN_ON`.

For more information, read [`configuration.md`](./configuration.md).

Additionally, custom components can be created by placing [`.handlebars`](https://handlebarsjs.com/guide/)
files in `sqlpage/templates`. [Example](./sqlpage/templates/card.handlebars).

### HTTPS

SQLPage supports HTTP/2 and HTTPS natively and transparently.
Just set `SQLPAGE_HTTPS_DOMAIN=example.com`, and SQLPage
will automatically request a trusted certificate and
start encrypting all your user's traffic with it.
No tedious manual configuration for you,
and no annoying "Connection is Not Secure" messages for your users !

## Serverless

You can run SQLpage [serverless](https://en.wikipedia.org/wiki/Serverless_computing)
by compiling it to an [AWS Lambda function](https://aws.amazon.com/lambda/).
An easy way to do so is using the provided docker image:

```bash
 docker build -t sqlpage-lambda-builder . -f lambda.Dockerfile --target builder
 docker run sqlpage-lambda-builder cat deploy.zip > sqlpage-aws-lambda.zip
```

You can then just add your own SQL files to `sqlpage-aws-lambda.zip`,
and [upload it to AWS Lambda](https://docs.aws.amazon.com/lambda/latest/dg/gettingstarted-package.html#gettingstarted-package-zip),
selecting *Custom runtime on Amazon Linux 2* as a runtime.

### Hosting sql files directly inside the database

When running serverless, you can include the SQL files directly in the image that you are deploying.
But if you want to be able to update your sql files on the fly without creating a new image,
you can store the files directly inside the database, in a table that has the following structure: 

```sql
CREATE TABLE sqlpage_files(
  path VARCHAR(255) NOT NULL PRIMARY KEY,
  contents BLOB,
  last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

Make sure to update `last_modified` every time you update the contents of a file (or do it inside a TRIGGER).
SQLPage will re-parse a file from the database only when it has been modified.

## Technologies and libraries used

- [actix web](https://actix.rs/) handles HTTP requests at an incredible speed,
- [tabler](https://preview.tabler.io) handles the styling for professional-looking clean components,
- [tabler icons](https://tabler-icons.io) is a large set of icons you can select directly from your SQL,
- [handlebars](https://handlebarsjs.com/guide/) render HTML pages from readable templates for each component.

## Frequently Asked Questions

> **Why use SQL instead of a real programming language? SQL isn't even [Turing-complete](https://en.wikipedia.org/wiki/Turing_completeness)!**

- You're focusing on the wrong issue. If you can express your application declaratively, you should—whether using SQL or another language. Declarative code is often more concise, readable, easier to reason about, and easier to debug than imperative code.
- SQL is simpler than traditional languages, often readable by non-programmers, yet very powerful.
- If complexity is your goal, note that [SQL is actually Turing-complete](https://stackoverflow.com/questions/900055/is-sql-or-even-tsql-turing-complete/7580013#7580013).
- Even without recursive queries, a sequence of SQL statements driven by user interactions (like SQLPage) would still be Turing-complete, enabling you to build a SQL-powered website that functions as a Turing machine.

> **Just Because You Can Doesn’t Mean You Should...**  
— [someone being mean on Reddit](https://www.reddit.com/r/rust/comments/14qjskz/comment/jr506nx)

It's not about "should" — it's about "why not?"
Keep coloring inside the lines if you want, but we'll be over here having fun with our SQL websites.

> **Is this the same as Microsoft Access?**

The goals are similar — creating simple data-centric applications — but the tools differ significantly:
- SQLPage is a web server, not a desktop app.
- SQLPage connects to existing robust relational databases; Access tries to **be** a database.
- Access is expensive and proprietary; SQLPage is [open-source](./LICENSE.txt).
- SQLPage spares you from the torment of [Visual Basic for Applications](https://en.wikipedia.org/wiki/Visual_Basic_for_Applications).

> **Is the name a reference to Microsoft FrontPage?**

FrontPage was a visual static website builder popular in the late '90s. I hadn't heard of it until someone asked.

> **I like CSS. I want to design websites, not write SQL.**

If you want to write your own HTML and CSS,
you can [create custom components](https://sql-page.com/custom_components.sql)
by adding a [`.handlebars`](https://handlebarsjs.com/guide/) file in `sqlpage/templates` and writing your HTML and CSS there. ([Example](./sqlpage/templates/alert.handlebars)).
You can also use the `html` component to write raw HTML, or the `shell` component to include custom scripts and styles.

But SQLPage believes you shouldn't worry about button border radii until you have a working prototype.
We provide good-looking components out of the box so you can focus on your data model, and iterate quickly.

## Download

SQLPage is available for download on the from multiple sources:

[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/sqlpage/SQLPage/total?label=direct%20download)](https://github.com/sqlpage/SQLPage/releases/latest)
[![Docker Pulls](https://img.shields.io/docker/pulls/sqlpage/SQLPage?label=docker%3A%20lovasoa%2Fsqlpage)](https://hub.docker.com/r/sqlpage/SQLPage)
[![homebrew downloads](https://img.shields.io/homebrew/installs/dq/sqlpage?label=homebrew%20downloads&labelColor=%232e2a24&color=%23f9d094)](https://formulae.brew.sh/formula/sqlpage#default)
[![Scoop Version](https://img.shields.io/scoop/v/sqlpage?labelColor=%23696573&color=%23d7d4db)](https://scoop.sh/#/apps?q=sqlpage&id=305b3437817cd197058954a2f76ac1cf0e444116)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/sqlpage?label=crates.io%20download&labelColor=%23264323&color=%23f9f7ec)](https://crates.io/crates/sqlpage)
[![](https://img.shields.io/badge/Nix-pkg-rgb(126,%20185,%20227))](https://search.nixos.org/packages?channel=unstable&show=sqlpage&from=0&size=50&sort=relevance&type=packages&query=sqlpage)

## Contributing

We welcome contributions! SQLPage is built with Rust and uses
vanilla javascript for its frontend parts.

Check out our [Contributing Guide](./CONTRIBUTING.md) for detailed instructions on development setup, testing, and pull request process.

# Code signing policy

Our windows binaries are digitally signed, so they should be recognized as safe by Windows.
Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by [SignPath Foundation](https://signpath.org/). [Contributors](https://github.com/sqlpage/SQLPage/graphs/contributors), [Owners](https://github.com/orgs/sqlpage/people?query=role%3Aowner).

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it
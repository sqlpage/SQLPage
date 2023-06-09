select 'shell' as component,
    'SQLPage: get started!' as title,
    'database' as icon,
    '/' as link,
    'en-US' as lang,
    'Get started with SQLPage: short tutorial' as description,
    'documentation' as menu_item,
    20 as font_size,
    'Poppins' as font;

SELECT 'hero' as component,
    'SQLPage setup' as title,
    'Let''s create your first SQLPage website together, step by step, from downloading SQLPage to making your site available online for everyone to browse.' as description,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/c/c4/Backlit_keyboard.jpg/1024px-Backlit_keyboard.jpg' as image,
    'mailto:contact@ophir.dev' as link,
    'Instructions unclear ? Get in touch !' as link_text;

SELECT 'alert' as component,
    'Afraid of the setup ? Do it the easy way !' as title,
    'mood-happy' as icon,
    'teal' as color,
    'You don’t want to have anything to do with scary hacker things ? You can use a preconfigured SQLPage  hosted on our servers, and **never have to configure a server** yourself.' as description_md,
    'hosted.sql' AS link,
    'Try SQLPage cloud' as link_text;

select 'text' as component,
'
Download SQLPage
================

[Download the latest SQLPage](https://github.com/lovasoa/SQLpage/releases) for your operating system. SQLPage is distributed as a single executable file, making it easy to get started.

Launch your development server
==============================

Create a folder on your computer where you will store your website. Then launch the `sqlpage` executable file you just downloaded in a terminal from this folder.

You should see a message in your terminal that includes the sentence `Starting server on 0.0.0.0:8080`

You can open your website locally by visiting [`http://localhost:8080`](http://localhost:8080)

Your website’s first SQL file
=============================

In the root folder of your SQLPage website, create a new SQL file called `index.sql`. Open it in a text editor.

The `index.sql` file will be executed when you open the root of your website. You can use it to retrieve data from your database and define how it should be displayed on your website.

As an example, let''s start with a simple SQL code that displays a list of popular websites:

```sql
SELECT ''list'' AS component, ''Popular websites'' AS title;

SELECT ''Hello'' AS title, ''world'' AS description, ''https://wikipedia.org'' AS link;
```

The list of components you can use and their properties is available in [SQLPage''s online documentation](https://sql.ophir.dev/documentation.sql).

Your database schema
====================

The database schema for your SQLPage website can be defined using SQL scripts located in the **`sqlpage/migrations`** subdirectory of your website''s root folder.
Each script represents a migration that sets up or modifies the database structure.
The scripts are executed in alphabetical order, so you can prefix them with a number to control the order in which they are executed.
If you don'' want SQLPage to manage your database schema, you can ignore  the `sqlpage/migrations` folder completely,
and manually create and update database tables using your own favorite tools.

For our first website, let''s create a file located in `sqlpage/migrations/0001_create_users_table.sql` with the following contents:

```sql
CREATE TABLE users ( id INTEGER PRIMARY KEY, name TEXT NOT NULL );
```

Connect to a custom database
============================

By default, SQLPage uses a [SQLite](https://www.sqlite.org/about.html) database stored in a file named `sqlpage.db` in your website''s root folder.
You can change this by creating a file named `sqlpage/config.json` in your website''s root folder with the following contents:

```sql
{ "database_url": "sqlite://:memory:" }
```

This will tell SQLPage to use an in-memory SQLite database instead of the default file-based database.
All your data will be lost when you stop the SQLPage server, but it is useful for quickly testing and iterating on your database schema.

Later, when you want to deploy your website online, you can switch back to a persisted database like 
 - a SQLite file with `sqlite://your-database-file.db` ([see options](https://docs.rs/sqlx/0.6.3/sqlx/sqlite/struct.SqliteConnectOptions.html#main-content)),
 - PostgreSQL-compatible server with `postgres://user:password@host/database` ([see options](https://www.postgresql.org/docs/15/libpq-connect.html#id-1.7.3.8.3.6)),
 - MySQL-compatible server with `mysql://user:password@host/database` ([see options](https://dev.mysql.com/doc/refman/8.0/en/connecting-using-uri-or-key-value-pairs.html)),
 
For more information about the properties that can be set in config.json, see [SQLPage''s configuration documentation](https://github.com/lovasoa/SQLpage/blob/main/configuration.md#configuring-sqlpage)

Deploy your SQLPage website online
==================================

If you want to make your SQLPage website accessible online for everyone to browse, you can deploy it to a VPS (Virtual Private Server). To get started, sign up for a VPS provider of your choice. Some popular options include: AWS EC2, DigitalOcean, Linode, Hetzner.

Once you have signed up with a VPS provider, create a new VPS instance. The steps may vary depending on the provider, but generally, you will need to:

1. Choose the appropriate server type and specifications. SQLPage uses very few resources, so you should be fine with the cheaper options.

2. Set up SSH access.

Once your VPS instance is up and running, you can connect to it using SSH. The provider should provide you with the necessary instructions on how to connect via SSH.

For example, if you are using a Linux or macOS terminal, you can use the following command:

`ssh username@your-vps-ip-address`

### Transfer your SQLPage website files to the VPS

For example, if you are using SCP, you can run the following command from your local computer, replacing the placeholders with your own information:

`scp -r /path/to/your/sqlpage/folder username@your-vps-ip-address:/path/to/destination`

### Run sqlpage on the server

Once your SQLPage website files are on the server, you can run sqlpage on the server, just like you did on your local computer. Download the sqlpage for linux binary and upload it to your server.

Then, run the following command on your server:

`./sqlpage`

To access your website, enter the adress of your VPS in your adress bar, followed by the port on which sqlpage runs. For instance: http://123.123.123.123:8080.
' as contents_md;
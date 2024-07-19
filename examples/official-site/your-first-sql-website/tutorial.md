# Download SQLPage: the SQL website framework

SQLPage is a small executable file that will take requests to your website, execute the SQL files you write,
and render the database responses as nice web pages.

[Download the latest SQLPage](https://github.com/lovasoa/SQLpage/releases) for your operating system.
In the _release assets_ section, you will find files named `sqlpage-windows.zip`, `sqlpage-linux.tgz`, and `sqlpage-macos.tgz`.
Download the one that corresponds to your operating system, and extract the executable file from the archive.

> **Note**: Advanced users can alternatively install SQLPage using
> [docker](https://hub.docker.com/repository/docker/lovasoa/sqlpage/general),
> [brew](https://formulae.brew.sh/formula/sqlpage),
> [nix](https://search.nixos.org/packages?channel=unstable&show=sqlpage),
> [scoop](https://scoop.sh/#/apps?q=sqlpage&id=305b3437817cd197058954a2f76ac1cf0e444116),
> or [cargo](https://crates.io/crates/sqlpage).

# Building your website locally

Create a folder on your computer where you will store all contents related to your sql website.
In the rest of this tutorial, we will call this folder the **root folder** of your website.

Open the file you downloaded above, and place `sqlpage.bin` (if you are on linux or Mac OS)
or `sqlpage.exe` at the root of the folder.

Then launch the `sqlpage.bin` executable file you just downloaded in a terminal from this folder.

![screenshot for the sql website setup](first-sql-website-launch.png)

You should see a message in your terminal that includes the sentence `accessible from the network, and locally on http://localhost:8080`

You can open your website locally by visiting [`http://localhost:8080`](http://localhost:8080)

# Your website’s first SQL file

In the root folder of your SQLPage website, create a new SQL file called `index.sql`.
Open it in a text editor that supports SQL syntax highlighting (I recommend [VSCode](https://code.visualstudio.com/)).

The `index.sql` file will be executed every time a visitor opens your website's home page.
You can use it to retrieve data from your database and define how it should be displayed to your visitors.

As an example, let's start with a simple `index.sql` that displays a list of popular websites:

```sql
SELECT 'list' AS component, 'Popular websites' AS title;

SELECT 'Hello' AS title, 'world' AS description, 'https://wikipedia.org' AS link;
```

The first line of the file defines the component that will be used to display the data, and properties of that component.
In this case, we use the [`list` component](/documentation.sql?component=list#component) to display a list of items.
The second line defines the data that will populate the component.
All the components you can use and their properties are documented in [SQLPage's online documentation](https://sql.ophir.dev/documentation.sql).

# Your database schema

> If you already have a database populated with data,
> or if you intend to use other tools to manage your database structure,
> you can skip this section.

The [database schema](https://en.wikipedia.org/wiki/Database_schema) for your SQLPage website
can be defined using SQL scripts located in the
**`sqlpage/migrations`** subdirectory of your website's root folder.

For our first website, let's create a file located in `sqlpage/migrations/0001_create_users_table.sql` with the following contents:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);
```

Please read our [**introduction to database migrations**](./migrations.sql) to
learn how to maintain your database schema in the long term.

> **Note**: The migration system is not supported on Microsoft SQL Server databases.
> If you are using a SQL Server database, you should create your tables using a different tool, such as _SQL Server Management Studio_.

# Connect to a custom database

By default, SQLPage uses a [SQLite](https://www.sqlite.org/about.html) database stored in a file named `sqlpage.db`
in the `sqlpage` configuration folder.
You can change this by creating a file named `sqlpage.json` in a folder called `sqlpage`.
So, if your website's root folder is `/my_website`, you should create a file at `/my_website/sqlpage/sqlpage.json`.

Here is an example `sqlpage.json` file:

```sql
{ "database_url": "sqlite://:memory:" }
```

This will tell SQLPage to use an in-memory SQLite database instead of the default file-based database.
All your data will be lost when you stop the SQLPage server, but it is useful for quickly testing and iterating on your database schema.

Later, when you want to deploy your website online, you can switch back to a persisted database like

- a SQLite file with `sqlite://your-database-file.db` ([see options](https://docs.rs/sqlx/0.6.3/sqlx/sqlite/struct.SqliteConnectOptions.html#main-content)),
- a PostgreSQL-compatible server with `postgres://user:password@host/database` ([see options](https://www.postgresql.org/docs/15/libpq-connect.html#id-1.7.3.8.3.6)),
- a MySQL-compatible server with `mysql://user:password@host/database` ([see options](https://dev.mysql.com/doc/refman/8.0/en/connecting-using-uri-or-key-value-pairs.html)),
- a Microsoft SQL Server with `mssql://user:password@host/database` ([see options](https://docs.rs/sqlx-oldapi/latest/sqlx_oldapi/mssql/struct.MssqlConnectOptions.html), [note about named instances](https://github.com/lovasoa/SQLpage/issues/92)),

If `user` or `password` contains special characters, you should [percent-encode](https://en.wikipedia.org/wiki/Percent-encoding) them.
For instance, a SQL Server database named `db` running on `localhost` port `1433` with the username `funny:user` and the password `p@ssw0rd` would be represented as `mssql://funny%3Auser:p%40ssw0rd@localhost:1433/db`.
For more information about the properties that can be set in sqlpage.json, see [SQLPage's configuration documentation](https://github.com/lovasoa/SQLpage/blob/main/configuration.md#configuring-sqlpage)

![screenshot for the full sql website folder organisation](full-website.png)

# Use parameterized SQL queries to let users interact with your database

### Displaying a form

Let’s create a form to let our users insert data into our database. Add the following code to your `index.sql` file:

```sql
SELECT 'form' AS component, 'Add a user' AS title;
SELECT 'Username' as name, TRUE as required;
```

The snippet above uses the [`form` component](https://sql.ophir.dev/documentation.sql?component=form#component) to display a form on your website.

### Handling form submission

Nothing happens when you submit the form at the moment. Let’s fix that.
Add the following below the previous code:

```sql
INSERT INTO users (name)
SELECT :Username
WHERE :Username IS NOT NULL;
```

The snippet above uses an [`INSERT INTO SELECT` SQL statement](https://www.sqlite.org/lang_insert.html) to
[safely](../safety.sql) insert a new row into the `users` table when the form is submitted.
It uses a `WHERE` clause to make sure that the `INSERT` statement is only executed when the `:Username` parameter is present.
The `:Username` parameter is set to `NULL` when you initially load the page, and then SQLPage automatically sets it to the value
from the text field when the user submits the form.

There are two types of parameters you can use in your SQL queries:

- `:ParameterName` is a [POST](<https://en.wikipedia.org/wiki/POST_(HTTP)>) parameter. It is set to the value of the field with the corresponding `name` in a form. If no form was submitted, it is set to `NULL`.
- `$ParameterName` works the same as `:ParameterName`, but it can also be set through a [query parameter](https://en.wikipedia.org/wiki/Query_string) in the URL.
  If you add `?x=1&y=2` to the end of the URL of your page, `$x` will be set to the string `'1'` and `$y` will be set to the string `'2'`.
  If a query parameter was not provided, it is set to `NULL`.

### Displaying contents from the database

Now, users are present in our database, but we can’t see them. Let’s fix that by adding the following code to our `index.sql` file:

```sql
SELECT 'list' AS component, 'Users' AS title;
SELECT name AS title,  CONCAT(name, ' is a user on this website.') as description FROM users;
```

### Your first SQLPage website is ready!

You can view [the full source code for this example on Github](https://github.com/lovasoa/SQLpage/tree/main/examples/simple-website-example)

Here is a screenshot of the final result:

![final result](final-result.png)

To go further, have a look at [the examples section of our Github repository](https://github.com/lovasoa/SQLpage/tree/main/examples).

# Deploy your SQLPage website online

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

### Run SQLPage on the server

Once your SQLPage website files are on the server, you can run SQLPage on the server, just like you did on your local computer. Download the SQLPage for linux binary and upload it to your server.

Then, run the following command on your server:

`./sqlpage`

To access your website, enter the address of your VPS in your address bar, followed by the port on which SQLPage runs. For instance: http://123.123.123.123:8080.

For production use, you should:

- run SQLPage as a service, using a tool like [docker](https://docs.docker.com/engine/reference/run/) or [systemd](https://linuxhandbook.com/create-systemd-services/).
- [use a reverse proxy like _nginx_](./nginx.sql) to improve security and performance, or to configure multiple websites on the same server.

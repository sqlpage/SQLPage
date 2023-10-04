INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES (
        'SQLPage v0.12',
        'New release! SQLPage v0.12 brings the sqlpage.exec function, variables, and more.',
        'refresh',
        '2023-10-05',
        '
# SQLPage Version 0.12.0 was just released 🚀

[SQLPage](https://sql.ophir.dev) is a small web server that renders your SQL queries as beautiful interactive websites. This releases brings exciting features that should make development even easier, faster, and more secure. Let''s dive into the exciting innovations of version 0.12.0:

## 🧮 **Variable Support**

SQLPage now empowers you with the ability to set and reuse variables between SQL statements. This dynamic feature allows you to craft more complex SQL queries and reuse query results across multiple places in your code. Here''s a sneak peek:

```sql
-- Set a variable 
SET person = (SELECT username FROM users WHERE id = $id); 
-- Use it in a query 
SELECT ''text'' AS component, ''Hello '' || $person AS contents;
```


## 🚀  Execute Server-Side Commands with `sqlpage.exec`

Introducing ```sqlpage.exec```—a powerful function that lets you execute commands on the server. This opens up a world of possibilities, from making external API calls to sending emails and running custom code on the server. Be creative, but remember that with great power comes great responsibility !

```sql
SELECT ''card'' AS component;
SELECT 
    value->>''name'' AS title, 
    value->>''email'' AS description 
FROM json_each(sqlpage.exec(''curl'', ''https://jsonplaceholder.typicode.com/users''));
```

### 🛡️ **Security**

For your security, the ```sqlpage.exec``` function is disabled by default. To enable it, simply set the ```allow_exec``` configuration parameter to ```true``` in the [configuration](./configuration.md). Please use caution, as enabling this function grants significant server access to anyone who can write SQL queries on your website.


## 🍔 **Menu Items Made Easy**

Configuring multiple menu items has never been simpler. Now, syntax like

```sql
SELECT ''shell'' AS component, ''["page 1", "page 2"]'' AS menu_item
```
works as expected. See 
 - the [shell component documentation](https://sql.ophir.dev/documentation.sql?component=shell#component) 
 - [the small SQL game example](./examples/corporate-conundrum/) 
 - and join the discussion [here](https://github.com/lovasoa/SQLpage/discussions/91).

## 🛠️ **Database Connection Setup**: `on_connect.sql`

Create the ```sqlpage/on_connect.sql``` file to run a SQL script on each database connection before it''s used. This versatile feature enables you to customize your database connection with settings like ```PRAGMA``` in SQLite, custom variables in PostgreSQL, and more. Explore the endless possibilities!

## 🚧 **Improved Error Handling**

Experience more precise and informative error messages with SQLPage. When an error occurs, you''ll now receive detailed error positions within the SQL statement. Say goodbye to vague error messages and welcome efficient debugging.


## 📟 ARM: **Hello, Raspberry Pi and Mac M1 Users!**

SQLPage now distributes Docker images for ARM architecture, expanding your possibilities for deployment. Whether you''re using a Raspberry Pi or a Mac M1, SQLPage is ready to power your projects!

## 🔒 **Enhanced Security by Default**

To enhance security, SQLPage now creates the default SQLite database file in the "sqlpage" config directory, making it inaccessible from the web by default. For those who prefer the previous behavior, simply set the ```database_url``` configuration parameter to ```sqlite://sqlpage.db``` in your [configuration](./configuration.md).

## 📜 **Empty List Customization**

Tailor your list components with precision using the new ```empty_title```, ```empty_description```, and ```empty_link``` top-level attributes in the [`list`](https://sql.ophir.dev/documentation.sql?component=list#component) component. Now you have full control over the text displayed when your list is empty.


## 🔒 **Asynchronous Password Hashing** for Enhanced Performance and Security

Say goodbye to request processing bottlenecks! SQLPage used to block a request processing thread while hashing passwords, potentially leaving your site vulnerable to denial of service attacks. Not anymore! SQLPage now launches password hashing operations on a separate thread pool, allowing your application to handle other requests while efficiently hashing passwords.

## 🔗 **URL Parameter Encoding**

Introducing ```sqlpage.url_encode```! This function simplifies URL parameter encoding, making it a breeze to create dynamic URLs in your web application.

```sql
SELECT ''card'' AS component; SELECT ''More...'' AS title, ''advanced_search.sql?query='' || sqlpage.url_encode($query)
```

## Upgrade

[Download SQLPage](https://github.com/lovasoa/SQLpage/releases) | [GitHub Repository](https://github.com/lovasoa/sqlpage)

Unleash your creativity, streamline your development, and craft extraordinary SQL-driven web applications with SQLPage 0.12.0. Happy coding! 💻🚀
');
select 'shell' as component,
    'SQLPage safety' as title,
    'shield-check-filled' as icon,
    '/' as link,
    'en-US' as lang,
    'SQLPage security guarantees' as description,
    'documentation' as menu_item,
    20 as font_size,
    'Poppins' as font;

select 'text' as component,
    '
# SQLPage''s security guarantees

SQLPage is a tool that allows you to create a full website using only SQL queries, and render results straight from the database to the browser.
Most programmers, hearing this, will immediately think of the security implications of this model.

This page is here to provide a list of the security guarantees that SQLPage provides.
SQLPage was designed from the ground up to be usable by non-technical *data analysts* and other non-web-developers,
so it provides safe defaults everywhere, so that you don''t have to worry about inadvertently 
exposing more data than you intended.


## Protection against SQL injections

SQL injections are a common security vulnerability in traditional back-end web development,
that allow an attacker to execute arbitrary SQL code on your database.

**SQLPage is immune to SQL injections**, because it uses [prepared statements](https://en.wikipedia.org/wiki/Prepared_statement)
to pass parameters to your SQL queries.

When a web page starts rendering, and before processing any user inputs, all your SQL queries have already been prepared, and no 
new SQL code can be passed to the database. Whatever evil inputs a user might try to pass to your website,
it will never be executed as SQL code on the database.

SQLPage **cannot** execute any other SQL code than the one you, the site author, wrote in your SQL files.

If you have a SQL query that looks like this:

```sql
SELECT * FROM users WHERE userid = $id;
```

and a user tries to pass the following value to the `id` parameter:

```
1; DROP TABLE users;
```

SQLPage will execute the search for the user with id `1; DROP TABLE users;` (and most likely not find any user with that id),
but it *will not* execute the `DROP TABLE` statement.

## Protection against XSS attacks

XSS attacks are a common security vulnerability in traditional front-end web development,
that allow an attacker to execute arbitrary JavaScript code on your users'' browsers.

**SQLPage is immune to XSS attacks**, because it uses an HTML-aware templating engine to render your SQL results to HTML.
When you execute the following SQL code:

```sql
SELECT ''text'' AS component, ''<script>alert("I am evil")</script>'' AS contents;
```

it will be rendered as:

```html
<p>
    &lt;script&gt;alert("I am evil")&lt;/script&gt;
</p>
```

Additionnally, SQLPage uses a [Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
that disallows the execution of any inline JavaScript code, and only allows loading JavaScript code from trusted sources.

If you have some legitimate JavaScript code that you want to execute on your website, you can use the `javascript`
parameter of the [`shell`](documentation.sql?component=shell#component) component to do so.

## Database connections

SQLPage uses a fixed pool of database connections, and will never open more connections than the ones you
[configured](https://github.com/lovasoa/SQLpage/blob/main/configuration.md). So even under heavy load, your database
connection limit should never be saturated by SQLPage.

And SQLPage will accept any restriction you put on the database user you use to connect to your database, so you can
create a specific user for SQLPage that only has access to the specific tables you will use in your application.

If your entire application is read-only, you can even create a user that only has the `SELECT` privilege on your database,

' as contents_md;
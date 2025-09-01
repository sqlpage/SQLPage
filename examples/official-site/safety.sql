select 'http_header' as component, 
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
    '<https://sql-page.com/safety>; rel="canonical"' as "Link";

select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', 'Security in SQLPage: SSO, protection against SQLi, XSS, CSRF, and more'
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

select 'hero' as component,
    'SQLPage''s security guarantees' as title,
    'SQLPage prevents common web vulnerabilities such as SQL injections and XSS attacks by default.' as description,
    'safety.webp' as image;

select 'text' as component,
    '
SQLPage is a tool that allows you to create a full website using only SQL queries, and render results straight from the database to the browser.
Most programmers, hearing this, will immediately think of the security implications of this model.

This page is here to provide a list of the security guarantees that SQLPage provides.
SQLPage was designed from the ground up to be usable by non-technical *data analysts* and other non-web-developers,
so it provides safe defaults everywhere, so that you don''t have to think about basic security issues
you would have to worry about in a traditional web development stack.

## SQLPage does not expose your database to the internet

SQLPage websites are *server-side rendered*, which means that the SQL queries stay on the server
where SQLPage is installed.

The results of these queries are then rendered to HTML, and sent to the user''s browser.
A malicious user cannot run arbitrary SQL queries on your database, because SQLPage
does not expose your entire database to the internet, only the results of
your prepared queries, rendered as web pages.

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

**SQLPage is immune to XSS attacks**, because it uses an HTML-aware templating engine to render your
SQL query results to HTML. When you execute the following SQL code:

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

## Authentication

Use either the built-in username/password or Single Sign-On; both follow safe defaults.

### Built-in username/password

SQLPage provides an [authentication](/documentation.sql?component=authentication#component) component to protect pages,
with helpers like [`sqlpage.basic_auth_username()`](/functions.sql?function=basic_auth_username#function),
[`sqlpage.basic_auth_password()`](/functions.sql?function=basic_auth_password#function), and
[`sqlpage.hash_password()`](/functions.sql?function=hash_password#function).
Passwords are salted and hashed with [argon2](https://en.wikipedia.org/wiki/Argon2),
following [best practices](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html).

### Session management

If you implement your own sessions using the [`cookie` component](/documentation.sql?component=cookie#component),
follow the [OWASP recommendations](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#cookies).
Avoid rolling your own unless you fully understand web security.

### Single Sign-On (OIDC)

When OIDC is enabled, SQLPage validates a signed identity token on every request
before any of your SQL runs. Without a successful login, requests are redirected
to your identity provider and your application code never executes.
This keeps attackers outside your SSO realm from reaching your app,
even if a vulnerability exists in your own code.

By default, all pages are protected when single sign-on is enabled.
Once authenticated, you can access user claims with
[`sqlpage.user_info()`](/functions.sql?function=user_info)
to further restrict what users see based on who they are.

## Protection against [CSRF attacks](https://en.wikipedia.org/wiki/Cross-site_request_forgery)

The recommended way to store session tokens for user authentication
in SQLPage is to use the [`cookie` component](/documentation.sql?component=cookie#component).

All cookies set by SQLPage have the `SameSite` attribute set to `strict` by default,
which means that they will only be sent to your website if the user is already on your website.
An attacker cannot make a user''s browser send a request to your website from another (malicious) 
website, and have it perform an action on your website in the user''s name,
because the browser will not send the cookies to your website.

SQLPage differentiates between POST variables (accessed with the `:variable` syntax), and 
variables that can come from URL parameters (accessible with `$variable`). Note that URL parameters
prefixed with `_sqlpage_` are reserved for internal use.

When a user submits a form, you should use POST variables to access the form data.
This ensures that you only use data that indeed comes from the form, and not from a
URL parameter that could be part of a malicious link.

Advanced users who may want to implement their own csrf protection system can do so
using the [`sqlpage.random_string()`](/functions.sql?function=random_string#function) function,
and the `hidden` input type of the [`form`](/documentation.sql?component=form#component) component.

For more information, see the [this discussion](https://github.com/sqlpage/SQLPage/discussions/148).

## Database connections

SQLPage uses a fixed pool of database connections, and will never open more connections than the ones you
[configured](https://github.com/sqlpage/SQLPage/blob/main/configuration.md). So even under heavy load, your database
connection limit will never be saturated by SQLPage.

And SQLPage will accept any restriction you put on the database user you use to connect to your database, so you can
create a specific user for SQLPage that only has access to the specific tables you will use in your application.

If your entire application is read-only, you can even create a user that only has the `SELECT` privilege on your database,
preventing any accidental data modification. SQLPage will work fine with such a user and will never try to execute any
other SQL statements than the ones you explicitly wrote in your SQL files.
' as contents_md;

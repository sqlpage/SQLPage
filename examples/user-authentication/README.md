# User authentication demo

This example demonstrates how to handle user authentication with SQLpage.

It uses a PostgreSQL database to store user information and session ids,
but the same principles can be applied to other databases.

All the user and password management is done in SQLPage,
which uses [best practices](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#maximum-password-lengths) for password storage.

This demonstrates how to implement:
 - [a signup form](./signup.sql)
 - [a login form](./signin.sql)
 - [a logout button](./logout.sql)
 - [secured pages](./protected_page.sql) that can only be accessed by logged-in users

User authentication is a complex topic, and you can follow the work on implementing differenet authentication methods in [this issue](https://github.com/lovasoa/SQLpage/issues/12).

## How to run

Install [docker](https://docs.docker.com/get-docker/) and [docker-compose](https://docs.docker.com/compose/install/).

Then run the following command in this directory:

```bash
docker-compose up
```

Then open [http://localhost:8080](http://localhost:8080) in your browser.

## Caveats

In this example, we handle user creation and login in SQLpage.

If you are implementing user authentication in an public application with potentially sensitive data,
you should propably read the [Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html) from OWASP.

## Screenshots

| Signup form | Login form | Protected page |
| --- | --- | --- |
| ![signup form](./screenshots/signup.png) | ![login form](./screenshots/signin.png) | ![protected page](./screenshots/secret.png) |
| ![home](./screenshots/homepage.png) | ![duplicate username](./screenshots/duplicate-user.png) | ![signup success](./screenshots/signup-success.png) |

## How it works

### User creation

The [signup form](./signup.sql) is a simple form that is handled by [`create_user.sql`](./create_user.sql).
You could restrict user creation to existing administrators and create an initial administrator in a database migration.

### User login

The [login form](./signin.sql) is a simple form that is handled by [`login.sql`](./login.sql).

`login.sql` checks that the username exists and that the password is correct using the [authentication component](https://sql.datapage.app/documentation.sql?component=authentication#component) extension with

```sql
SELECT 'authentication' AS component,
    'signin.sql' AS link,
    (SELECT password_hash FROM user_info WHERE username = :username) AS password_hash,
    :password AS password;
```

If the login is successful, an entry is added to the [`login_session`](./sqlpage/migrations/0000_init.sql) table with a random session id.

If it is not, the authentication component will redirect the user to the login page and stop the execution of the page.

The session id is then stored in a cookie on the user's browser.

The user is then redirected to [`./protected_page.sql`](./protected_page.sql) which will check that the user is logged in.

### Protected pages

Protected pages are pages that can only be accessed by logged-in users.

There is an example in [`protected_page.sql`](./protected_page.sql) that uses
the [`redirect`](https://sql.datapage.app/documentation.sql?component=redirect#component)
component to redirect the user to the login page if they are not logged in.

Checking whether the user is logged in is as simple as checking that session id returned by [`sqlpage.cookie('session')`](https://sql.datapage.app/functions.sql?function=cookie#function) exists in the [`login_session`](./sqlpage/migrations/0000_init.sql) table.


### User logout

The cookie can be deleted in the browser by navigating to [`./logout.sql`](./logout.sql).
select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, '

# Password Hashing

In SQLPage, you can use the [`sqlpage.hash_password`](/functions.sql?function=hash_password) function to
create a sequence of letters and numbers that can be used to verify
a password, but cannot be used to recover the password itself.
This is called a [hash](https://en.wikipedia.org/wiki/Hash_function) of the password,
and it is a common way to store passwords in a database.
This way, even if someone gains access to the database, they cannot
recover the passwords.

They could still try to guess the passwords, but since SQLPage
uses the [argon2](https://en.wikipedia.org/wiki/Argon2) algorithm,
it would take a very long time (hundreds of years) to guess a strong password.

The `sqlpage.hash_password` function takes a password as input, and
returns a hash of the password as output. It takes some time
(a few hundred milliseconds) to compute the hash, so you should
only call it when the user is creating a new account and on the initial
login. You should not call it on every page load.

When you have logged in an user using the 
[`authentication`](/documentation.sql?component=authentication#component) component,
you can store their session identifier on their browser using the
[`cookie`](/documentation.sql?component=cookie#component) component.

## Example

 - [Source code for this page](https://github.com/lovasoa/SQLpage/blob/main/examples/official-site/examples/hash_password.sql)
 - [Full user authentication and session management example](https://github.com/lovasoa/SQLpage/blob/main/examples/user-authentication)

# Try it out

You can try the password hashing function out by entering a password
below and clicking "Hash Password".
' as contents_md;

select 'form' as component, 'Hash Password' as validate;
select 'password' as type, 'password' as name, 'Password' as label, 'Enter a password to hash' as placeholder;

select 'text' as component, '

### Hashed Password

The password you entered above hashed to the following value:

```
' || sqlpage.hash_password(:password) || '
```
' as contents_md
where :password is not null;
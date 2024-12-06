
INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'SQLPage website update',
        'Performance, security, and new features',
        'browser',
        '2024-05-01',
        '
Today is may day, and we are happy to announce that the SQLPage website has been updated with new contents.

## Homepage

The [homepage](/) has been updated to include prominent information about commonly asked questions,
such as the [security guarantees](/safety.sql) of SQLPage, and the [performance](/performance.sql) of SQLPage applications.

## Performance of SQLPage applications

We now have a [detailled explanation of the performance of SQLPage applications](/performance.sql) on the website.
It explains why and how SQLPage applications are often faster than equivalent applications written in other frameworks.

## Single-Sign-On

Since SQLPage v0.20.3, SQLPage can natively make requests to external HTTP APIs with [the `fetch` function](/documentation.sql#fetch),
which opens the door to many new possibilities.

An example of this is the [**SSO demo**](https://github.com/sqlpage/SQLPage/tree/main/examples/single%20sign%20on),
which demonstrates how to use SQLPage to authenticate users on a website using a third-party authentication service,
such as Google, Facebook, an enterprise identity provider using [OIDC](https://openid.net/connect/),
or an academic institution using [CAS](https://apereo.github.io/cas/).

## New architecture diagram

The README of the SQLPage repository now includes a
[clear yet detailed architecture diagram](https://github.com/sqlpage/SQLPage?tab=readme-ov-file#how-it-works).
'
    );
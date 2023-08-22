INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES (
        'I’m sorry I forked you',
        'SQLPage forked the sqlx database drivers library. Here is why.',
        'git-fork',
        '2023-08-13',
        '
# I’m sorry I forked you

I’ve been immersed in open source coding since my teenage years, and now, I can’t fathom a world without it.
Throughout my career as a computer engineer, I’ve yet to encounter a tech company that isn’t built on the foundation of free and open source software.
Each company adds its proprietary touch to this vast open source landscape,
but they are ultimately just forming a unique blend atop a colossal iceberg of shared resources.
In the software world, open source truly is the driving force behind innovation.

## Unconventional Financial Currents in Software 

The software industry operates on a financial current that defies conventional norms.
Unlike other sectors where key players like oil companies rake in the riches by supplying essentials to other businesses,
the software realm flips the script.
In this landscape, it’s user-facing giants like Google that reap the profits,
while the very creators crafting the software forming the bedrock for Google and countless others often find themselves on a different end of the economic spectrum.


Open source developers observe this intriguing dynamic,
sometimes even finding satisfaction in witnessing how their freely contributed software
fuels the creation of multimillion-dollar ventures.

## SQLx: A Rust Marvel

[sqlx](https://crates.io/crates/sqlx) is one of the numerous software libraries
that lie at the foundation of this software iceberg.
It’s a formidable SQL database driver for the *Rust* programming language,
that harmonizes connection to a multitude of databases.
It garners approximately 20,000 daily downloads.

### Version 0.7

sqlx’s main maintainer sought to find a middle ground – crafting good open source software while seeking a sustainable livelihood.
This endeavor led to a pivotal decision: extracting the database drivers from the core library.
While retaining most drivers as open source, **compatibility with Microsoft SQL Server was relinquished**.
This significant architectural shift also necessitated the removal of some other features from the core framework,
and the introduction of a new API, making it non trivial to migrate from the previous version.

## SQLPage

As the principal caretaker of the [SQLPage web application server](/), which relies on sqlx,
I faced a pivotal juncture. The path ahead diverged into two distinct trails: 
1. a challenging migration to sqlx v0.7, making a cross on MSSQL support;
2. persisting with v0.6, a realm housing outdated and potentially vulnerable dependencies.

After a lot of hesitation I chose a third path: **forking sqlx**.

## I’m sorry I forked you

I’m sorry I forked you, sqlx. I really am all for financially sustainable open source.
My hope is that the newfound proprietary drivers find success,
duly compensating *@abonander* for the invaluable contributions made.

But I really need a good fully open source set of database drivers for Rust,
I need some of the features that were removed in v0.7, and most importantly,
I want to support SQL Server in SQLPage.
So I created [sqlx-oldapi](https://lib.rs/crates/sqlx-oldapi), a fork of sqlx v0.6.

In the fork:
 - I’ve meticulously updated all dependencies to their latest iterations, ensuring the foundation remains robust and secure.
 - Essential features that were missing have been thoughtfully incorporated to address specific needs, and longstanding bugs have been resolved. Notably, data type support has been fortified, with efficient lossy decoding of `DECIMAL` values as floats across all drivers.
 - My endeavors have been focused on elevating the SQL Server driver to the same level as its peers. This involved fixing bugs and crashes, and supporting new data types like `DATETIME` and `DECIMAL`.

The full list of changes can be found in the [changelog](https://github.com/lovasoa/sqlx/blob/main/CHANGELOG.md).

## Concluding Notes

 - My best wishes extend to sqlx on their pursuit of successful monetization.
   May their path be paved with prosperity as they navigate this new chapter.
 - To fellow developers facing a similar crossroads as I did with SQLPage,
   know that [sqlx-oldapi](https://lib.rs/crates/sqlx-oldapi) awaits you, ready to empower your endeavors, for free.
   Contributions and bug reports are all welcomed [on github](https://github.com/lovasoa/sqlx-oldapi).

And if you are curious about why this page’s URL ends in `.sql`, check out [SQLPage](/).

---

**Important addendum**:
The main contributor to sqlx reacted to this post, and they wanted to clarify two things:

 - sqlx is a project of the *Launchbase* company, not a personal project of *@abonander*.
   Although he is the main contributor, important decisions are taken in consultation with the company.
   If new drivers are monetized, the money will go to the company, not to him personally.
   The stated goal is to then allocate the money to the development of sqlx.
 - Most importantly: the current plan for the new drivers **is not to release them as proprietary**
  code as initially planned, but to release them as open source, under the more restrictive *AGPL* license.
  This means that they will be free to use for similarly licensed open source projects
  (unlike SQLPage, which is free to use even in conjunction with proprietary software, under the *MIT* license).
  Companies that want to use these future sqlx drivers in proprietary software will have to pay for a commercial license.
  That change in the decision has been announced in 2022, and I should have been aware of it when I wrote this post. My bad.
');
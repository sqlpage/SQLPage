# Understanding SQL Migrations: Your Database, Layer by Layer

Maintaining a structured and evolving database is crucial for web app development. Rarely do we get a schema 100% correct on day one. New insights about the shape of the application are discovered over time, or business needs themselves evolve. In the world of databases, we can evolve schemas using migration files. These files are just more SQL that append or amend layers of development. Think of this process like sedimentary rock layers. Each migration adds a layer, and together, these layers create a complete, functional structure along with a visible trail of historical changes.

## What Makes up a SQL Migration File?

SQL migrations are incremental changes to a database. These changes can include creating tables, adding columns, modifying data types, or even inserting or updating records. Each migration is a distinct script that applies a specific change.

### Use Caution!

Since migration files change the database, they can have unintended consequences if not thought through carefully. For instance, you may accidentally delete a column that is still being used by your application or remove records that are still needed.

> ⚠️ Be thoughtful, double-check your work, and **always back up your data before running a migration**.

### Order Matters

**It's important that migrations are distinct ordered files** as SQLPage uses the sequence of migration files to build the database over time: `0001_initial_setup.sql`, `0002_my_first_change.sql`, `0003_my_next_change`, etc.

### No take-backs!

**Do not make changes to an existing migration file** in production. If a previously implemented migration file is altered, it will confuse SQLPage and cause a crash.

*If you are in early stages of development and are okay with losing data*, you can delete the database and start over with an altered migration file. However, in a production environment, especially once persisted data is involved, this is not an option.

It's like trying to go back in time and change a previous sedimentary layer. That's not how rocks work, and that's not how migrations work.

Append or amend; do not try to change the past.

## Examples

Let's start off easy with a simple database to store user information: `first_name`, `last_name`, `email`, `phone`, and `password_hash`. Our first migration actually creates the `users` table with these columns. That is, we migrate from *nothing* to *having a users table*.

**`sqlpage/migrations/001_create_users.sql`**:
```sql
create table users (
	id integer primary key autoincrement,
	first_name not null,
	last_name not null,
	email not null unique,
	phone,
	rewards_level,
	password_hash not null
);
```

In the terminal, we can see the new schema:

```console
sqlite> .schema
CREATE TABLE users (
	id integer primary key autoincrement,
	first_name not null,
	last_name not null,
	email not null unique,
	phone,
	rewards_level,
	password_hash not null
);
```

### A Simple Change

Later, we discover we need a `middle_name` column, so we create a new migration file to add this column to the `users` table. Remember, we must ensure the order is written into the filename so SQLPage can apply them in the correct order when building the database.

**`sqlpage/migrations/002_add_middle_name.sql`**:
```sql
alter table users add column middle_name;
```

In the terminal:

```console
sqlite> .schema
CREATE TABLE users (
	id integer primary key autoincrement,
	first_name not null,
	last_name not null,
	email not null unique,
	phone,
	rewards_level integer,
	password_hash not null,
	middle_name
);
```

### A More Complex Change

But notice here, SQLite has appended the column to the very end. What if we really need that `middle_name` column to be next to the other name columns? Further, what if we realize `rewards_level` should really be an integer and only one between 1 and 20?

We can create a new migration file to make these changes, albeit a bit more complicated.

Because we'll be altering column types and modifying column order, we'll need to *create a temporary table* to hold the data while we drop the original table and recreate it with the new schema.

**`sqlpage/migrations/003_alter_users.sql`**:
```sql
create table users_temp (
	id integer primary key autoincrement,
	first_name not null,
	last_name not null,
	middle_name,
	email not null unique,
	phone,
	rewards_level integer check(rewards_level between 1 and 20),
	password_hash not null
);

insert into users_temp select id, first_name, last_name, middle_name, email, phone, rewards_level, password_hash from users;

drop table users; -- backups are important!

alter table users_temp rename to users;
```

In the terminal:

```console
sqlite> .schema
CREATE TABLE users (
	id integer primary key autoincrement,
	first_name not null,
	last_name not null,
	middle_name,
	email not null unique,
	phone,
	rewards_level integer check(rewards_level between 1 and 20),
	password_hash not null
);
```

## Conclusion

SQL migration is our tool for evolving databases over time. By creating distinct, ordered migration files, we can incrementally build and modify our databases without losing data or breaking our application functionality. Just **remember to always back up data before running a migration**, and always be thoughtful about changes.

Since SQLPage runs migrations forward in time, we won't dive into the complexities of rolling back migrations here. Just remember, we can't change the past, only build upon it.

[Rollbacks](https://en.wikipedia.org/wiki/Rollback_(data_management)) are an intriguing topic that you may run into in other frameworks.

## Further Study

To learn more on the migrations topic, consider the Wikipedia article on [Schema Migration](https://en.wikipedia.org/wiki/Schema_migration). **Note**: database engines are different, so be sure to review the documentation for your specific database engine and what types of SQL statements are permitted. For SQLite, the [official documentation](https://www.sqlite.org/lang_altertable.html) is a good place to start.

Best migrations on your evolving database journey! 👋

---

Article written by [Matthew Larkin](https://github.com/matthewlarkin) for [SQLPage](https://sql-page.com/).
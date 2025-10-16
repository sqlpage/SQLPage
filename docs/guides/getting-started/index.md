---
title: Getting Started with SQLPage
difficulty: beginner
estimated_time: 10 minutes
categories: [tutorial, basics]
tags: [installation, setup, first-steps]
prerequisites: []
next: [user-authentication, form-handling]
---

# Getting Started with SQLPage

Welcome to SQLPage! This guide will help you create your first SQLPage application in just a few minutes.

## What is SQLPage?

SQLPage is a web framework that lets you build dynamic websites using only SQL. Instead of writing complex server-side code, you write SQL queries that return data in a structured format, and SQLPage automatically renders them as beautiful web pages.

## Installation

### Option 1: Download Binary

1. Go to the [releases page](https://github.com/lovasoa/SQLPage/releases)
2. Download the binary for your operating system
3. Make it executable: `chmod +x sqlpage`
4. Run it: `./sqlpage`

### Option 2: Using Cargo

```bash
cargo install sqlpage
```

### Option 3: From Source

```bash
git clone https://github.com/lovasoa/SQLPage.git
cd SQLPage
cargo build --release
```

## Your First Page

Create a file called `index.sql`:

```sql
SELECT 'text' AS component, 'Hello, World!' AS contents;
```

Now run SQLPage:

```bash
sqlpage
```

Open your browser to `http://localhost:8080` and you should see "Hello, World!" displayed on the page.

## Adding Components

Let's make it more interesting by adding some components:

```sql
-- Page title
SELECT 'text' AS component, 'My First SQLPage App' AS contents, 'h1' AS level;

-- Some text
SELECT 'text' AS component, 'Welcome to SQLPage! This page was created using only SQL.' AS contents;

-- A button
SELECT 'button' AS component, 'Click me!' AS contents, 'https://sqlpage.com' AS link;
```

## Working with Data

SQLPage really shines when you work with databases. Let's create a simple data table:

```sql
-- Create a table (in a real app, you'd do this in a migration)
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL
);

-- Insert some sample data
INSERT OR IGNORE INTO users (name, email) VALUES 
    ('Alice', 'alice@example.com'),
    ('Bob', 'bob@example.com'),
    ('Charlie', 'charlie@example.com');

-- Display the data
SELECT 'table' AS component, 'Users' AS title;
SELECT name, email FROM users;
```

## Next Steps

Now that you have a basic understanding of SQLPage, you can:

1. Learn about [user authentication](../user-authentication.md) to add login functionality
2. Explore [form handling](../form-handling.md) to collect user input
3. Check out the [component reference](../../components/) for all available UI components
4. Browse the [function reference](../../functions/) for built-in functions

## Troubleshooting

### Common Issues

**Port already in use**: If port 8080 is busy, specify a different port:
```bash
sqlpage --port 3000
```

**Database not found**: Make sure you're running SQLPage from the directory containing your `.sql` files.

**Permission denied**: Make sure the SQLPage binary is executable:
```bash
chmod +x sqlpage
```

## Getting Help

- Check the [documentation](../../)
- Join our [community discussions](https://github.com/lovasoa/SQLPage/discussions)
- Report issues on [GitHub](https://github.com/lovasoa/SQLPage/issues)
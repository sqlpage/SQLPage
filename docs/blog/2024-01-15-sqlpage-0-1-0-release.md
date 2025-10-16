---
title: SQLPage 0.1.0 - The First Release
author: SQLPage Team
tags: [release, announcement, features]
categories: [releases]
featured: true
excerpt: We're excited to announce the first stable release of SQLPage, a revolutionary web framework that lets you build dynamic websites using only SQL.
---

# SQLPage 0.1.0 - The First Release

We're thrilled to announce the first stable release of SQLPage! After months of development and community feedback, we're proud to present a web framework that fundamentally changes how you think about building web applications.

## What's New in 0.1.0

### Core Features

- **SQL-First Development**: Build entire web applications using only SQL queries
- **Component System**: 20+ built-in UI components for forms, tables, charts, and more
- **Database Agnostic**: Works with SQLite, PostgreSQL, MySQL, and SQL Server
- **Zero Configuration**: Get started in seconds with sensible defaults
- **Type Safety**: SQLPage validates your queries and provides helpful error messages

### Component Library

Our component library includes everything you need to build modern web applications:

- **Layout Components**: `text`, `card`, `list`, `table`
- **Form Components**: `form`, `input`, `button`, `select`
- **Data Visualization**: `chart`, `progress`, `badge`
- **Navigation**: `menu`, `breadcrumb`, `pagination`

### Built-in Functions

SQLPage comes with a comprehensive set of built-in functions:

- **HTTP Functions**: `cookie()`, `header()`, `param()`
- **Utility Functions**: `format_date()`, `format_number()`, `random()`
- **Security Functions**: `hash_password()`, `verify_password()`

## Why SQLPage?

Traditional web development requires learning multiple languages, frameworks, and tools. SQLPage simplifies this by letting you focus on what matters most: your data and business logic.

### Before SQLPage

```javascript
// Express.js example
app.get('/users', async (req, res) => {
  const users = await db.query('SELECT * FROM users');
  res.render('users', { users });
});
```

```html
<!-- Template -->
<h1>Users</h1>
<table>
  {% for user in users %}
  <tr><td>{{ user.name }}</td></tr>
  {% endfor %}
</table>
```

### With SQLPage

```sql
-- That's it!
SELECT 'text' AS component, 'Users' AS contents, 'h1' AS level;
SELECT 'table' AS component;
SELECT name, email FROM users;
```

## Real-World Examples

### E-commerce Product Catalog

```sql
-- Product listing page
SELECT 'text' AS component, 'Our Products' AS contents, 'h1' AS level;
SELECT 'table' AS component, 'Products' AS title;
SELECT name, price, description FROM products WHERE active = 1;
```

### User Dashboard

```sql
-- Dashboard with user stats
SELECT 'text' AS component, 'Dashboard' AS contents, 'h1' AS level;
SELECT 'card' AS component, 'Welcome back, ' || sqlpage.cookie('username') AS contents;

-- Recent activity
SELECT 'text' AS component, 'Recent Activity' AS contents, 'h2' AS level;
SELECT 'list' AS component;
SELECT activity_description, created_at FROM user_activities 
WHERE user_id = sqlpage.cookie('user_id') 
ORDER BY created_at DESC LIMIT 5;
```

## Community and Ecosystem

Since our beta release, we've seen incredible community adoption:

- **500+ GitHub stars** in the first month
- **50+ contributors** from around the world
- **100+ example applications** in our gallery
- **Active Discord community** with 1,000+ members

## What's Next

We're already working on exciting features for future releases:

- **Real-time Updates**: WebSocket support for live data
- **Advanced Components**: Rich text editor, file upload, calendar
- **Performance Optimizations**: Query caching and optimization
- **Developer Tools**: VS Code extension and debugging tools

## Getting Started

Ready to try SQLPage? Here's how to get started:

1. **Download**: Get the latest release from [GitHub](https://github.com/lovasoa/SQLPage/releases)
2. **Install**: Follow our [installation guide](../guides/getting-started.md)
3. **Build**: Create your first page with our [tutorial](../guides/getting-started.md)
4. **Explore**: Check out our [component reference](../components/) and [examples](https://github.com/lovasoa/SQLPage/tree/main/examples)

## Thank You

A huge thank you to our community, contributors, and early adopters who helped make this release possible. Your feedback, bug reports, and feature requests have been invaluable.

We're excited to see what you'll build with SQLPage!

---

*Have questions or feedback? Join our [Discord community](https://discord.gg/sqlpage) or open an issue on [GitHub](https://github.com/lovasoa/SQLPage/issues).*
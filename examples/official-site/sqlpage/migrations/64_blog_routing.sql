
INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'File-based routing in SQLPage',
        'Understanding how SQLPage maps URLs to files and handles errors',
        'route',
        '2025-07-28',
        '
SQLPage uses a simple file-based routing system that maps URLs directly to SQL files in your project directory.
No complex configuration is needed. Just create files and they become accessible endpoints.

This guide explains how SQLPage resolves URLs, handles different file types, and manages 404 errors so you can structure your application effectively.

## How SQLPage Routes Requests

### 1. Site Prefix Handling

If you''ve configured a [`site_prefix`](/your-first-sql-website/nginx) in your settings,
SQLPage will redirect all requests that do not start with the prefix to `/<site_prefix>`.

### 2. Path Resolution Priority

**Directory requests (paths ending with `/`)**: SQLPage looks for an `index.sql` file in that directory and executes it if found.

**Direct SQL file requests (`.sql` extension)**: SQLPage executes the requested SQL file if it exists.

**Static asset requests (other extensions)**: SQLPage serves files like CSS, JavaScript, images, or any other static content directly.

**Clean URL requests (no extension)**: SQLPage first tries to find a matching `.sql` file. If that doesn''t exist but there''s an `index.sql` file in a directory with the same name, it redirects to the directory path with a trailing slash.

### Error Handling

When, after applying each of the rules above in order, SQLPage can''t find a requested file,
it walks up your directory structure looking for [custom `404.sql` files](/your-first-sql-website/custom_urls).

## Dynamic Routing with SQLPage

SQLPage''s file-based routing becomes powerful when combined with strategic use of 404.sql files to handle dynamic URLs. Here''s how to build APIs and pages with dynamic parameters:

### Product Catalog with Dynamic IDs

**Goal**: Handle URLs like `/products/123`, `/products/abc`, `/products/new-laptop`

**Setup**:
```text
products/
├── index.sql          # Lists all products (/products/)
├── 404.sql           # Handles /products/<product-id>
└── categories.sql    # Product categories (/products/categories)
```

**How it works**:
- `/products/` → Executes `products/index.sql` (product listing)
- `/products/123` → No `123.sql` file exists, so executes `products/404.sql`
- `/products/laptop` → No `laptop.sql` file exists, so executes `products/404.sql`

**In `products/404.sql`**:
```sql
set product_id = substr(sqlpage.path(), 1+length(''/products/''));
```
        '
    );

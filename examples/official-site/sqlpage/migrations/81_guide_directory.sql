CREATE TABLE guide (
    sort_order INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    link TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL
);

INSERT INTO guide (sort_order, title, link, description) VALUES
    (1, 'Your first website', '/your-first-sql-website/', 'Install SQLPage and build a website from a SQL file.'),
    (2, 'SQL and variables', '/extensions-to-sql.sql', 'Read request parameters, set variables, and call SQLPage functions.'),
    (3, 'Custom components', '/custom_components.sql', 'Render your own HTML with Handlebars templates.'),
    (4, 'Database migrations', '/your-first-sql-website/migrations.sql', 'Create and update your database schema with versioned SQL files.'),
    (5, 'Custom URLs', '/your-first-sql-website/custom_urls.sql', 'Give your pages readable URLs and route requests to SQL files.'),
    (6, 'Reverse proxy', '/your-first-sql-website/nginx.sql', 'Serve your site with Nginx and HTTPS.'),
    (7, 'Managed hosting', '/your-first-sql-website/hosted.sql', 'Try SQLPage online or publish your website with DataPage.');

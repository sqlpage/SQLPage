select 'http_header' as component, 'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

-- Fetch the page title and header from the database
select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

SELECT 'hero' as component,
    'From database to data app, fast.' as title,
    '**SQLPage** lets you build data-driven applications in a few SQL queries.

It’s free, [open-source](https://github.com/lovasoa/sqlpage), lightweight, and easy to use.
    ' as description_md,
    'sqlpage_cover_image.webp' as image,
    TRUE as rounded,
    'your-first-sql-website/' as link,
    'Build your first SQL website !' as link_text;

-- the mantra: fast, beautiful, easy
SELECT 'Easy' as title,
    'You can teach yourself enough SQL to query and edit a database through SQLPage in a weekend.
We handle [security](safety.sql) and [performance](performance.sql) for you, so you can focus on your data.' as description_md,
    'sofa' as icon,
    'blue' as color;
SELECT 'Beautiful' as title,
    'The page you are looking at right now is written entirely in SQL.
No design skills are required, yet your website will be responsive, and look professional and modern by default.' as description,
    'eye' as icon,
    'green' as color;
SELECT 'Fast' as title,
    'Pages [load instantly](performance.sql), even on slow mobile networks.
    SQLPage is designed as a single **lightweight** executable, ensuring fast performance even on low-cost servers.' as description_md,
    'mail-fast' as icon,
    'red' as color;

SELECT 'hero' as component,
    true as reverse,
    '🧩 Batteries included' as title,
    'At the core of SQLPage is a [rich library of components](/documentation.sql) for tables, charts, maps, tables, forms and much more.

You just populate the components with data returned by your database queries.' as description_md,
    'sqlpage_illustration_components.webp' as image;

SELECT 'hero' as component,
    '🪄 We did the hard part' as title,
    'SQLPage handles HTTP requests, database connections, streaming rendering, styling, [security](safety.sql), and [performance](performance.sql) for you.

Focus only on your data, and how you want to present it. We''ve tamed the tech, you tame the data.' as description_md,
    'sqlpage_illustration_alien.webp' as image;

-- Quick feature overview
SELECT 'card' as component,
    'What is SQLPage ?' as title,
    1 as columns;
SELECT '✨ SQLPage turns your SQL queries into eye-catching websites' as title,
    '
SQLPage is a tool that allows you to **build websites** using nothing more than **SQL queries**.
You write simple text files containing SQL queries, SQLPage runs them on your database, and **renders the results as a website**.

You can display the information you `SELECT` from your database in
lists, tables, charts, maps, forms, and many other user interface widgets.
But you can also `INSERT`, `UPDATE` and `DELETE` data from your database using SQLPage, and build a full webapp.' as description_md,
    'paint' as icon,
    'blue' as color;
SELECT '🧩 Pre-built components let you construct websites Quickly and Easily' as title,
    'At the core of SQLPage is [a rich library of **components**](./documentation.sql).
    These components are built using traditional web technologies, but you never have to edit them if you don''t want to.
    SQLPage populates the components with data returned by your SQL queries.
    You can build entire web applications just by combining the components that come bundled with SQLPage.

As an example, the list of features on this page is generated using a simple SQL query that looks like this:

```sql
SELECT ''card'' as component, ''What is SQLPage ?'' as title;
SELECT header AS title, contents AS description_md FROM homepage_features;
```

However, you can also create your own components, or edit the existing ones to customize your website to your liking.
Creating a new component is as simple as creating an HTML template file.
' as description_md,
    'rocket' as icon,
    'green' as color;
SELECT '🛡️ A secure web server written in Rust' as title,
    'SQLPage removes a lot of the complexity and bloat of the modern web.
It''s a **lightweight web server** that just receives a request, finds the file to execute, runs it,
and returns a web page for the browser to display.

Written in a fast and secure programming language ([**Rust**](https://en.wikipedia.org/wiki/Rust_(programming_language))),
it empowers non-developers to build secure web applications easily. You download [a single executable file](https://github.com/lovasoa/SQLpage/releases),
write an `index.sql`, and in five minutes you turned your database into a website that you can
[deploy on the internet easily](https://datapage.app).

We made all the [optimizations](performance.sql), wrote all of the HTTP request handling code and rendering logic,
implemented all of the security features, so that you can think about your data, and nothing else.

When SQLPage receives a request with a URL ending in `.sql`, it finds the corresponding
SQL file, runs it on the database, passing it information from the web request as SQL statement parameters
[in a safe manner](safety.sql).
When the database starts returning rows for the query,
SQLPage maps each piece of information in the row to a parameter in the template of a pre-defined component,
and streams the result back to the user''s browser.
' as description_md,
    'server' as icon,
    'purple' as color;
SELECT '🌱 Start Simple, Scale to Advanced' as title,
    'SQLPage is a great starting point for building websites, internal tools, dashboards and data applications,
especially if you don''t have a lot of time, but need something more powerful and user-friendly than a spreadsheet.

When your app grows, you can take the same underlying data structure and queries,
and wrap them in a more established framework with a dedicated front end if you need to.
There is no lock-in, only simple, standard SQL queries that run directly on your database. 

**Focus on what matters** first: your data and your users. Not centering boxes in CSS, not setting up a web framework.' as description_md,
    'world-cog' as icon,
    'orange' as color;

-- Useful links
SELECT 'list' as component,
    'Get started: where to go from here ?' as title,
    'Here are some useful links to get you started with SQLPage.' as description;
SELECT 'Download' as title,
    'https://github.com/lovasoa/SQLpage/releases' as link,
    'SQLPage is distributed as a single binary that you can execute locally or on a web server to get started quickly.' as description,
    'green' as color,
    'download' as icon;
SELECT 'Tutorial' as title,
    'get started.sql' as link,
    'A short tutorial that will guide you through the creation of your first SQL-only website.' as description,
    'orange' as color,
    'book' as icon,
    TRUE as active;
SELECT 'SQLPage Documentation' as title,
    'documentation.sql' as link,
    'List of all available components, with examples of how to use them.' as description,
    'purple' as color,
    'book' as icon;
SELECT 'Technical documentation on Github' as title,
    'https://github.com/lovasoa/SQLpage/blob/main/README.md#sqlpage' as link,
    'The official README file on Github contains instructions to get started using SQLPage.' as description,
    'yellow' as color,
    'file-text' as icon;
SELECT 'Youtube Video Series' as title,
    'https://www.youtube.com/playlist?list=PLTue_qIAHxAf9fEjBY2CN0N_5XOiffOk_' as link,
    'A series of video tutorials that will guide you through the creation of an application with SQLPage.' as description,
    'red' as color,
    'brand-youtube' as icon;
SELECT 'Learnsqlpage.com' as title,
    'https://learnsqlpage.com' as link,
    'A website dedicated to learning SQLPage, with detailed tutorials.' as description,
    'blue' as color,
    'globe' as icon;

SELECT 'list' as component, 'Examples' as title;
SELECT 'Github Examples' as title,
    'https://github.com/lovasoa/SQLpage/tree/main/examples/' as link,
    'SQL source code for examples and demos of websites built with SQLPage.' as description,
    'teal' as color,
    'code' as icon;
SELECT 'Corporate Conundrum' as title,
    'https://conundrum.ophir.dev' as link,
    'A demo web application powered by SQLPage, designed for playing a fun trivia board game with friends.' as description,
    'cyan' as color,
    'affiliate' as icon;

SELECT 'list' as component, 'Community' as title;
SELECT 'Discussion forum' as title,
    'https://github.com/lovasoa/SQLpage/discussions' as link,
    'Come to our community page to discuss SQLPage with other users and ask questions.' as description,
    'pink' as color,
    'user-heart' as icon;
-- github link
SELECT 'Source code' as title,
    'https://github.com/lovasoa/SQLPage' as link,
    'The rust source code for SQLPage itself is open and available on Github.' as description,
    'github' as color,
    'brand-github' as icon;
SELECT 'Report a bug, make a suggestion' as title,
    'https://github.com/lovasoa/SQLPage/issues' as link,
    'If you have a question, a suggestion, or if you found a bug, please open an issue on Github.' as description,
    'red' as color,
    'bug' as icon;

-- User personas: who is SQLPage for ?
SELECT 'card' as component,
    'Is SQLPage for you ?' as title,
    '
SQLPage empowers SQL-savvy individuals to create dynamic websites without complex programming.

 - If you are looking to quickly build something simple yet dynamic, SQLPage is for you.
 - If you want to customize how every pixel of your website looks, SQLPage is not for you.

Compared to other low-code platforms, SQLPage focuses on SQL-driven development, more lightweight performance, and total openness.
Where other platforms try to lock you in, SQLPage makes it trivial to switch to something else when your application grows.' as description_md,
    4 as columns;
SELECT 'Business Analyst' as title,
    'Replace static dashboards with dynamic websites' as description,
    'Business analysts can leverage SQLPage to create interactive and real-time data visualizations, replacing traditional static dashboards and enabling more dynamic and insightful reporting.' as footer,
    'green' as color,
    'chart-arrows-vertical' as icon;
SELECT 'Data Scientist' as title,
    'Prototype and share data-driven experiments and analysis' as description,
    'Data scientists can utilize SQLPage to quickly prototype and share their data-driven experiments and analysis by creating interactive web applications directly from SQL queries, enabling collaboration and faster iterations.' as footer,
    'purple' as color,
    'square-root-2' as icon;
SELECT 'Marketer' as title,
    'Create dynamic landing pages and personalized campaigns' as description,
    'Marketers can leverage SQLPage to create dynamic landing pages and personalized campaigns by fetching and displaying data from databases, enabling targeted messaging and customized user experiences.' as footer,
    'orange' as color,
    'message-circle-dollar' as icon;
SELECT 'Engineer' as title,
    'Build internal tools and admin panels with ease' as description,
    'Engineers can use SQLPage to build internal tools and admin panels, utilizing their SQL skills to create custom interfaces and workflows, streamlining processes and improving productivity.' as footer,
    'blue' as color,
    'settings' as icon;
SELECT 'Product Manager' as title,
    'Create interactive prototypes and mockups' as description,
    'Product managers can leverage SQLPage to create interactive prototypes and mockups, allowing stakeholders to experience and provide feedback on website functionalities before development, improving product design and user experience.' as footer,
    'red' as color,
    'cube-send' as icon;
SELECT 'Educator' as title,
    'Develop interactive learning materials and exercises' as description,
    'Educators can utilize SQLPage to develop interactive learning materials and exercises, leveraging SQLPage components to present data and engage students in a dynamic online learning environment.' as footer,
    'yellow' as color,
    'school' as icon;
SELECT 'Researcher' as title,
    'Create data-driven websites to share findings and insights' as description,
    'Researchers can use SQLPage to create data-driven websites, making complex information more accessible and interactive for the audience, facilitating knowledge dissemination and engagement.' as footer,
    'cyan' as color,
    'flask-2' as icon;
SELECT 'Startup Founder' as title,
    'Quickly build a Minimum Viable Product' as description,
    'Startup founders can quickly build a Minimum Viable Product (MVP) using their SQL expertise with SQLPage, creating a functional website with database integration to validate their business idea and gather user feedback.' as footer,
    'pink' as color,
    'rocket' as icon;

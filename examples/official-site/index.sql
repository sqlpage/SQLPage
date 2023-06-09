select 'http_header' as component, 'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";
-- Using the 'shell' component at the top allows you to customize your web page, giving it a title and a description
select 'shell' as component,
    'SQLPage: Build SQL-only web applications' as title,
    'database' as icon,
    '/' as link,
    'en-US' as lang,
    'Official SQLPage website: write web applications in SQL !' as description,
    'get started' as menu_item,
    'documentation' as menu_item,
    20 as font_size,
    'Poppins' as font;

SELECT 'hero' as component,
    'SQLPage' as title,
    'Open-source *low-code* web application framework.

Create **full websites** writing only simple database queries.' as description_md,
    'Lac_de_Zoug.jpg' as image,
    'get started.sql' as link,
    'Build your first SQL website now !' as link_text;
-- the mantra: fast, beautiful, easy
SELECT 'Fast' as title,
    'Pages load instantly, even on slow mobile networks.
    SQLPage is designed as a single **lightweight** executable, ensuring fast performance even on low-cost servers.' as description_md,
    'mail-fast' as icon,
    'red' as color;
SELECT 'Beautiful' as title,
    'The page you are looking at right now is written entirely in SQL,
    using only professional looking pre-defined SQLPage components.' as description,
    'eye' as icon,
    'green' as color;
SELECT 'Easy' as title,
    'You can teach yourself enough SQL to select, update, and insert data in a database through SQLPage in a weekend.' as description,
    'sofa' as icon,
    'blue' as color;

-- Quick feature overview
SELECT 'card' as component,
    'What is SQLPage ?' as title,
    1 as columns;
SELECT 'SQLPage transforms your SQL queries into stunning websites' as title,
    '
SQLPage is a tool that allows you to **build websites** using nothing more than **SQL queries**.
You write simple text files containing SQL queries, SQLPage runs them on your database, and **renders the results as a website**.

You can display the information you `SELECT` from your database in lists, tables, cards and other user interface widgets.
But you can also `INSERT`, `UPDATE` and `DELETE` data from your database using SQLPage, and build a full webapp with **C**reate, **R**ead,
**U**pdate, **D**elete functionality.' as description_md,
    'paint' as icon,
    'blue' as color;
SELECT 'Pre-built components let you construct websites Quickly and Easily' as title,
    'At the core of SQLPage is [a rich library of **components**](./documentation.sql).
    These components are built using traditional web technologies, but you never have to edit them if you don''t want to.
    SQLPage populates the components with data returned by your SQL queries.
    You can build entire web applications just by combining the components that come bundled with SQLPage.

As an example, the list of features on this page is generated using a simple SQL query that looks like this:

```sql
SELECT ''card'' as component, ''What is SQLPage ?'' as title;
SELECT header AS title, contents AS description FROM homepage_features;
```

Additionnally, SQLPage itself is written in a fast and secure programming language: Rust.
We made all the optimizations so that you can think about your data, and nothing else.' as description_md,
    'rocket' as icon,
    'green' as color;
SELECT 'Technically, it''s just a good old web server' as title,
    '
The principles behind SQLPage are not too far from those that powered the early days of the internet.
Like [PHP](https://en.wikipedia.org/wiki/PHP), SQLPage just receives a request, finds the file to execute, runs it, and returns a response.

SQLPage is a *web server* written in
[rust](https://en.wikipedia.org/wiki/Rust_(programming_language))
and [distributed as a single executable file](https://github.com/lovasoa/SQLpage/releases).
When it receives a request with a URL ending in `.sql`, it finds the corresponding
SQL file, runs it on the database,
passing it information from the web request as SQL statement parameters.
When the database starts returning rows for the query,
SQLPage maps each piece of information in the row to a parameter in the template of a pre-defined component,
and streams the result back to the user''s browser.
' as description_md,
    'flask' as icon,
    'purple' as color;
SELECT 'Start Simple, Scale to Advanced' as title,
    'SQLPage is a great starting point for building websites, especially if you''re new to coding, or want to test out a new idea quickly.
    Then if the app becomes important, you can take the same underlying data structure and wrap it in a more established framework with a dedicated front end.
    And if it doesn''t, you only spent a few hours on it!

    SQLPage does not impose any specific database structure, allowing for seamless integration with other tools and frameworks.
    SQLPage is a solid foundation for your website development, because it lets you focus on what matters at the beginning, without closing the door to future improvements.' as description,
    'world-cog' as icon,
    'orange' as color;

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

-- Useful links
SELECT 'list' as component,
    'Useful links' as title,
    'Here are some useful links to get you started with SQLPage.' as description;
SELECT 'Official website' as title,
    'https://sql.ophir.dev' as link,
    'The project''s official home page.' as description,
    'blue' as color,
    'home' as icon,
    TRUE as active;
SELECT 'Download' as title,
    'https://github.com/lovasoa/SQLpage/releases' as link,
    'SQLPage is distributed as a single binary that you can execute locally or on a web server to get started quickly.' as description,
    'green' as color,
    'download' as icon;
SELECT 'SQLPage Documentation' as title,
    'documentation.sql' as link,
    'List of all available components, with examples of how to use them.' as description,
    'purple' as color,
    'book' as icon;
SELECT 'Examples' as title,
    'https://github.com/lovasoa/SQLpage/tree/main/examples/' as link,
    'SQL source code for examples and demos of websites built with SQLPage.' as description,
    'teal' as color,
    'code' as icon;
SELECT 'Corporate Conundrum' as title,
    'https://conundrum.ophir.dev' as link,
    'A demo web application powered by SQLPage, designed for playing a fun trivia board game with friends.' as description,
    'cyan' as color,
    'affiliate' as icon;
-- github link
SELECT 'Source code' as title,
    'https://github.com/lovasoa/SQLPage' as link,
    'The rust source code for SQLPage itself is open and available on Github.' as description,
    'github' as color,
    'brand-github' as icon;
SELECT 'Technical instructions to get started' as title,
    'https://github.com/lovasoa/SQLpage/blob/main/README.md#sqlpage' as link,
    'The official README file on Github contains instructions to get started using SQLPage.' as description,
    'yellow' as color,
    'file-text' as icon;
SELECT 'Report a bug, make a suggestion' as title,
    'https://github.com/lovasoa/SQLPage/issues' as link,
    'If you have a question, a suggestion, or if you found a bug, please open an issue on Github.' as description,
    'red' as color,
    'bug' as icon;
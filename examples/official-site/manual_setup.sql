select 'shell' as component,
    'SQLPage Manual Setup' as title,
    'database' as icon,
    '/' as link,
    'en-US' as lang,
    'SQLPage technical introduction' as description,
    'documentation' as menu_item,
    'Poppins' as font;

SELECT 'hero' as component,
    'SQLPage setup' as title,
    'Let''s create your first SQLPage website together, step by step, from downloading SQLPage to making your site available online for everyone to browse.' as description,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/c/c4/Backlit_keyboard.jpg/1024px-Backlit_keyboard.jpg' as image,
    'mailto:contact@ophir.dev' as link,
    'Instructions unclear ? Get in touch !' as link_text;


SELECT 'text' as component,
    'Download SQLPage' as title;
SELECT 'Download the latest SQLPage' as contents, 'https://github.com/lovasoa/SQLpage/releases' as link;
SELECT ' for your operating system.
SQLPage is distributed as a single executable file, making it easy to get started.' as contents;

SELECT 'text' as component, 'Launch your development server' as title;
SELECT 'Create a folder on your computer where you will store your website.
Then launch the ' as contents;
SELECT 'sqlpage' as contents, TRUE as code;
SELECT ' executable file you just downloaded in a terminal from this folder.' as contents;
SELECT 'text' as component,
    'You should see a message in your terminal that includes the sentence ' as contents;
SELECT 'Starting server on 0.0.0.0:8080' as contents, TRUE as code;
SELECT '. This means that sqlpage is running successfully. '
SELECT 'text' as component, 'You can open your website locally by visiting ' as contents;
SELECT 'http://localhost:8080' as contents, TRUE as code, 'http://localhost:8080' as link;

SELECT 'text' as component,
    'Your website''s first SQL file' as title;
SELECT 'In the root folder of your SQLPage website, create a new SQL file called "index.sql".' as contents;
SELECT 'text' as component, 'Open the "index.sql" file in a text editor.' as contents;
SELECT 'text' as component, 'Write your SQL code in the "index.sql" file to retrieve data from your database and define how it should be displayed on your website.' as contents;
SELECT 'text' as component, 'For example, you can start with a simple SQL code that displays a list of popular websites from your "website" table. Here''s an example:' as contents;
SELECT 'text' as component;
SELECT 'SELECT
''list'' AS component,
''Popular websites'' AS title;' as contents, TRUE as code;
SELECT 'text' as component;
SELECT '
SELECT
 ''Hello'' AS title,
 ''world'' AS description,
 ''https://wikipedia.org'' AS link;' as contents, TRUE as code;

SELECT 'text' as component, 'The list of components you can use and their properties is available in ' as contents;
select 'SQLPage''s online documentation' as contents, 'https://sql.ophir.dev/documentation.sql' as link;
SELECT '.' as contents;

SELECT 'text' as component, 'Your database schema' as title;

SELECT 'The database schema for your SQLPage website is defined using SQL scripts located in the "sqlpage/migrations" subdirectory of your website''s root folder.
Each script represents a migration that sets up
or modifies the database structure.
The scripts are executed in alphabetical order, so you can prefix them with a number to control the order in which they are executed.' as contents;
SELECT 'text' as component, 'For example, you can create a file called "0001_create_website_table.sql" with the following contents:' as contents;
SELECT 'text' as component;
SELECT 'CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);' as contents, TRUE as code;

SELECT 'text' as component, 'Connect to a custom database' as title;
SELECT 'By default, SQLPage uses a SQLite database stored in a file named "sqlpage.db" in your website''s root folder.
You can change this by creating a file named "sqlpage/config.json" in your website''s root folder with the following contents:' as contents;
SELECT 'text' as component;
SELECT '{
    "database_url": "postgres://user:password@localhost:5432/database"
}' as contents, TRUE as code;
SELECT 'text' as component, 'For more information about the properties that can be set in config.json, see ' as contents;
SELECT 'SQLPage''s configuration documentation' as contents, 'https://github.com/lovasoa/SQLpage/blob/main/configuration.md#configuring-sqlpage' as link;

SELECT 'text' as component,
    'Deploy your SQLPage website online' as title;
SELECT 'If you want to make your SQLPage website accessible online for everyone to browse, you can deploy it to a VPS (Virtual Private Server). ' as contents;
SELECT 'To get started, sign up for a VPS provider of your choice. Some popular options include: AWS EC2, DigitalOcean, Linode, Hetzner. ' as contents;
SELECT 'text' as component, 'Once you have signed up with a VPS provider, create a new VPS instance. The steps may vary depending on the provider, but generally, you will need to:' as contents;
SELECT 'text' as component, '1. Choose the appropriate server type and specifications. SQLPage uses very few resources, so you should be fine with the cheaper options.' as contents;
SELECT 'text' as component, '2. Set up SSH access.' as contents;
SELECT 'text' as component;
SELECT 'Once your VPS instance is up and running, you can connect to it using SSH. The provider should provide you with the necessary instructions on how to connect via SSH.' as contents;
SELECT 'text' as component,
    'For example, if you are using a Linux or macOS terminal, you can use the following command:' as contents;
SELECT 'text' as component;
SELECT 'ssh username@your-vps-ip-address' as contents, TRUE as code;
SELECT 'title' as component,
    'Transfer your SQLPage website files to the VPS' as contents, 3 as level;
SELECT 'You need to transfer your SQLPage website files from your local computer to the VPS. There are several ways to achieve this, including using SCP (Secure Copy) or SFTP (SSH File Transfer Protocol).' as contents;
SELECT 'text' as component,
    'For example, if you are using SCP, you can run the following command from your local computer, replacing the placeholders with your own information:' as contents;
SELECT 'text' as component;
SELECT 'scp -r /path/to/your/sqlpage/folder username@your-vps-ip-address:/path/to/destination' as contents,
    TRUE as code;

SELECT 'title' as component, 'Run sqlpage on the server' as contents, 3 as level;
SELECT 'text' as component, 'Once your SQLPage website files are on the server, you can run sqlpage on the server, just like you did on your local computer.
Download the sqlpage for linux binary and upload it to your server. ' as contents;
SELECT 'text' as component, 'Then, run the following command on your server:' as contents;
SELECT 'text' as component;
SELECT './sqlpage' as contents, TRUE as code;
SELECT 'text' as component,
    'To access your website, enter the adress of your VPS in your adress bar, followed by the port on which sqlpage runs. For instance: http://123.123.123.123:8080.' as contents;
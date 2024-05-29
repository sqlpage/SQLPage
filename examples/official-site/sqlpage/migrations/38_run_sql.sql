INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'run_sql',
        '0.20.0',
        'login',
        'Executes another SQL file and returns its result as a JSON array.

### Example
    
#### Include a common header in all your pages

It is common to want to run the same SQL queries at the beginning of all your pages,
to check if an user is logged in, render a header, etc.
You can create a file called `common_header.sql`,
and use the [`dynamic`](documentation.sql?component=dynamic#component) component with the `run_sql` function
to include it in all your pages.

```sql
select ''dynamic'' as component, sqlpage.run_sql(''common_header.sql'') as properties;
```

#### Notes

 - **recursion**: you can use `run_sql` to include a file that itself includes another file, and so on. However, be careful to avoid infinite loops. SQLPage will throw an error if the inclusion depth is superior to 8.
 - **security**: be careful when using `run_sql` to include files. Never use `run_sql` with a user-provided parameter. Never run a file uploaded by a user, or a file that is not under your control.
 - **variables**: the included file will have access to the same variables (URL parameters, POST variables, etc.)
   as the calling file.
   If the included file changes the value of a variable or creates a new variable, the change will not be visible in the calling file.
'
    );
INSERT INTO sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES (
        'run_sql',
        1,
        'file',
        'Path to the SQL file to execute, can be absolute, or relative to the web root (the root folder of your website sql files).
        In-database files, from the sqlpage_files(path, contents, last_modified) table are supported.',
        'TEXT'
    );

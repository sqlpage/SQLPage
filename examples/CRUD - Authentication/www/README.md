This demo/template defines a basic CRUD application with authentication designed for use with SQLite. The primary goal of this demo is to explore some of the SQLPage features and figure out how to implement various aspects of a data-centric application. The template contains both public pages ([**SQLite Introspection**](intro.sql)) and pages requiring authentication. A user management GUI is not available (database migrations, included in the project, create a default login admin/admin).

The website root is set to _./www_, and the database is in the _./db/_ directory. The database contains one data table, "currencies", and its content is listed on a login-protected [**page**](currencies_list.sql). After successful authentication, you can see the list of records and access the form for adding/editing/deleting records.

## Authentication process

Three files (login.sql, logout.sql, and create_session.sql) implement authentication mostly following the code provided in other examples. The login.sql defines the actual login form, and the two other files do not have any associated GUI but perform appropriate processing and redirect to designated targets. Given a protected page, the general authentication flow is as follows.
 
1. The user attempts to open a protected page (e.g., currencies_list.sql)
2. Session checking code snippet at the top of the protected page checks if a valid session token (cookie) is set. In this example, the SET statement sets a local variable, `$_username`, for later use:
```sql
-- Checks if a valid session token cookie is available
SET $_username = (
    SELECT username
    FROM sessions
    WHERE sqlpage.cookie('session_token') = id
      AND created_at > datetime('now', '-1 day')
);
```
3. Redirect to login page (login.sql) if no session is available (`$_username IS NULL`) and the starting page requires authentication (by setting `SET $_session_required = 1;` before executing the session checking code; see, e.g., the top of currencies_item_form.sql and currencies_list.sql):
```sql
SELECT
    'redirect' AS component,
    '/login.sql?path=' || $_curpath AS link
WHERE $_username IS NULL AND $_session_required;
```
4. The login page renders the login form, accepts the user credentials, and redirects to create_session.sql, passing the login credentials as POST variables.
5. create_session.sql checks credentials. If this check fails, it redirects back to the login form. If the check succeeds, it generates a session token and performs the final redirect.

## Header module

### Controlling execution of parts in a loaded script

Because the same code is used for session token check for all protected pages, it makes sense to place it in a separate module (header_shell_session.sql) and execute it via run_sql() at the top of protected files:

```sql
SET $_curpath = sqlpage.path();
SET $_session_required = 1;

SELECT
    'dynamic' AS component,
    sqlpage.run_sql('header_shell_session.sql') AS properties;
```

The second line above sets the local variable $\_session_required, which indicates whether authentication is required for a particular page. This variable, like the GET/POST variables, is then accessible to the loaded "header" module header_shell_session.sql. This way, if other common code is placed in the header module, it can be executed by a non-protected page while skipping the authentication part (by setting $\_session_required = 0, which prevents redirect to the login form even if no valid session token is available).

Another "filter" variable (`$_shell_enabled`) controls the execution of another section in the header module, as discussed below.

### Tracking the calling page

The first line above sets another useful variable $\_curpath, which makes it possible to redirect back to the starting page after the authentication process is completed, rather than redirecting to the front page. The loaded header module has access to this variable as well, and if a login redirect is required, this variable is passed alone as a GET URL parameter (as a part of the "link" property):

```sql
SELECT
    'redirect' AS component,
    '/login.sql?path=' || $_curpath AS link
WHERE $_username IS NULL AND $_session_required;
```

The login form passes it further in a similar fashion to the create_session.sql script (as part of the "action" property):

```sql
SELECT
    'form'  AS component,
    'Login' AS title,
    'create_session.sql' || ifnull('?path=' || $path, '') AS action;
```
If authentication fails, create_session.sql redirects back to login.sql and makes sure to pass the $path value alone (as a part of the "link" property):

```sql
SELECT
    'authentication' AS component,
    'login.sql?' || ifnull('path=' || $path, '') || '&error=1' AS link,
    :password AS password,
    (SELECT password_hash
     FROM "accounts"
     WHERE username = :username) AS password_hash;
```

If authentication succeeds, create_session.sql redirects back to starting page using the $path value as the final redirect target:

```sql
SELECT
    'redirect' AS component,
    ifnull($path, '/') AS link;
```

### Adding User/Login/Logout buttons to the page menu

It is customary to show the "User Profile" button in the top right corner when the user is authenticated. Also, it is customary to show the "Logout" button next to the "User Profile" button or the "Login" button when no active session is available. The associated code is common to all pages, and it makes sense to place it in the same header module.

The "shell" component is responsible for constructing the top menu, but the standard component does not support menu buttons. The simplest solution to this "limitation" is to modify the standard shell.handlebars template found in the "sqlpage/templates" directory of the SQLPage source code repository and place it inside the project "sqlpage/templates" directory.

To extend the "shell" component with button items in the menu, I have added a hybrid section of code mostly constructed from template code defining menu items and the "button" component. In the present implementation, menu buttons are defined as a JSON array value to the "menu_buttons" property. Each array member is a JSON object defining a single button and may include "shape", "color", "size", "outline", "link", "tooltip", and "title" properties (see description of these properties in the official "button" component docs.

Note how the `$_curpath` variable, which is set in core page modules (such as currencies_list.sql) is used to define links for the Login/Logout buttons. These links are irrelevant for protected pages, but for non-protected pages, such as intro.sql, these links make sure that the user remains on the same page after he/she presses on Logout/Login buttons (and completes authentication in the latter case).

The `$_username` variable set during the authentication process is then used to decide which buttons (Login or User/Logout) should be shown. 

The `$_shell_enabled` variable controls the execution of the custom shell component. This feature is necessary because the header module is also loaded by the currencies_item_dml.sql module, which should only be accessible to authenticated users. However, the currencies_item_dml.sql module is a no-GUI module, which performs database operations and uses redirects after the requested operations are completed. At the same time, if the loaded header module executes the custom shell component, generating GUI buttons, the redirection mechanism in currencies_item_dml.sql will fail.

### Required variable guards

The header modules expects that the calling module sets several variables. The SET statement makes it possible to check if the variables are set appropriately in one place at the beginning of the module, rather then placing guards every time theses variables are used. Hence, the top section of the header file includes

```sql
SET $_curpath = ifnull($_curpath, '/');
SET $_session_required = ifnull($_session_required, 1);
SET $_shell_enabled = ifnull($_shell_enabled, 1);
```
In this case, if any required variable is not set, a suitable default value is defined, so that the following code would not have to check for NULL values. Alternatively, a redirect to an error page may be used, to inform the programmer about the potential issue.

## Footer module - debug information

POST/GET/SET variables may provide helpful information for debugging purposes. In earlier [post](https://reddit.com/r/SQLpage/comments/1dh1siw/structuring_code_showing_debug_info/), I described the code I use to output variables in a convenient way. Briefly, I use `sqlpage.variables('GET')` and `sqlpage.variables('POST')` to get all variables, and I distinguish between the GET variables and local SET variables by prefixing SET variable names with an underscore. Initially, I copy-pasted the code snippets at the bottom of pages, but later I moved it to a separate file, footer_debug_post-get-set.sql, which I load via

```sql
SELECT
    'dynamic' AS component,
    sqlpage.run_sql('footer_debug_post-get-set.sql') AS properties
WHERE $DEBUG OR $error IS NOT NULL;
```
## Structuring code modules

The "currencies" table is handled by three modules:

- "table" view - __currencies_list.sql__
  Displays the entire table using the powerful "table" component. One way to extend this module is, possibly, to hide certain less important columns, especially for wide tables. 
- "detail" view - __currencies_item_form.sql__
  This is the "detail" view. It shows all fields for a single record. In this case, it is an "editable" form, though the fields maybe made conditionally read-only. Another possible option for a read-only detail view is to use the "datagrid" component.
- database DML processor - __currencies_item_dml.sql__
  This is a no-GUI module, which only processes database modification operations using  data submitted to the currencies_item_form.sql form. Presently, all DML statements (INSERT/UPDATE/DELETE) are processed by this module. If necessary, this module maybe split into more specialized modules.

Let us briefly go over the code block in these modules.

### Debug information (bottom section)

All three module load the footer module discussed above that produces a conditional output of GET/POST/SET variables.

### Authentication (top section)

All three modules provide access to the database and are treated as protected: they are only accessible to authenticated users. Hence, they start with (mostly) the same code block:

```sql
SET $_curpath = sqlpage.path();
SET $_session_required = 1;

SELECT
    'dynamic' AS component,
    sqlpage.run_sql('header_shell_session.sql') AS properties;
```

This code discussed above sets the current path variable (necessary for correct redirects), authentication flag before loading the header module that takes care of authentication and common settings, such as top menu buttons. 

The "no-GUI" currencies_item_dml.sql module does not set `$_curpath`, since it cannot be a start/end point in a redirect chain, but it sets the `$_shell_enabled` flag to suppress top menu buttons generation, as discussed earlier.

### Common variables

The second section may generally be used to set additional common variables, such as the name of the "table" view inside the "detail" view and the other way around (to switch between the two).

The "detail" view also uses the "&path" GET URL parameter, if provided (e.g., by the "table" view). This way, if a record is modified/deleted starting from, e.g., the "table" view, the same view is set as the final redirect target after the DML operation is completed.

### Table view

The rest of the table view module is fairly basic. It defines two alerts for displaying confirmation and error messages, a "new record" button, and the table itself. The last "actions" column is added to the table, designated as markdown, and includes shortcuts to edit/delete the corresponding record.

![](https://raw.githubusercontent.com/pchemguy/SQLpage/crud_auth/examples/crud-authentication/www/img/table_view.png)

### Detail view

The detail view module is more elaborate. If "&id" GET URL parameter is provided, the form shows the corresponding record. Otherwise, the ID field is rendered as a dropdown list populated from the database, but is set to NULL. The remaining fields are either blank or contain dummy values.

The first step (after the previously discussed common sections), therefore, is to filter invalid id values.

```sql
SELECT
    'redirect' AS component,
    $_curpath AS link
WHERE $id = '' OR CAST($id AS INT) = 0;

SET $error_msg = sqlpage.url_encode('Bad {id = ' || $id || '} provided');
SELECT
    'redirect' AS component,
    $_curpath || '?error=' || $error_msg AS link
WHERE $id NOT IN (SELECT currencies.id FROM currencies);
```

The blank string and zero are considered the equivalents of NULL, so redirect to itself is activated, removing the id parameter. If no id is provided or id is set to an integer value, the first check does not trigger. The second check above triggers when there is no record with provided id. This check resets id and displays an error message.

Another accepted GET URL parameter is $values, which may be set to a JSON representation of the record. This parameter is returned from the currencies_item_dml.sql script if the database operation fails. Then the detail view will display an error message, but the form will remain populated with the user-submitted data. If $values is set, it takes precedence. This check throws an error if $values is set, but does not represent a valid JSON.

```sql
SET $_err_msg =
    sqlpage.url_encode('Values is set to bad JSON: __ ') || $values || ' __';

SELECT
    'redirect' AS component,
    $_curpath || '?error=' || $_err_msg AS link
WHERE NOT json_valid($values);
```
The detail view maybe called with zero, one, or two (\$id/\$values) parameters. Invalid values are filtered out at this point, so the next step is to check provided parameters and determine the dataset that should go into the form. 

```sql
SET $_values = (
    WITH
        fields AS (
            SELECT id, name, to_rub
            FROM currencies
            WHERE id = CAST($id AS INT) AND $values IS NULL
                UNION ALL
            SELECT NULL, '@', 1
            WHERE $id IS NULL AND $values IS NULL
                UNION ALL
            SELECT
                $values ->> '$.id' AS id,
                $values ->> '$.name' AS name,
                $values ->> '$.to_rub' AS to_rub
            WHERE json_valid($values)
        )
    SELECT
        json_object(
            'id',     CAST(fields.id AS INT),
            'name',   fields.name,
            'to_rub', CAST(CAST(fields.to_rub AS TEXT) AS NUMERIC)
        )
    FROM fields
);
```

Each of the three united SELECTs in the "fields" CTE returns a single row and only one of them is selected for any given combination of \$id/\$values using the WHERE clauses. This query returns the "final" set of fields as a JSON object.

![](https://raw.githubusercontent.com/pchemguy/SQLpage/crud_auth/examples/crud-authentication/www/img/detail_view.png)

Now that the input parameters are validated and the "final" dataset is determined, it is the time to define the form GUI elements. First, I define the button to switch to the table view. Note that the same form is used to confirm record deletion, and when this happens, the "Browse" button is not shown.

```sql
SELECT 
    'button'            AS component,
    'pill'              AS shape,
    'lg'                AS size,
    'end'               AS justify;
SELECT                             
    'BROWSE'            AS title,
    'browse_rec'        AS id,
    'corner-down-left'  AS icon,
    'corner-down-left'  AS icon_after,
    'green'             AS outline,
    $_table_list        AS link,
    'Browse full table' AS tooltip
WHERE NOT ifnull($action = 'DELETE', FALSE);
```

The following section defines the main form with record fields. First the $\_valid_ids variable is constructed as the source for the drop-down id field. The code also adds the NULL value used for defining a new record. Note that, when this form is opened from the table view via the "New Record" button, the $action variable is set to "INSERT" and the id field is set to the empty array in the first assignment via the alternative UINION and to the single NULL in the second assignment. The two queries can also be combined relatively straightforwardly using CTEs.

```sql
SET $_valid_ids = (
    SELECT json_group_array(
        json_object('label', CAST(id AS TEXT), 'value', id) ORDER BY id
    )
    FROM currencies
    WHERE ifnull($action, '') <> 'INSERT'
        UNION ALL
    SELECT '[]'
    WHERE $action = 'INSERT'
);
SET $_valid_ids = (
    json_insert($_valid_ids, '$[#]',
        json_object('label', 'NULL', 'value', json('null'))
    )
);
```

The next part defines form fields via the "dynamic" component (for some reason I am having issues with POST variables when the form is defined directly via the "form" component. Note how the $values variable prepared in previous blocks is used to populate the form. Without the SET statement, everything would need to be incorporated in a single query (which is feasible thanks to CTEs, but would still be significantly more difficult to develop and maintain).
 
Also note that this single form definition actually combines two forms (the second being the record delete confirmation form).  If the $action variable is set to "DELETE" (after the delete operation is initiated from either the table or detail view), buttons are adjusted appropriately and all fields are set to read-only. Whether this is a good design is a separate question. Perhaps, defining two separate forms is a better approach.

![](https://raw.githubusercontent.com/pchemguy/SQLpage/crud_auth/examples/crud-authentication/www/img/delete_confirmation.png)


After the main form fields goes the delete confirmation alert, displayed after the delete operation is completed.

The last big section defines the main form buttons, which are adjusted based on the type of operation (similarly to the form fields above).

The final section includes a general confirmation alert (used after INSERT/UPDATE operations) and an error alert.

### Coding style conventions

Consistent code style is important for code readability. Because SQLPage module maybe a mix of SQL code and sizeable text fragments, which may contain plain text, Markdown, JSON, HTML, etc., it might be difficult to follow a fixed set of rules. In fact, dynamically generated webpages regardless of specific technologies used tend to get messy. At the very least I strive to

 - keep all SQL keywords always in the UPPER case,
 - have reasonably sensible code alignment (though some alignment approaches may not be generally advisable)
 - keep large static text pieces in separate appropriate dedicated files and load them via `sqlpage.read_file_as_text()` (e.g., the text of this file comes from Readme.md, where it can be properly edited by any Markdown editor and version-controlled; similarly, static JSON should go in \*.json files or in a dedicated database table with a designated JSON column).

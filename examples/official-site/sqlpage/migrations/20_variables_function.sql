-- Insert the 'variables' function into sqlpage_functions table
INSERT INTO sqlpage_functions (
    "name",
    "introduced_in_version",
    "icon",
    "description_md"
)
VALUES (
    'variables',
    '0.15.0',
    'variable',
    'Returns a JSON string containing variables from the HTTP request and user-defined variables.

The database''s json handling functions can then be used to process the data.

## Variable Types

SQLPage distinguishes between three types of variables:

- **GET variables**: URL parameters from the query string (immutable)
- **POST variables**: Form data from POST requests (immutable)  
- **SET variables**: User-defined variables created with the `SET` command (mutable)

## Usage

- `sqlpage.variables()` - returns all variables (GET, POST, and SET combined, with SET variables taking precedence)
- `sqlpage.variables(''get'')` - returns only URL parameters
- `sqlpage.variables(''post'')` - returns only POST form data
- `sqlpage.variables(''set'')` - returns only user-defined variables created with `SET`

When a SET variable has the same name as a GET or POST variable, the SET variable takes precedence in the combined result.

## Example: a form with a variable number of fields

### Making a form based on questions in a database table

We can create a form which has a field for each value in a given table like this:

```sql
select ''form'' as component, ''handle_survey_answer.sql'' as action;
select question_id as name, question_text as label from survey_questions;
```

### Handling form responses using `sqlpage.variables`

In `handle_survey_answer.sql`, one can process the form results even if we don''t know in advance
how many fields it contains.
The function to parse JSON data varies depending on the database engine you use.

#### In SQLite
In SQLite, one can use [`json_each`](https://www.sqlite.org/json1.html#jeach) :

```sql
insert into survey_answers(question_id, answer)
select "key", "value" from json_each(sqlpage.variables(''post''))
```

#### In Postgres

Postgres has [`json_each_text`](https://www.postgresql.org/docs/9.3/functions-json.html) :

```sql
INSERT INTO survey_answers (question_id, answer)
SELECT key AS question_id, value AS answer
FROM json_each_text(sqlpage.variables(''post'')::json);
```


#### In Microsoft SQL Server

```sql
INSERT INTO survey_answers
SELECT [key] AS question_id, [value] AS answer
FROM OPENJSON(sqlpage.variables(''post''));
```

#### In MySQL

MySQL has [`JSON_TABLE`](https://dev.mysql.com/doc/refman/8.0/en/json-table-functions.html), 
and [`JSON_KEYS`](https://dev.mysql.com/doc/refman/8.0/en/json-search-functions.html#function_json-keys)
which are a little bit less straightforward to use:

```sql
INSERT INTO survey_answers (question_id, answer)
SELECT
    question_id,
    json_unquote(
        json_extract(
            sqlpage.variables(''post''),
            concat(''$."'', question_id, ''"'')
        )
    )
FROM json_table(
    json_keys(sqlpage.variables(''post'')),
    ''$[*]'' columns (question_id int path ''$'')
) as question_ids
```

'
);

-- Insert the parameters for the 'variables' function into sqlpage_function_parameters table
-- Parameter 1: 'method' parameter
INSERT INTO sqlpage_function_parameters (
    "function",
    "index",
    "name",
    "description_md",
    "type"
)
VALUES (
    'variables',
    1,
    'method',
    'Optional. Filter variables by source: ''get'' (URL parameters), ''post'' (form data), or ''set'' (user-defined variables). When not provided, all variables are returned with SET variables taking precedence over request parameters.',
    'TEXT'
);

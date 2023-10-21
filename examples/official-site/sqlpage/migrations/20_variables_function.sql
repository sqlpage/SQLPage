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
    'Returns a JSON string containing all variables passed as URL parameters or posted through a form.

The database''s json handling functions can then be used to process the data.

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
FROM OPENJSON(''{"x":"y"}'');
```

#### In MySQL

MySQL has [`JSON_TABLE`](https://dev.mysql.com/doc/refman/8.0/en/json-table-functions.html), which is less straightforward to use.

Outside of sqlpage, one can define a procedure :

```sql
DELIMITER //
CREATE PROCEDURE process_survey_answer(jsonData JSON)
BEGIN
  insert into survey_answers (question_id, answer)
  SELECT
    var_name as "key",
    JSON_UNQUOTE(JSON_EXTRACT(jsonData, CONCAT(''$."'', var_name, ''"''))) as "value"
  FROM JSON_TABLE(
    JSON_KEYS(jsonData),
    ''$[*]'' COLUMNS (var_name text PATH ''$'' ERROR ON ERROR)
  ) AS vars;

END//
DELIMITER ;
```

then in `handle_survey_answer.sql` :

```sql
CALL process_survey_answer(sqlpage.variables(''post''));
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
    'Optional. The HTTP request method (GET or POST). Must be a literal string. When not provided, all variables are returned.',
    'TEXT'
);

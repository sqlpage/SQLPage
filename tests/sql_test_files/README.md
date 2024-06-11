The sql files in this folder are all tested automatically.

## `it_works_` files

Files with names starting with `it_works` should all
return a page that contains the text "It works !" and does not contain the
text "error" (case insensitive) when executed.

If a file name contains `nosqlite`, `nomssql`, `nopostgres` or `nomysql`, then
the test will be ignored when running against the corresponding database. 
This allows using syntax that is not supported on all databases in some tests.

## `error_` files

Files with names starting with `error` should all return a page that contains
the text "error" and the rest of the file name when executed.
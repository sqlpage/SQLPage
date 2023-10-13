The sql files in this folder are all tested automatically.

## `it_works_` files

Files with names starting with `it_works` should all
return a page that contains the text "It works !" and does not contain the
text "error" (case insensitive) when executed.

## `error_` files

Files with names starting with `error` should all return a page that contains
the text "error" and the rest of the file name when executed.
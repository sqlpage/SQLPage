GET http://localhost:8080/
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "Website files"
body contains "Create new file"
body contains "Database tables"
body contains "sqlpage_files"
body htmlUnescape contains "view_table.sql?table%5Fname=sqlpage%5Ffiles"
body not contains "An error occurred"

GET http://localhost:8080/edit.sql
HTTP 200
[Asserts]
body contains "New page"
body contains "insert_file.sql"
body contains "code-editor"
body not contains "An error occurred"

POST http://localhost:8080/insert_file.sql
[FormParams]
path: hurl_test.sql
contents: SELECT 'text' as component, 'Hello from Hurl' as contents;
HTTP 302
[Asserts]
header "Location" == "index.sql?inserted=hurl%5Ftest%2Esql"

GET http://localhost:8080/hurl_test.sql
HTTP 200
[Asserts]
body contains "Hello from Hurl"
body not contains "An error occurred"

GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "hurl_test.sql"
body htmlUnescape contains "edit.sql?path=hurl%5Ftest%2Esql"
body htmlUnescape contains "delete.sql?path=hurl%5Ftest%2Esql"
body not contains "An error occurred"

GET http://localhost:8080/view_table.sql?table_name=sqlpage_files
HTTP 200
[Asserts]
body contains "sqlpage_files"
body contains "hurl_test.sql"
body not contains "An error occurred"

GET http://localhost:8080/delete.sql?path=hurl_test.sql
HTTP 200
[Asserts]
body contains "Delete hurl_test.sql ?"
body htmlUnescape contains "delete.sql?path=hurl%5Ftest%2Esql&confirm=yes"
body not contains "An error occurred"

GET http://localhost:8080/delete.sql?path=hurl_test.sql&confirm=yes
HTTP 302
[Asserts]
header "Location" == "index.sql?deleted=hurl%5Ftest%2Esql"

GET http://localhost:8080/
HTTP 200
[Asserts]
body not contains "hurl_test.sql"
body not contains "An error occurred"

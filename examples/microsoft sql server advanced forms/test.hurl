GET http://127.0.0.1:8080/
HTTP 200
[Asserts]
xpath "//form//input[@name='Name']" exists
xpath "//form//input[@name='GroupDescription']" exists
xpath "//code[@class='sqlpage-error-description']" not exists

# The form maps a blank description to SQL NULL before binding the insert.
POST http://127.0.0.1:8080/
[FormParams]
Name: L'équipe 東京
GroupDescription:
HTTP 200
[Asserts]
xpath "//code[@class='sqlpage-error-description']" not exists

GET http://127.0.0.1:8080/
Accept: application/json
HTTP 200
[Asserts]
jsonpath "$[?(@.title == \"L'équipe 東京\")].title" == "L'équipe 東京"
jsonpath "$[?(@.title == \"L'équipe 東京\")].description" == null

# SQL Server rejects descriptions longer than the column's 80 characters.
POST http://127.0.0.1:8080/
[FormParams]
Name: Rejected group
GroupDescription: This description is more than eighty characters long and exceeds the database column size.
HTTP 200
[Asserts]
xpath "string(//code[@class='sqlpage-error-description'])" contains "would be truncated"

POST http://127.0.0.1:8080/
[FormParams]
Name: Recovered group
GroupDescription: Saved after a failed write
HTTP 200
[Asserts]
xpath "//code[@class='sqlpage-error-description']" not exists

GET http://127.0.0.1:8080/
Accept: application/json
HTTP 200
[Asserts]
jsonpath "$[?(@.title == 'Rejected group')]" not exists
jsonpath "$[?(@.title == 'Recovered group')].description" == "Saved after a failed write"

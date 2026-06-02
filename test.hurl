# Generic smoke test for SQLPage examples.
GET http://localhost:8080
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
xpath "//html" exists
xpath "//body" exists
body not contains "An error occurred"

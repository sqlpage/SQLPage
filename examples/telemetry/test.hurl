GET http://localhost:8080/
traceparent: 00-00000000000000000000000000000001-0000000000000001-01
HTTP 200
[Asserts]
body contains "Todo List"
body contains "Add a todo"
body not contains "An error occurred"

POST http://localhost:8080/add_todo.sql
traceparent: 00-00000000000000000000000000000002-0000000000000002-01
[FormParams]
title: Hurl telemetry todo
HTTP 302
[Asserts]
header "Location" == "/"

GET http://localhost:8080/
traceparent: 00-00000000000000000000000000000003-0000000000000003-01
HTTP 200
[Asserts]
body contains "Hurl telemetry todo"
body not contains "An error occurred"

GET http://localhost:3200/api/search?q=%7Bresource.service.name%3D%22sqlpage%22%7D&limit=20
HTTP 200
[Asserts]
body contains "sqlpage"
body contains "\"traceID\":\"3\""

GET http://localhost:3200/api/search?q=%7Bresource.service.name%3D%22nginx%22%7D&limit=20
HTTP 200
[Asserts]
body contains "nginx"
body contains "\"traceID\":\"3\""

GET http://localhost:3200/api/traces/3
HTTP 200
[Asserts]
body contains "\"service.name\""
body contains "\"sqlpage\""
body contains "\"nginx\""
body contains "\"name\":\"GET /\""
body contains "\"name\":\"SELECT\""
body contains "\"kind\":\"SPAN_KIND_CLIENT\""
body contains "\"db.query.text\""
body contains "\"db.system.name\""
body contains "\"db.operation.name\""

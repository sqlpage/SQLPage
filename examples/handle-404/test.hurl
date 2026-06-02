GET http://localhost:8080/api/
HTTP 200
[Asserts]
body contains "Welcome to the API"
body contains "Back home"
body not contains "An error occurred"

GET http://localhost:8080/api/does/not/actually/exist
HTTP 200
[Asserts]
body htmlUnescape contains "\"serving_file\":\"api/404.sql\""
body htmlUnescape contains "\"request_path\":\"/api/does/not/actually/exist\""
body contains "Back home"
body not contains "An error occurred"

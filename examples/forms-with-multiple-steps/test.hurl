GET http://localhost:8080/hidden/step_1.sql
HTTP 200
[Asserts]
body contains "name"
body contains "step_2.sql"

POST http://localhost:8080/hidden/step_2.sql
[FormParams]
name: Hurl User
HTTP 200
[Asserts]
body contains "Hey Hurl User! what is your email?"
body contains "step_3.sql"

POST http://localhost:8080/hidden/step_3.sql
[FormParams]
name: Hurl User
email: hurl@example.com
HTTP 200
[Asserts]
body contains "How old are you, Hurl User?"
body contains "finish.sql"

POST http://localhost:8080/hidden/finish.sql
[FormParams]
name: Hurl User
email: hurl@example.com
age: 42
HTTP 200
[Asserts]
body contains "Welcome, Hurl User!"
body contains "Existing users"
body contains "hurl@example.com"

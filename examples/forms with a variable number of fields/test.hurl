GET http://localhost:8080/order_form.sql
HTTP 200
[Asserts]
body contains "Pass an order"
body contains "Quantity of Invisible Umbrella"
body contains "Quantity of Silent Alarm Clock"
body contains "Quantity of Pet Rock 2.0"
body not contains "An error occurred"

POST http://localhost:8080/order_insert.sql
[FormParams]
Name: Ada Lovelace
Email: ada@example.com
1: 2
2: 1
3: 0
HTTP 302
[Captures]
location: header "Location"
[Asserts]
header "Location" matches "^orders\\.sql\\?id=\\d+$"

GET http://localhost:8080/{{location}}
HTTP 200
[Asserts]
body contains "Thanks Ada Lovelace!"
body contains "2 x Invisible Umbrella"
body contains "Subtotal: 2 x 99.99"
body contains "199.98"
body contains "1 x Silent Alarm Clock"
body contains "Subtotal: 1 x 49.95"
body contains "Total: 249.93 €"
body not contains "Pet Rock 2.0"
body not contains "An error occurred"

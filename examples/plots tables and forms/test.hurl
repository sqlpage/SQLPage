GET http://localhost:8080/

HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "SQLPage Demo"
body contains "Revenue per country"
body contains "John"
body contains "Jane"
body contains "1 times 1"
body contains "10 times 10"
body contains "New User"
body not contains "An error occurred"

GET http://localhost:8080/?x=3

HTTP 200
[Asserts]
body contains "1 times 3"
body contains "10 times 3"
body contains "New User"
body not contains "An error occurred"

POST http://localhost:8080/
Content-Type: application/x-www-form-urlencoded
```
age=42&First%20Name=Ada&Last%20name=Lovelace&favorite_food=cake&checks%5B%5D=1&checks%5B%5D=2
```

HTTP 200
[Asserts]
body contains "Ada"
body contains "1"
body contains "2"
body not contains "An error occurred"

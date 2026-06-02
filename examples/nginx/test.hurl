# nginx proxies SQLPage dynamic pages and rewrites pretty post URLs.
GET http://localhost:8080/
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "Blog Posts"
body contains "First Post"
body contains "Hello World"

GET http://localhost:8080/post/1
HTTP 200
[Asserts]
header "Content-Type" contains "text/html"
body contains "Post Details"
body contains "This is the content of the first post."
body contains "Great post!"
body contains "Add a comment"

# /static/ is served by nginx directly, not proxied to SQLPage.
GET http://localhost:8080/static/index.sql
HTTP 404
[Asserts]
body contains "nginx"

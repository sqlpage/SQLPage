GET http://localhost:8080
HTTP 200
[Asserts]
body contains "Rich text editor"
body contains "Create a new blog post"
body contains "Blog posts"
body not contains "An error occurred"

POST http://localhost:8080/create_blog_post
[FormParams]
title: Hurl Rich Post
content: Hurl **rich** content
HTTP 302
[Captures]
location: header "Location"
[Asserts]
header "Location" matches "^post\\?id=\\d+$"

GET http://localhost:8080/{{location}}
HTTP 200
[Asserts]
body contains "Hurl Rich Post"
body contains "Hurl"
body contains "rich"
body contains "Edit"
body not contains "An error occurred"

POST http://localhost:8080/edit?id=1
[FormParams]
title: Hurl Updated Post
content: Updated rich content
HTTP 302
[Asserts]
header "Location" == "post?id=1"

GET http://localhost:8080/post?id=1
HTTP 200
[Asserts]
body contains "Hurl Updated Post"
body contains "Updated rich content"
body not contains "An error occurred"

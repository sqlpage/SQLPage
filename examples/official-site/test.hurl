GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "SQLPage"
body contains "Documentation"
body contains "Run it"
body contains "Instant by default."
body contains "Override any component."
body contains "Copy files. Or don't."
xpath "string(//button[@id='instant-tab']/@aria-selected)" == "true"
body contains "/assets/landing/js/main.js"
body not contains "An error occurred"

GET http://localhost:8080/documentation.sql
HTTP 200
[Asserts]
body contains "SQLPage v"
xpath "string(//main//a[normalize-space()='Components']/@href)" == "/components.sql"
xpath "string(//main//a[normalize-space()='Functions']/@href)" == "/functions.sql"
body not contains "An error occurred"

GET http://localhost:8080/configuration
HTTP 200
[Asserts]
xpath "//*[@id='listen_on']//a[@href='#listen_on']" count == 1
xpath "normalize-space(//*[@id='listen_on']/dd)" contains "443"
xpath "//*[@id='database_connection_idle_timeout_seconds']" count == 1
body not contains "An error occurred"

GET http://localhost:8080/search.sql?search=LISTEN_ON
HTTP 302
[Asserts]
header "Location" == "/configuration#listen_on"

GET http://localhost:8080/components.sql
HTTP 200
[Asserts]
xpath "string(//main//h3/a[span='shell']/@href)" == "/component.sql?component=shell"
body not contains "An error occurred"

GET http://localhost:8080/component.sql?component=form
HTTP 200
[Asserts]
body contains "The form component"
body contains "Building forms in SQL"
body not contains "An error occurred"

GET http://localhost:8080/component.sql?component=unknowncomponent
HTTP 302
[Asserts]
header "Location" == "component_not_found.sql?component=unknowncomponent"

GET http://localhost:8080/component_not_found.sql?component=unknowncomponent
HTTP 404
[Asserts]
header "X-Robots-Tag" == "noindex"
xpath "normalize-space(//main//h1)" == "Component not found"
xpath "//main//a[@href='/components.sql']" count == 1
xpath "//main//img[starts-with(@src,'http') or starts-with(@src,'//')]" count == 0
xpath "//main//*[normalize-space(@role)='alert']" count == 0

GET http://localhost:8080/functions.sql?q=variables
HTTP 200
[Asserts]
xpath "normalize-space(//main//h3/a[@href='/functions.sql?function=variables'])" == "variables([method])"

GET http://localhost:8080/functions.sql?function=url_encode
HTTP 200
[Asserts]
body contains "sqlpage.url_encode"
body contains "Parameters"
body not contains "An error occurred"

GET http://localhost:8080/llms.txt
HTTP 200
[Asserts]
header "Content-Type" == "text/markdown; charset=utf-8"
body contains "### [sqlpage.send_mail()](/functions.sql?function=send_mail)"
body contains "- `message` (JSON) **REQUIRED**: A JSON object containing the email to send."
body contains "> ## Building forms in SQL"
body contains "> ##### `reusable.sql`"
body contains "> ```sql\n> "

GET http://localhost:8080/search.sql?search=form
HTTP 302
[Asserts]
header "Location" == "/component.sql?component=form"

GET http://localhost:8080/search.sql?search=SQLPage.cookie()
HTTP 302
[Asserts]
header "Location" == "/functions.sql?function=cookie"

GET http://localhost:8080/search.sql?search=empty_description
HTTP 200
[Asserts]
xpath "//main//a[@href='/component.sql?component=list#top-level-parameters-empty_description']" count == 1

GET http://localhost:8080/search.sql?search=no-such-doc-4283
HTTP 200
[Asserts]
xpath "//form[@role='search']" count == 1
xpath "string(//main//input[@name='search']/@value)" == "no-such-doc-4283"
body contains "No results for"

GET http://localhost:8080/component.sql?component=list
HTTP 200
[Asserts]
xpath "//*[@id='top-level-parameters-empty_description']//dt//a[@href='#top-level-parameters-empty_description']" count == 1
xpath "//*[@id='row-level-parameters-title']" count == 1
xpath "//*[@id='top-level-parameters-title']" count == 1

GET http://localhost:8080/search.sql?search=migrations
HTTP 200
[Asserts]
xpath "//main//a[@href='/your-first-sql-website/migrations.sql']" count == 1
body not contains "An error occurred"

GET http://localhost:8080/your-first-sql-website/
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)
HTTP 200
[Asserts]
header "Vary" contains "User-Agent"
xpath "//main//h1" count == 1
xpath "//*[@id='connect-to-a-custom-database']" count == 1
xpath "//*[@id='download']//code[normalize-space()='brew install sqlpage']" count == 1

GET http://localhost:8080/your-first-sql-website/?os=windows&method=homebrew
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)
HTTP 200
[Asserts]
xpath "//*[@id='download']//a[normalize-space()='sqlpage-windows.zip' and contains(@href,'/releases/latest/download/sqlpage-windows.zip')]" count == 1

GET http://localhost:8080/your-first-sql-website/
User-Agent: Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 Mobile Safari/537.36
HTTP 200
[Asserts]
xpath "//*[@id='download']//nav[@aria-label='Installation target']//a[not(@aria-current)]" count == 4
xpath "//*[@id='download']//a[contains(@href,'/releases/latest/download/')]" count == 0

GET http://localhost:8080/your-first-sql-website/?os=any
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)
HTTP 200
[Asserts]
xpath "//*[@id='download']//nav[@aria-label='Installation target']//a[not(@aria-current)]" count == 4

GET http://localhost:8080/examples/tabs/?db=postgres
HTTP 302
[Asserts]
header "Location" == "/examples/?db=postgres#projects"

GET http://localhost:8080/examples/?db=postgres
HTTP 200
[Asserts]
body contains "Developer UI"
body not contains "Roundest Pokemon"

GET http://localhost:8080/not-a-real-doc-page
HTTP 404
[Asserts]
body contains "Not Found"
body not contains "An error occurred"

GET http://localhost:8080/examples/authentication/
HTTP 302
[Asserts]
header "Location" == "login.sql"

GET http://localhost:8080/examples/authentication/login.sql
HTTP 200
[Asserts]
body contains "Demo Login Form"
body contains "admin"
body not contains "An error occurred"

POST http://localhost:8080/examples/authentication/create_session_token.sql
[FormParams]
username: admin
password: admin
HTTP 302
[Asserts]
header "Location" == "/examples/authentication"

GET http://localhost:8080/examples/authentication/
HTTP 200
[Asserts]
body contains "You are logged in as admin"
body contains "Log out"
body not contains "An error occurred"

GET http://localhost:8080/examples/authentication/logout.sql
HTTP 302
[Asserts]
header "Location" == "login.sql"

GET http://localhost:8080/landing-demos/demo.sql?component=table
HTTP 200
[Asserts]
body contains "Acme Corp"
body contains "data-pre-init=\"table\""
body not contains "An error occurred"

GET http://localhost:8080/landing-demos/demo.sql?component=chart
HTTP 200
[Asserts]
body contains "Tickets resolved"
body contains "2026-09-30"
body not contains "An error occurred"

POST http://localhost:8080/landing-demos/save.sql
[FormParams]
name: Ada
team: Query crew
HTTP 200
[Asserts]
body contains "Hello, Ada!"
body contains "Query crew"
body contains "This preview is private to your submission."
body not contains "An error occurred"

# A fresh visitor always sees the seeded profile, never another submission.
GET http://localhost:8080/landing-demos/demo.sql?component=form
HTTP 200
[Asserts]
body contains "SQL builders"
body not contains "Query crew"

GET http://localhost:8080/landing-demos/demo.sql?component=big_number
HTTP 200
[Asserts]
body contains "98%"
body not contains "An error occurred"

GET http://localhost:8080/landing-demos/source.sql?component=../../configuration
HTTP 200
[Asserts]
jsonpath "$.source" contains "from customers"


GET http://localhost:8080/landing-demos/source.sql?component=form
HTTP 200
[Asserts]
jsonpath "$.save_source" contains "sqlpage.run_sql"
jsonpath "$.save_source" not contains "update profiles"

GET http://localhost:8080/landing-demos/demo.sql?component=catalog
HTTP 200
[Asserts]
body contains "component-catalog"
xpath "string(//a[span='form']/@href)" == "/component.sql?component=form"
body not contains "An error occurred"

POST http://localhost:8080/landing-demos/save.sql
[FormParams]
name:
team: Rejected input
HTTP 200
[Asserts]
body contains "Please check your profile"
body not contains "Hello,"

GET http://localhost:8080/landing-demos/demo.sql?component=form
HTTP 200
[Asserts]
body contains "SQL builders"
body not contains "Rejected input"

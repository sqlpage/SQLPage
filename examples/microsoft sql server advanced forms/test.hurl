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
[Captures]
unicode_group_id: xpath "string(//select[@name='Memberships[]'][@multiple]/option[normalize-space(.)=\"L'équipe 東京\"]/@value)"
[Asserts]
xpath "//code[@class='sqlpage-error-description']" not exists
variable "unicode_group_id" matches "^[0-9]+$"
xpath "string(//div[contains(@class, 'alert-success')][.//h4='Group created']//div[@class='alert-description'])" == "L'équipe 東京"
xpath "//div[contains(@class, 'alert-success')][.//h4='Group created']//a[@href='#group-{{unicode_group_id}}']" exists
xpath "//div[@id='group-{{unicode_group_id}}']" exists

GET http://127.0.0.1:8080/
Accept: application/json
HTTP 200
[Asserts]
jsonpath "$[4].id" == "group-{{unicode_group_id}}"
jsonpath "$[4].title" == "L'équipe 東京"
jsonpath "$[4].description" == null

# SQL Server rejects descriptions longer than the column's 80 characters.
POST http://127.0.0.1:8080/
[FormParams]
Name: Rejected group
GroupDescription: This description is more than eighty characters long and exceeds the database column size.
HTTP 200
[Asserts]
xpath "string(//code[@class='sqlpage-error-description'])" contains "would be truncated"
xpath "//div[contains(@class, 'alert-success')][.//h4='Group created']" not exists

POST http://127.0.0.1:8080/
[FormParams]
Name: Recovered group
GroupDescription: Saved after a failed write
HTTP 200
[Captures]
recovered_group_id: xpath "string(//select[@name='Memberships[]'][@multiple]/option[normalize-space(.)='Recovered group']/@value)"
[Asserts]
xpath "//code[@class='sqlpage-error-description']" not exists
xpath "//select[@name='Memberships[]'][@multiple]/option[@value='{{unicode_group_id}}']" exists
variable "recovered_group_id" matches "^[0-9]+$"

GET http://127.0.0.1:8080/
Accept: application/json
HTTP 200
[Asserts]
jsonpath "$[*].title" not contains "Rejected group"
jsonpath "$[5].id" == "group-{{recovered_group_id}}"
jsonpath "$[5].title" == "Recovered group"
jsonpath "$[5].description" == "Saved after a failed write"

# A stale group selection rejects the whole onboarding transaction.
POST http://127.0.0.1:8080/
Content-Type: application/x-www-form-urlencoded
`UserName=Report+Reader&Memberships%5B%5D={{unicode_group_id}}&Memberships%5B%5D=-1`
HTTP 200
[Asserts]
xpath "string(//code[@class='sqlpage-error-description'])" contains "FK_group_members_groups"

GET http://127.0.0.1:8080/
Accept: application/json
HTTP 200
[Asserts]
jsonpath "$[*].title" not contains "Report Reader"

# Submit two options rendered from FOR JSON PATH. SQLPage binds the selections
# as a JSON array and OPENJSON expands it into membership rows.
POST http://127.0.0.1:8080/
Content-Type: application/x-www-form-urlencoded
`UserName=Report+Reader&Memberships%5B%5D={{unicode_group_id}}&Memberships%5B%5D={{recovered_group_id}}`
HTTP 200
[Asserts]
xpath "//code[@class='sqlpage-error-description']" not exists

POST http://127.0.0.1:8080/
[FormParams]
UserName: Élodie
HTTP 200
[Asserts]
xpath "//code[@class='sqlpage-error-description']" not exists
xpath "//p[normalize-space()='No groups assigned']" exists

GET http://127.0.0.1:8080/
Accept: application/json
HTTP 200
[Asserts]
jsonpath "$[10].title" == "Report Reader"
jsonpath "$[10].description" == "L'équipe 東京, Recovered group"
jsonpath "$[11].title" == "Élodie"
jsonpath "$[11].description" == null

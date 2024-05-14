INSERT INTO
    sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES
    (
        'request_method',
        '0.20.6',
        'http-get',
        'Returns the HTTP request method (GET, POST, etc.) used to access the page.

# HTTP request methods

HTTP request methods (also known as verbs) are used to indicate the desired action to be performed on the identified resource. The most common methods are:
 - **GET**: retrieve information from the server. This is the default method used by browsers when you click on a link.
 - **POST**: submit data to be processed by the server. This is the default method used by browsers when you submit a form.
 - **PUT**: replace the current representation of the target resource with the request payload. Most commonly used in REST APIs.
 - **DELETE**: remove the target resource.
 - **PATCH**, **HEAD**, **OPTIONS**, **CONNECT**, **TRACE**: less common methods that are used in specific situations.

# Example

```sql
select ''redirect'' as component,
    ''/error?msg=expected+a+PUT+request'' as link,
where sqlpage.request_method() != ''PUT'';

insert into my_table (column1, column2) values (:value1, :value2);
```
'
    );
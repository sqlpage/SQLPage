select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component,
    '
# Performance of SQLPage applications

In SQLPage, the website author *declaratively* specifies the contents and behavior of the website using SQL queries,
as opposed to writing imperative code in a backend programming language like Java, Ruby, Python, or PHP.

This declarative approach allows SQLPage to offer **optimizations** out of the box that are difficult or time-consuming
to achieve in traditional web development stacks.

## Server-side rendering

SQLPage applications are [server-side rendered](https://web.dev/articles/rendering-on-the-web),
which means that the SQL queries are executed on the server, and the results are sent to the user''s browser
as HTML, which allows it to start rendering the page as soon as the first byte is received.
In contrast, many other web frameworks render the page on the client side, which means that the browser has to download
some HTML, then download some JavaScript, then execute the JavaScript, then make more requests,
then process the responses before it can start rendering the actual data the user is interested in.
This can lead to loading times that are several times longer than a SQLPage application.

### Streaming

SQLPage applications will often feel faster than even equivalent applications written even in alternative server-side rendering
frameworks, because SQLPage streams the results of the SQL queries to the browser as soon as they are available.

Most server-side rendering frameworks will first wait for all the SQL queries to finish, then render the page in memory
on the server, and only then send the HTML webpage to the browser. If a page contains a long list of items, the user
will have to wait for all the items to have been fetched from the database before seeing anything on the screen.
In contrast, SQLPage will start sending the first item as HTML to the browser as soon as it is available,
and the browser will start rendering it immediately.

## Compiled SQL queries

SQLPage prepares all your SQL queries only once, when they are first executed, and then caches the prepared statements
for future use. This means that the database does not have to parse the SQL queries, check their syntax, and create
an execution plan every time an user requests a page. 

When an user loads a page, all SQLPage has to do is tell the database: "Hey, do you remember that query we talked about
earlier? Can you give me the results for these specific parameters?". This is much faster than sending the whole SQL query
string to the database every time.

## Compiled templates

SQLPage also caches the compiled component templates that are used to generate the HTML for your website.
Both [built-in components](/documentation.sql) and [custom components](/custom_components.sql) you write yourself are parsed just once, and
compiled to an efficient memory representation that can be reused for every request.

## Processing data as fast as your CPU can go

In a traditional web development stack, the code you write in a high-level language has to be interpreted by a runtime
again and again every time a request is made to your website.
In contrast, SQLPage is entirely written in Rust, a compiled language that is known for its speed and safety guarantees.
The SQLPage binary you download already contains the optimized machine code that your cpu understands natively.


You traditionnally had to choose between the speed of compiled languages,
and the ease of use and developer productivity of interpreted languages. SQLPage offers the best of both worlds:
 - requests are processed as fast as if you had manually written the code in Rust,
 - you just have to write SQL queries, which are orders of magnitude easier to write and maintain than C++ or Rust code.

All the logic required to serve a request to your application will be executed either in rust in SQLPage 
itself, or in the database, which is also written in a performant compiled language.

## SQL query elimination

In the SQLPage model, you will often find yourself writing SQL queries that are entirely static,
and the results of which do not depend on the contents of the database.
For example, when you open a list with `SELECT ''list'' as component;`, you already know that the query will return
a single row with a single column, containing the string `''list''`, no matter what the contents of the database are.
SQLPage is able to detect these static queries, and it will not execute them on the database at all.
Instead, it will cache statically known results, and process them as soon as the page is requested, without any database
interaction.

## Key Takeaways

SQLPage offers a radically different approach to web development,
resolving the classical tension between performance and ease of use.

By leveraging a declarative approach, server-side rendering, and advanced optimization techniques, SQLPage enables:

* **Faster page loads**: Long loading times make your website feel sluggish and unresponsive, causing users to leave.
* **Easier development**: Focus on writing SQL queries; all the heavy lifting is done for you.
* **Cost effective**: SQLPage''s low CPU and memory usage means you can host your website extremely cheaply, even if it gets significant traffic.

## Ready to get started?

[Build your fast, secure, and beautiful website](/your-first-sql-website) with SQLPage today!
' as contents_md;

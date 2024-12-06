# 📢 Announcing SQLPage: Build Dynamic SQLite Applications in SQL

Hello everyone ! 

I'm not sure whether announcements like this are allowed here; feel free to delete this post if they are not.

I wanted to introduce a cool piece of open source software I have been working on for a long time, and that is now ready for more general use.

It's called [SQLPage](https://sql-page.com), and it lets you build a full web application on top of your SQLite database using nothing more than standard SQL queries. 

# SQLPage: [build a website in SQL](https://sql-page.com)

[![code-screenshots](https://github.com/sqlpage/SQLPage/assets/552629/03ed65bc-ecb1-4c01-990e-d6ab97be39c0)](https://github.com/sqlpage/SQLPage)


## ❓ What is it ?

It is a small opensource web server distributed as a single binary that executes your `.sql` files, and renders the results using nice web components (tables, lists, forms, plots, ...).

Of course, if you are making a huge application with a complex business logic, SQLPage is not for you. But if you have a SQLite database lying around that you would want to share access to through a nice UI without spending too much time on it, you should try it.


## Features

 - **🗄️ SQL-only**: Create full web applications with a sleek frontend without touching HTML, CSS, or JavaScript.
 - **📝 Full SQL Support**: Auto-generated Web UI. Write only raw SQL queries.
 - **🔄 Integrated**: Supports any existing SQLite database, including using SQLlite extensions, leveraging its data using a standard .sql file.
 - **🌐 Web Standards Support**: Read and write HTTP cookies, manage user authentication, handle form submissions, and URL parameters.
 - **🚀🔒 Fast And Secure**: Written in Rust, ensuring no memory corruption, SQL injections, or XSS vulnerabilities.

##  Use Cases

Some cool things people are building with SQLPage:

 - 📊 Internal Dashboards: Empower teams with data-driven insights.
 - 📈 Small Business Intelligence Apps: Build powerful applications for analysis and exploration.
 - 🗂️ Admin Interfaces: Manage and interact with SQLite data effectively.
 - 🎮 A Game: Rapidly validate and iterate on the idea.
 - 📚 Knowledge Management Tool: Replace an Excel file with a real database quickly

## Open-Source

 - [Official project page](https://sql-page.com)
 - [Source Code on Github](https://github.com/sqlpage/SQLPage)
 - [Examples](https://github.com/sqlpage/SQLPage/tree/main/examples)
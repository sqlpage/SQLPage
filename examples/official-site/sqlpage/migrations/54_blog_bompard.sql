
INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'Alexis''s performance monitoring tool',
        'Alexis built a performance monitoring tool with SQLPage',
        'shirt',
        '2024-10-26',
        '
# How I Built And Deployed An Exhaustive Performance Monitoring Tool For a 100M€/year Company Using SQL Queries Only

### What is SQLPage ?

> [SQLPage](http://sql.datapage.app) allows anyone with SQL skills to build and deploy digital tools (websites, data applications, dashboards, user forms…) using only **SQL queries**. Official website: [https://sql.datapage.app/](https://sql.datapage.app/)

SQLPage eliminates the need to learn server languages, HTML, CSS, JavaScript, or front-end frameworks, and instead uses SQL to generate modern UI layouts populated with database query results. You get native SQL interactions with the database, without all the other layers that typically get in the way.

The execution of the project is straightforward: simply run a single executable without any installation dependencies. Everything from authentication to security, and even HTTPS termination is automated. The code required to complete most real-world development tasks is minimal and seamless.

Finally, it’s open source with an MIT license.

### Why SQLPage became a game-changer for me, as a Head of Data

As a Head of Data at a mid-size company, I understand the challenge of juggling multiple tools — often expensive and proprietary — alongside a variety of dashboards. Building an **all-in-one**, **user-friendly**, **mobile-compatible** platform for data monitoring and visualization that serves everyone, from C-level executives to store managers, is no small feat.

The struggle intensifies when teams are small and lack coding skills or experience with diverse tech stacks. A typical data flow in a digital-native company involves several teams, specialized skills, and costly tools:

![](https://cdn-images-1.medium.com/max/800/1*1IoXc8-07rqXO3yvKC13nQ.png)

*Typical Data Flow of digital native companies.

SQLPage changes this by allowing data professionals to use the same language — SQL — across the entire process, from building data pipelines to creating fully functional digital tools. Data analysts, scientists, business analysts, DBAs, and IT teams already have the expertise to craft their own custom data applications from the ground up.

### Building an all-in one monitoring tool using SQL-queries only
[![youtube](https://github.com/user-attachments/assets/1afe36d7-9deb-40fc-a174-7a869348500b)](https://www.youtube.com/embed/R-5Pej8Sw18?si=qgxacwip2Mm-0wC7)

*Excerpt from a series of videos explaining how to build and deploy your first digital tool with SQLPage* ([https://www.youtube.com/@SQLPage](https://www.youtube.com/@SQLPage)).


I am using SQLPage to build a 360° Performance tool for my company, integrating data from multiple sources — Revenue, traffic, marketing investments, live performance monitoring, financial targets, images of top sold products, Google Analytics for the online traffic… — .

#### With SQLPage, I can:

-   **Centralize** all company data in one tool for visualizations, year-over-year comparisons, financial targets, and more.
-   **Provide tailored insights**: A store owner can instantly access last year’s performance and top-selling products, while the e-commerce director can track conversion rate history. SQLPage’s pre-built components offer limitless possibilities for displaying results.
-   **Perform CRUD operations**: Unlike traditional BI tools, SQLPage not only displays data but also allows users to interact with it — inputting data, such as comments or updates, directly through the interface. This capability to both display and collect data is a significant enhancement over traditional BI tools, which typically do not support data input.
-   **Ensure a single source of truth**: By connecting directly to the database, SQLPage avoids discrepancies between dashboards, ensuring all teams work with consistent and accurate data.

Here are some pages I built using only SQL queries, allowing everyone in the company to instantly access any level of information, from the fiscal year 2024 revenue trends to the top-selling products in Marseille in October 2022.

![](https://cdn-images-1.medium.com/max/800/1*MkORbAC7oGEG-8I1mthu6A.png)  
    
*Performance of different channels vs last year and best sellers.

![](https://cdn-images-1.medium.com/max/800/1*_3-g1om_p9ghXhdcw0zHmw.png)  
    
*Examples of views built with SQLPage to provide a 360° tool for the company.

### How Does It Work ?

The process in SQLPage follows a simple pattern:

> 1) Select a component

> 2) Write a query to populate the selected component with data

You can find the full list of components: [https://sql.datapage.app/documentation.sql](https://sql.datapage.app/documentation.sql)

Here’s an example of a parameterized SQL query that uses the “chart” component, along with the query to feed data into it:

  

```sql 
-- Chart Component  
SELECT ''chart'' AS component,  
       CONCAT(''Daily Revenue from '', $start_date_comparison, '' to '', $end_date_comparison) AS title,  
       ''area'' AS type,  
       ''indigo'' AS color,  
       5 AS marker,  
       0 AS ymin;  
  
-- Chart Data Query  
SELECT DATE(business_date) AS x,  
       ROUND(SUM(value), 2) AS y  
FROM data_example  
WHERE DATE(business_date) BETWEEN $start_date_comparison AND $end_date_comparison  
  AND variable_name = ''CA''  
GROUP BY DATE(business_date)  
ORDER BY x ASC;  
  
-- NB: The variables $start_date_comparison and $end_date_comparison are  
-- defined dynamically in the SQL script
```
And the result:

![Example of a SQL-generated page using the “graph” and the “table” components](https://cdn-images-1.medium.com/max/800/1*mtgJNP7DSOmMnq0iu6dDcg.png)

*Example of a SQL-generated page using the components “graphs” and “tables”.*

That’s it! Each component comes with customizable parameters, allowing you to tailor the display. As shown in the screenshot, links are clickable, enabling users to add data, such as leaving a comment for a specific date.

The ability to perform CRUD operations and interact directly with databases is a game changer compared to traditional BI tools. You can try it yourself by clicking “add” in the column “COMMENT THIS YEAR” [https://demo-test.datapage.app/lets_see_some_graphs.sql](https://demo-test.datapage.app/lets_see_some_graphs.sql)

### What About GenAI ?

I couldn’t write an article about data in 2024 without mentioning GenAI. The great news is that SQLPage, relying solely on SQL queries, is naturally GenAI-friendly. In fact, I rarely write SQL queries myself anymore — I let GenAI handle that. My workflow in SQLPage now becomes:

> 1) Select a component

> 2) Ask a GenAI tool to write the query I need

![](https://cdn-images-1.medium.com/max/800/1*mg45EO7XCVPNiuIQ_5Xg0Q.png)

*Example of Generated SQL to display a specific format of numbers.*

### How to Host Your SQLPage Application

Once my app was ready, I could have chosen to host it myself on any server for a few euros a month, but I opted for SQLPage’s official hosting service, DataPage ([https://datapage.app/](https://beta.datapage.app/)), which is fully managed and very convenient. My app was hosted at _domainname.datapage.app._ The service includes a Postgres database, allowing you to either store your data on the server or connect directly to your existing database (Microsoft SQL Server, SQLite, Postgres, MySQL, etc).

### What Difficulties Can Be Encountered With SQLPage

While SQLPage simplifies the process of building digital tools, it does come with some challenges.

As applications grow in complexity, so do the SQL queries required to power them, which can result in long and intricate scripts. Additionally, to fully leverage SQLPage, you need to understand how its components work, especially if user input is involved. Developers should be comfortable with creating tables in a database, writing `INSERT` queries, and managing data effectively. Without a solid grasp of these fundamentals, building more advanced apps can become a bit overwhelming.

### Conclusion

With SQLPage, any company with a database and one employee who knows how to query it has the tools and workforce to build and deploy virtually any digital tool.

In this article, I focused on creating an enhanced Business Intelligence tool, but SQLPage’s versatility goes far beyond that. It is being used to build a planning tool for lumberjacks in Finland, a monitoring app for a South African transport and logistics company, by archaeologists to input excavation data in the field…

What all these projects have in common is that they were built by a single person, using nothing but SQL queries. If you’re ready to streamline your processes and build powerful tools with ease, SQLPage is worth exploring further.

### Useful links

-   🏡Official website [https://sql.datapage.app](https://sql.datapage.app/)
-   🔰Quick start (written by [Nick Antonaccio](https://medium.com/u/b6a791990395)): [https://learnsqlpage.com/sqlpage_quickstart.html](https://learnsqlpage.com/sqlpage_quickstart.html)
-   📹Youtube tutorial videos on SQLPage channel: [https://www.youtube.com/@SQLPage/playlists](https://www.youtube.com/@SQLPage/playlists)
-   🤓github: [https://github.com/sqlpage/SQLPage/](https://github.com/sqlpage/SQLPage/)
-   ☁️Host your applications: [https://datapage.app](https://datapage.app)
');

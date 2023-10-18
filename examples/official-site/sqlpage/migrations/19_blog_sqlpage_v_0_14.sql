INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES (
        'SQLPage v0.14',
        'SQLPage v0.14 has nice customizable buttons and time series plots.',
        'refresh',
        '2023-10-19',
        '
## SQLPage v0.14.0

 - Better support for time series in the [chart](https://sql.ophir.dev/documentation.sql?component=chart#component) component. You can now use the `time` top-attribute to display a time series chart
 with smart x-axis labels.
 - **New component**: [button](https://sql.ophir.dev/documentation.sql?component=button#component). This allows you to create rows of buttons that allow navigation between pages.
 - Better error messages for Microsoft SQL Server. SQLPage now displays the line number of the error, which is especially useful for debugging long migration scripts.
 - Many improvements in the official website and the documentation.
    - Most notably, the documentation now has syntax highlighting on code blocks (using [prism](https://prismjs.com/) with a custom theme made for tabler). This also illustrates the usage of external javascript and css libraries in SQLPage. See [the shell component documentation](https://sql.ophir.dev/documentation.sql?component=shell#component).
    - Better display of example queries in the documentation, with smart indentation that makes it easier to read.
 - Clarify some ambiguous error messages:
   - make it clearer whether the error comes from SQLPage or from the database
   - specific tokenization errors are now displayed as such

');
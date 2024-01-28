
INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'New SQLPage, and a talk at PGConf.eu',
        'SQLPage v0.18.0 is out, and there is detailed introduction to SQLPage on youtube',
        'code-minus',
        '2023-08-03',
        '
[SQLPage](https://sql.ophir.dev) is a small web server that renders your SQL queries as beautiful interactive websites. This release has seen significant new features and fixes from new contributors, which is great and show the health of the project ! If you feel something is missing or isn''t working quite right, all your contributions are always welcome. 

On a side note, I [gave a talk about SQLPage last December at PGConf.eu](https://www.youtube.com/watch?v=mXdgmSdaXkg).
It is a great detailed introduction to SQLPage, and I recommend it if you want to learn more about the project.

1. **New `tracking` component for beautiful and compact status reports:** This feature adds a new way to display status reports, making them more visually appealing and concise. 
    1. ![screenshot](https://github.com/lovasoa/SQLpage/assets/552629/3e792953-3870-469d-a01d-898316b2ab32)


3. **New `divider` component to add a horizontal line between other components:** This simple yet useful addition allows for better separation of elements on your pages.
    1. ![image](https://github.com/lovasoa/SQLpage/assets/552629/09a2cc77-3b37-401f-ab3e-441637a2c022)

5. **New `breadcrumb` component to display a breadcrumb navigation bar:** This component helps users navigate through your website''s hierarchical structure, providing a clear path back to the homepage.
    1. ![image](https://github.com/lovasoa/SQLpage/assets/552629/cbf2174a-1d75-499e-9d6b-e111136dbbbc)

8. **Multi-column layouts with `embed` attribute in `card` component:** This feature enables you to create more complex and dynamic layouts within cards.
    1. ![image](https://github.com/lovasoa/SQLpage/assets/552629/3f4435f0-d89b-424e-8b8a-39385a61d5ad)


6. **Customizable y-axis step size in `chart` component with `ystep` attribute:** This feature gives you more control over the chart''s appearance, especially for situations with multiple series.
 
7. **Updated default graph colors for better distinction:** This enhancement ensures clarity and easy identification of different data series.

10. **ID and class attributes for all components for easier styling and referencing:** This improvement simplifies custom CSS customization and inter-page element linking.

11. **Implementation of `uploaded_file_mime_type` function:** This function allows you to determine the MIME type of a uploaded file.

12. **Upgraded built-in SQLite database to version 3.45.0:** This ensures compatibility with recent SQLite features and bug fixes. See [sqlite release notes](https://www.sqlite.org/releaselog/3_45_0.html)

13. **Unicode support for built-in SQLite database:** This enables case-insensitive string comparisons and lower/upper case transformations.

5. **Improved `card` component with smaller margin below footer text:** This fix ensures consistent and visually balanced card layouts.

        '
    );
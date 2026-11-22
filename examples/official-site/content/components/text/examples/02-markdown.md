---
rows:
  - component: text
    article: true
    contents_md: |+

      # Markdown in SQLPage

      ## Simple formatting

      SQLPage supports only plain text as column values, but markdown allows easily adding **bold**, *italics*, [external links](https://github.com/sqlpage/SQLPage), [links to other pages](/index.sql) and [intra-page links](#my-paragraph).

      ## Lists
      ### Unordered lists
      * SQLPage is easy
      * SQLPage is fun
      * SQLPage is free

      ### Ordered lists
      1. SQLPage is fast
      2. SQLPage is safe
      3. SQLPage is open-source

      ## Code
      ```sql
      SELECT 'list' AS component;
      SELECT name as title FROM users;
      ```

      ## Tables

      | SQLPage component | Description  | Documentation link  |
      | --- | --- | --- |
      | text | A paragraph of text. | [Documentation](https://sql-page.com/documentation.sql?component=text) |
      | list | A list of items. | [Documentation](https://sql-page.com/documentation.sql?component=list) |
      | steps | A progress indicator. | [Documentation](https://sql-page.com/documentation.sql?component=steps) |
      | form | A series of input fields. | [Documentation](https://sql-page.com/documentation.sql?component=form) |

      ## Quotes
      > Fantastic.
      >
      > — [HackerNews User](https://news.ycombinator.com/item?id=36194473#36209061) about SQLPage

      ## Images
      ![SQLPage logo](https://sql-page.com/favicon.ico)

      ## Horizontal rules
      ---

---

Rendering rich text using markdown

-- https://github.com/lovasoa/SQLpage/discussions/600
select 'table' as component, 'Link' AS markdown; -- uppercase Link
select '[It works !](https://example.com)

[comment]: <> (This is a comment. If this is visible, there is an error in markdown rendering)' as link; -- lowercase "link".
-- If the markdown is not rendered, the page will contain the string "error" and trigger a test failure.

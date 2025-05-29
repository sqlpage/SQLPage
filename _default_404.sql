SELECT
    'shell' as component,
    'Page Not Found' as title,
    'error-404' as body_class,
    '/' as link;

SELECT
    'empty_state' as component,
    'Page Not Found' as title,
    '404' as header,
    'The page you were looking for does not exist.' as description_md,
    'Go to Homepage' as link_text,
    'home' as link_icon,
    '/' as link;

select
    'text' as component,
    '
> **Routing Debug Info**  
> When a URL is requested, SQLPage looks for matching files in this order:
> 1. **Exact filename match** (e.g. `page.html` for `/page.html`)
> 2. **For paths ending with `/`**:
>    - Looks for `index.sql` in that directory (e.g. `/dir/` → `dir/index.sql`)
> 3. **For paths without extensions**:
>    - First tries adding `.sql` extension (e.g. `/dir/page` → `dir/page.sql`)
>    - If not found, redirects to add trailing `/` (e.g. `/dir` → `/dir/`)
> 4. **If no matches found**:
>    - Searches for `404.sql` in current and parent directories (e.g. `dir/x/y/` could use `dir/404.sql`)
>
> Try creating one of these files to handle this route.
' as contents_md;
select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', 'Hosted SQLPage',
    'description', 'Set up a SQLPage website online with managed hosting.',
    'section', 'Guides', 'current', 'Managed hosting'
)) as properties;

select 'docs-intro' as component,
    'Hosted SQLPage' as title,
    'Publish a SQLPage website with DataPage without installing software or configuring a server.' as description,
    '/assets/landing/assets/scene-first-frame.webp' as image,
    'https://datapage.app' as link,
    'Host your website' as link_text;

SELECT 'text' as component,
    '## Run SQLPage in your browser

Replit is an online development environment that lets you run SQLPage without a local installation.

Try [the SQLPage repl](https://replit.com/@pimaj62145/SQLPage).
Click *Use template* to create your own editable copy of the demo website, then click *Run* to see the result.
On the left side you can edit the SQLPage code, on the right side you can see the result.
' as contents_md;

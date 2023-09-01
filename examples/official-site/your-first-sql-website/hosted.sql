select 'shell' as component,
    'SQLPage: get started!' as title,
    'database' as icon,
    '/' as link,
    'en-US' as lang,
    'Hosted SQLPage: set-up a SQLPage website in three clicks.' as description,
    'documentation' as menu_item,
    'Poppins' as font;

SELECT 'hero' as component,
    'Hosted SQLPage' as title,
    'Work In Progress: We are working on a cloud version of SQLPage
    that will enable you to effortlessly set up your website online without the need to download any software or configure your own server.' as description,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/9/94/Baustelle_H%C3%B6lzla_6066312.jpg/1024px-Baustelle_H%C3%B6lzla_6066312.jpg' as image,
    'https://forms.gle/z1qmuCwdNT5Am7gp6' as link,
    'Get notified when we are ready' as link_text;

SELECT 'text' as component,
    'Try SQLPage online today' as title,
    '
If you want to fiddle around with SQLPage without installing anything on your computer, you can still try it out online today.

Repl.it is an online development environment that allows you to run SQLPage in your browser.

Try [the SQLPage repl](https://replit.com/@pimaj62145/SQLPage).
Click *Use template* to create your own editable copy of the demo website, then click *Run* to see the result.
On the left side you can edit the SQLPage code, on the right side you can see the result.
' as contents_md;
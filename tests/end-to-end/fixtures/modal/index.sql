SELECT 'text' AS component, 'Component lifecycle' AS contents WHERE $fragment IS NULL;

-- The simple modal and embedded form from 63_modal.sql, reusing the form fixture.
SELECT 'dynamic' AS component, JSON('[
    {"component":"modal","id":"my_modal","title":"A modal box","close":"Close"},
    {"contents":"I''m a modal window, and I allow you to display additional information or help for the user."},
    {"component":"button"},
    {"title":"Open a simple modal","link":"#my_modal"},
    {"component":"modal","id":"my_embed_form_modal","title":"Embeded form content","large":true,"embed":"/form/?_sqlpage_embed=1"}
]') AS properties
WHERE $fragment IS NOT NULL;

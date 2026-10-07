SELECT 'text' AS component, 'Component lifecycle' AS contents WHERE $fragment IS NULL;

-- Properties exercised by the toast documentation smoke assertions (76_toast.sql).
SELECT 'dynamic' AS component, JSON('[
    {"component":"toast","id":"toast-auto","title":"This is a SQLPage toast","description":"This toast will open automatically when the page loads.","icon":"check","color":"green"},
    {"component":"toast","id":"toast-dismissible","trigger":"persistent-error","title":"Could not save","description":"Review the highlighted fields and try again.","icon":"alert-triangle","color":"black","duration":0,"dismissible":true},
    {"component":"toast","id":"toast-nondismissible","trigger":"persistent-status","title":"Connection unavailable","description":"This persistent notification has no manual close control.","duration":0,"dismissible":false},
    {"component":"button"},
    {"title":"Show dismissible error","link":"#persistent-error","color":"red"},
    {"title":"Show non-dismissible status","link":"#persistent-status"},
    {"component":"toast","id":"toast-markdown","trigger":"rich-notifications","title":"Release available","description":"<strong>This fallback stays escaped</strong>","description_md":"Version **2.0** is ready. [Read the notes](https://example.com/releases).","color":"blue","duration":0},
    {"component":"toast","id":"toast-plain","trigger":"rich-notifications","description":"<strong>Plain text stays escaped</strong>","color":"white","duration":0},
    {"component":"button"},
    {"title":"Show rich notifications","link":"#rich-notifications"},
    {"component":"toast","id":"toast-stack-one","trigger":"queued notifications","title":"Import started","description":"Preparing records.","duration":0},
    {"component":"toast","id":"toast-stack-two","trigger":"queued notifications","title":"Import running","description":"Processing records.","duration":0},
    {"component":"toast","id":"toast-short","trigger":"queued notifications","title":"Temporary update","description":"This message closes shortly.","duration":2000},
    {"component":"button"},
    {"title":"Show queued notifications","link":"#queued notifications"},
    {"component":"toast","id":"toast-bottom-center","trigger":"bottom-notification","title":"Download ready","description":"Your export is ready.","position":"bottom-center","duration":0},
    {"component":"button"},
    {"title":"Show bottom notification","link":"#bottom-notification"}
]') AS properties
WHERE $fragment IS NOT NULL;

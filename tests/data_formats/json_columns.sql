select
    'columns' as component;

select
    'Pro Plan' as title,
    '€40' as value,
    'rocket' as icon,
    'For growing projects needing enhanced features' as description,
    JSON (
        '{"icon":"database","color":"blue","description":"1GB Database"}'
    ) as item,
    JSON (
        '{"icon":"headset","color":"green","description":"Priority Support"}'
    ) as item;
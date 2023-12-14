select 'shell' as component,
       'TinyTweeter' as title;

select 'form' as component,
       'Tweet' as validate;
select 'new_tweet' as name,
       'Your story' as label,
       'textarea' as type,
       'Tell me your story...' as placeholder;
select 'checkbox' as type,
       'Terms and conditions' as label,
       true as required;
       
insert into tweets (tweet)
select :new_tweet
where :new_tweet is not null;

select 'card' as component,
       'Tweets' as title,
       1 as columns;
select tweet as description
from tweets;
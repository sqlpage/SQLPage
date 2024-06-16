select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

create temporary table if not exists colors as select column1 as color, column2 as hex from (values
    ('blue', '#0054a6'), ('azure', '#4299e1'), ('indigo', '#4263eb'), ('purple', '#ae3ec9'), ('pink', '#d6336c'), ('red', '#d63939'), ('orange', '#f76707'), ('yellow', '#f59f00'), ('lime', '#74b816'), ('green', '#2fb344'), ('teal', '#0ca678'), ('cyan', '#17a2b8'),
    ('blue-lt', '#e9f0f9'), ('azure-lt', '#ecf5fc'), ('indigo-lt', '#eceffd'), ('purple-lt', '#f7ecfa'), ('pink-lt', '#fbebf0'), ('red-lt', '#fbebeb'), ('orange-lt', '#fef0e6'), ('yellow-lt', '#fef5e6'), ('lime-lt', '#f1f8e8'), ('green-lt', '#eaf7ec'), ('teal-lt', '#e7f6f2'), ('cyan-lt', '#e8f6f8'),
    ('gray-50', '#f8fafc'), ('gray-100', '#f1f5f9'), ('gray-200', '#e2e8f0'), ('gray-300', '#c8d3e1'), ('gray-400', '#9ba9be'), ('gray-500', '#6c7a91'), ('gray-600', '#49566c'), ('gray-700', '#313c52'), ('gray-800', '#1d273b'), ('gray-900', '#0f172a'),
    ('facebook', '#1877F2'), ('twitter', '#1da1f2'), ('linkedin', '#0a66c2'), ('google', '#dc4e41'), ('youtube', '#ff0000'), ('vimeo', '#1ab7ea'), ('dribbble', '#ea4c89'), ('github', '#181717'), ('instagram', '#e4405f'), ('pinterest', '#bd081c'), ('vk', '#6383a8'), ('rss', '#ffa500'), ('flickr', '#0063dc'), ('bitbucket', '#0052cc'), ('tabler', '#0054a6'),
    ('black', '#000000'), ('white', '#ffffff'), ('gray', '#808080'),
    ('primary', '#0054a6'), ('secondary', '#49566c'), ('success', '#2fb344'), ('info', '#17a2b8'), ('warning', '#f59f00'), ('danger', '#d63939'), ('light', '#f1f5f9'), ('dark', '#0f172a')
);

select 'card' as component, 'Colors' as title;
select color as title, hex as description, color as background_color
from colors;
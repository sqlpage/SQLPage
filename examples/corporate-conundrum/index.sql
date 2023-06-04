select 'shell' as component, * FROM shell;
SELECT 'hero' as component,
    'Welcome to Corporate Conundrum' as title,
    'Unleash your inner executive in this thrilling board game of corporate espionage. Make the right choices to lead your company to success!' as description,
    'New Game.sql' as link,
    'Start a New Game' as link_text;
SELECT 'Answer questions' as title,
    'Each turn, a question will be presented to the group. One player will be assigned as the infiltrator and receive a specific wrong answer. Engage in lively debates and exchange ideas to uncover the truth and make accurate decisions.' as description,
    'help-hexagon' as icon,
    'blue' as color;
SELECT 'Hidden Votes' as title,
    'After the discussion phase, all players submit their individual answers privately. The answers are revealed simultaneously, and points are awarded based on their proximity to the true answer.' as description,
    'file' as icon,
    'green' as color;
SELECT 'Role Assignment' as title,
    'The web app randomly assigns roles to players, designating one as the infiltrator. The infiltrator''s objective is to sway others toward incorrect answers, while the team tries to collaborate and deduce the correct answer.' as description,
    'spy' as icon,
    'red' as color;
SELECT 'Continuing Gameplay' as title,
    'The game progresses with new questions and role assignments, allowing each player to take turns as the infiltrator. The player with the highest score at the end of the predetermined number of rounds wins the game.' as description,
    'player-play' as icon,
    'purple' as color;


SELECT 'title' as component, 'Game rules' as contents;
SELECT 'steps' as component,
    1 as counter,
    'cyan' as color;
SELECT 'Create game' as title,
    'plus' as icon,
    'Create a new game from the home page.' as description;
SELECT 'Add players' as title,
    'user-plus' as icon,
    'Add players by their name, and send them their own unique game URL.' as description;
SELECT 'Answer questions' as title,
    'Answer trivia questions to get points. Don''t be fooled by the imposter.' as description,
    'help-hexagon' as icon;
SELECT 'Trick the others' as title,
    'When you become the imposter, try to trick the others into giving wrong answers.' as description,
    'help-hexagon' as icon;
SELECT 'The smartest wins' as title,
    'In the end, the game counts your points. You win if you tricked the others and did not get tricked.' as description,
    'brain' as icon;

select 'card' as component, 1 as columns;
SELECT 'Objective' as title, 'As a team of genuine employees, your goal is to make accurate decisions based on challenging questions. The infiltrator''s objective is to sway you toward incorrect answers.' as description;
SELECT 'Gameplay' as title, 'Each turn, a question will be presented to the group. One player, secretly assigned as the infiltrator, will receive a specific wrong answer. They must cunningly lead others astray, while you must collaborate and deduce the correct answer. Every player will be the infiltrator once during the game.' as description;
SELECT 'Discussion Phase' as title, 'Engage in lively debates and exchange ideas to uncover the truth. Analyze arguments, question motives, and use your critical thinking skills to navigate the murky waters of corporate deception.' as description;
SELECT 'Hidden Votes' as title, 'After the discussion phase, all players simultaneously submit their individual answers privately, without revealing them to others. Votes will not be revealed until the end of the game.' as description;
SELECT 'Scoring System' as title, 'Points are awarded based on the proximity of each player''s answer to the true answer.
If a player''s answer is closer to the true answer than to the wrong answer provided by the saboteur, they earn one point.
Conversely, if a player''s answer is closer to the wrong answer, they inadvertently contribute one point to the saboteur''s score.' as description;
SELECT 'Continuing Gameplay' as title, 'The game progresses with new questions and role assignments, allowing each player to take turns as the infiltrator. The player with the highest score at the end of the predetermined number of rounds wins the game.' as description;
select * FROM sqlpage_shell;
SELECT 'hero' as component,
    'Welcome to Corporate Conundrum' as title,
    'Unleash your inner executive in this thrilling board game of corporate espionage. Make the right choices to lead your company to success!' as description,
    'New Game.sql' as link,
    'Start a New Game' as link_text;
SELECT 'Lively discussions' as title,
    'Each turn, a question will be presented to the group. One player will be assigned as the infiltrator and receive a specific wrong answer. Engage in lively real-life debates and exchange ideas to uncover the truth and make accurate decisions.' as description,
    'help-hexagon' as icon,
    'blue' as color;
SELECT 'Hidden Votes' as title,
    'After the discussion phase, all players submit their individual answers privately. Points are awarded based on their proximity to the true answer.' as description,
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

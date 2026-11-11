-- Earlier builds stored Steam's placeholder answer for delisted apps
-- (appid 0) as a game row; nothing references appid 0.
DELETE FROM steam_games WHERE appid = 0;

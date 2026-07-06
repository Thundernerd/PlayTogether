CREATE TABLE owned_games (
    member_id TEXT NOT NULL REFERENCES members(id),
    appid INTEGER NOT NULL,
    playtime_forever INTEGER NOT NULL DEFAULT 0,
    rtime_last_played INTEGER,
    PRIMARY KEY (member_id, appid)
);

CREATE INDEX idx_owned_games_member_id ON owned_games(member_id);

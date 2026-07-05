CREATE TABLE play_groups (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    invite_code TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

CREATE TABLE members (
    id TEXT PRIMARY KEY NOT NULL,
    group_id TEXT NOT NULL REFERENCES play_groups(id),
    display_name TEXT NOT NULL,
    steam_id INTEGER,
    token TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

CREATE TABLE member_sync_jobs (
    member_id TEXT PRIMARY KEY NOT NULL REFERENCES members(id),
    status TEXT NOT NULL DEFAULT 'idle',
    phase TEXT,
    progress_done INTEGER NOT NULL DEFAULT 0,
    progress_total INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    updated_at TEXT NOT NULL
);

CREATE TABLE wishlist_items (
    member_id TEXT NOT NULL REFERENCES members(id),
    appid INTEGER NOT NULL,
    priority INTEGER NOT NULL,
    date_added INTEGER NOT NULL,
    PRIMARY KEY (member_id, appid)
);

CREATE TABLE steam_games (
    appid INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    short_description TEXT NOT NULL DEFAULT '',
    header_image_url TEXT NOT NULL DEFAULT '',
    tags_json TEXT NOT NULL DEFAULT '[]',
    raw_json TEXT NOT NULL DEFAULT '{}',
    fetched_at TEXT NOT NULL
);

CREATE TABLE member_selections (
    member_id TEXT NOT NULL REFERENCES members(id),
    appid INTEGER NOT NULL,
    wants_to_play INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (member_id, appid)
);

CREATE TABLE group_game_votes (
    group_id TEXT NOT NULL REFERENCES play_groups(id),
    appid INTEGER NOT NULL,
    member_id TEXT NOT NULL REFERENCES members(id),
    vote INTEGER NOT NULL DEFAULT 0,
    star_rating INTEGER,
    PRIMARY KEY (group_id, appid, member_id)
);

CREATE TABLE steam_tag_cache (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    version_hash TEXT NOT NULL,
    tags_json TEXT NOT NULL,
    fetched_at TEXT NOT NULL
);

CREATE INDEX idx_members_group_id ON members(group_id);
CREATE INDEX idx_members_token ON members(token);
CREATE INDEX idx_wishlist_items_member_id ON wishlist_items(member_id);
CREATE INDEX idx_member_selections_member_id ON member_selections(member_id);
CREATE INDEX idx_group_game_votes_group_id ON group_game_votes(group_id);

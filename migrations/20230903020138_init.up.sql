CREATE TABLE words
(
    id       serial PRIMARY KEY,
    guild_id bigint,
    word     text NOT NULL
);

CREATE INDEX words_guild_id_index ON words (guild_id);

CREATE TABLE allowed_words
(
    guild_id bigint NOT NULL PRIMARY KEY,
    word_id  int    NOT NULL
);

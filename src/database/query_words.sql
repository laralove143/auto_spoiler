SELECT id as "id!", guild_id, word as "word!"
FROM words
WHERE guild_id = $1
UNION ALL
SELECT id, guild_id, word
FROM words
WHERE guild_id IS NULL
  AND id NOT IN (SELECT word_id FROM allowed_words WHERE guild_id = $1);
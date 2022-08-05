# spoiler bot

[<img src="img/add_to_server_badge.png" height="32"/>]
[<img src="img/join_support_server_badge.png" height="32"/>]

[<img src="img/add_to_server_badge.png" height="32"/>]: https://discord.com/api/oauth2/authorize?client_id=955408072199766086&permissions=536880128&scope=applications.commands%20bot
[<img src="img/join_support_server_badge.png" height="32"/>]: https://discord.gg/6vAzfFj8xG

a discord bot that automatically puts possibly triggering words in spoilers

## features

### auto-spoiler

puts swear words or possibly triggering words in spoilers  

- you can allow swear words or trigger words using `/allow`
- add your own custom words with `/custom_word`
- and even suggest words to be added to the list for everyone

**this is not auto-moderation**, it's simply for people that don't realize what
words might be triggering

### other commands

#### `/tw message tw_type`

send a possibly triggering message in spoilers, also telling why it might be triggering

#### `/tag message tag`

end your message with one of the listed tone tags, it also lists the tags in
case you forgot

## nerdy stuff

don't forget to change the guild id in [main.rs](src/main.rs) if you want to self-host

made by [laralove143] with [rust] using [twilight] and [sqlite], licensed MIT

[laralove143]: https://github.com/laralove143
[rust]: https://www.rust-lang.org
[twilight]: https://github.com/twilight-rs/twilight
[sqlite]: https://sqlite.org

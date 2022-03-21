# auto-spoiler-bot-discord

a discord bot that automatically puts possibly triggering words in spoilers

## features

### auto-spoiler

puts swear words or possibly triggering words in spoilers  
- you can allow swear words or trigger words using `/allow`
- add your own custom words with `/custom_word`
- and even suggest words to be added to the list for everyone

**this is not auto-moderation**, it's simply for people that don't realize what words might be triggering

### other commands

#### `/tw message tw_type`
send a possibly triggering message in spoilers, also telling why it might be triggering

#### `/tag message tag`
end your message with one of the listed tone tags, other people can even click on the command to see what the tag means

## nerdy stuff

don't forget to change the guild id in [main.rs](src/main.rs) if you want to self-host

made by [laralove143](https://github.com/laralove143) with [rust](https://www.rust-lang.org) using [twilight](https://github.com/twilight-rs/twilight) and [sqlite](https://sqlite.org), licensed MIT

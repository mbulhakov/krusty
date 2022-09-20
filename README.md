# krusty

### What
Bot that sends media by triggering on certain word patterns. Relies on postgres.

### How to run
Either build using cargo or use Docker.

### Configuration
The bot can be configured only with environment variables. For now there are we support the following variables:

| Name | Description | Values | Default value |
|------|-------------|--------|---------------|
| TELOXIDE_TOKEN | Telegram bot token | Any valid and registered Telegram bot token | ❌ |
| MEDIA_TIMEOUT_SEC | Timeout for media triggering in chat in secs | Any meaningful integer value from 0 | 30 |
| IGNORE_MESSAGE_OLDER_THEN_SEC | Ignore messages that were sent odler then duration specified in seconds |  Any meaningful integer value from 0 | 60 |
| MEDIA_SEND_CHANCE_IN_PERCENT | Chance of media triggering | From 0 to 100 | 50 |
| MAX_ACCEPTED_SCORE_SIMILARITY | Similarity score threshold. The lesser threshold the more precise and strict the behavior | From 0.0 to 1.0 | 0.26 |
| DATABASE_URL | Postgres URI | Any valid uri | ❌ |

If for any variable there is no default value, and you didn't provide any value - the bot won't start.

### How to use
TODO
Just fill the goddamn postgres DB and start the bot.

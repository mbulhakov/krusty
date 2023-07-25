# krusty

### What
Yet another TG bot. Does a few tricks:
- Checks hot words in the message => sends a media as a response (voice, video, picture) with a specific chance. The hot words can be either plain words or regexp patterns. Supports both by-word and whole-text matching. The plain text is checked for similarity using the unsophisticated inequality ```levenstein_distance(x,y) / max(x.len, y.len) <= max_accepted_score_similarity```.
- Checks forwarded posts from TG channels on duplication => sends a media as a response (voice, video, picture).
- Sends scheduled messages with media using cron jobs.

Works in supergroups.

### How to run
Either build using cargo or Docker (it is assumed that the builder runs Ubuntu 22.04). The native build is tested on Windows and MacOS and relies on libpq. The initial instance is running on https://fly.io/ as a Docker container.

### Configuration
The bot can be configured only with environment variables.

| Name | Description | Values | Default value |
|------|-------------|--------|---------------|
| TELOXIDE_TOKEN | Telegram bot token | Any valid and registered Telegram bot token | ❌ |
| MEDIA_TIMEOUT_SEC | Timeout for media in a chat in seconds | Any meaningful integer value from 0 | 30 |
| IGNORE_MESSAGE_OLDER_THAN_SEC | Ignore messages that were sent after a specified duration in seconds |  Any meaningful integer value from 0 | 60 |
| MEDIA_SEND_CHANCE_IN_PERCENT | A chance of media being sent upon successful hot word detection | From 0 to 100 | 50 |
| MAX_ACCEPTED_SCORE_SIMILARITY | Similarity score threshold. Lesser threshold implies more similarity is needed. Works for plain words. | From 0.0 to 1.0 | 0.26 |
| DATABASE_URL | Postgres URI | Any valid url | ❌ |
| LOG_LEVEL | log level| case insensitive: [off, error, warn, info, trace, debug] | info |

Variables that have no default value are mandatory to be set.

### How to use

The bot is not intended for general use since one heavily relies on data in Postgres, which should be ingested somehow. Some sort of panel might be added in the future to ease this burden.

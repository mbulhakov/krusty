# krusty

### What
Yet another TG bot. Does a few tricks:
- Checks hot words in the message => sends a media as a response (voice, video, picture) with a specific chance. The hot words are named tags in code and can be either plain words or regexp patterns.
- Checks forwarded posts from TG channels on duplication => sends a media as a response (voice, video, picture).
- Sends scheduled messages with media using cron jobs.

Works in supergroups.

### How to run
Either build using cargo or use Docker. The native build is tested on Windows and MacOS. The initial instance is running on https://fly.io/

### Configuration
The bot can be configured only with environment variables.

| Name | Description | Values | Default value |
|------|-------------|--------|---------------|
| TELOXIDE_TOKEN | Telegram bot token | Any valid and registered Telegram bot token | ❌ |
| MEDIA_TIMEOUT_SEC | Timeout for media in a chat in seconds | Any meaningful integer value from 0 | 30 |
| IGNORE_MESSAGE_OLDER_THAN_SEC | Ignore messages that were sent after a specified duration in seconds |  Any meaningful integer value from 0 | 60 |
| MEDIA_SEND_CHANCE_IN_PERCENT | A chance of media being sent upon successful triggering on word | From 0 to 100 | 50 |
| MAX_ACCEPTED_SCORE_SIMILARITY | Similarity score threshold. The lesser threshold the more precise and strict the behavior | From 0.0 to 1.0 | 0.26 |
| DATABASE_URL | Postgres URI | Any valid url | ❌ |

Variables that have no default value are mandatory to be set.

### How to use

The bot is not intended for general use due to being heavily relied on data in Postgres, which should be ingested somehow. Some sort of panel might be added in the future to ease this burden.

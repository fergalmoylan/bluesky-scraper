# bluesky-scraper

A Rust application which streams data from Bluesky's firehose api and produces in to a Kafka topic.
The data is formatted into the following structure before being sent to Kafka:
```json
{
  "created_at": "2024-11-25T19:11:20.256Z",
  "text": "This is a Bluesky post #Bluesky #Follow https://bsky.app/discover",
  "languages": ["English"],
  "hashtags": ["#Bluesky", "#Follow"],
  "urls": ["https://bsky.app/discover"],
  "hostnames": ["bsky.app"],
  "sentiment": {"score" :0.0, "comparative-score": 0.0, "sentiment": "neutral"}
}
```
The hashtags, urls, and hostnames are extracted from the text field.

Some of the code used is taken from https://github.com/sugyan/atrium/tree/main/examples/firehose
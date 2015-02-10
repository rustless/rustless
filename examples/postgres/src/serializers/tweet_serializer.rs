use jsonway::{self, ObjectSerializer};
use time;

use super::super::models::tweet;

pub struct TweetSerializer;
impl jsonway::ObjectSerializer<tweet::Tweet> for TweetSerializer {
    fn root(&self) -> Option<&str> { Some("tweet") }
    fn build(&self, tweet: &tweet::Tweet, json: &mut jsonway::ObjectBuilder) {
        json.set("id", tweet.get_id().to_string());
        json.set("author_name", tweet.get_author_name().to_string());
        json.set("content", tweet.get_content().to_string());
        json.set("created_at", time::at_utc(tweet.get_created_at().clone()).rfc3339().to_string());
        json.set("updated_at", time::at_utc(tweet.get_updated_at().clone()).rfc3339().to_string());
    }
}

pub struct TweetListSerializer<'a> {
    tweets: &'a Vec<tweet::Tweet>
}

impl<'a> TweetListSerializer<'a> {
    pub fn new(tweets: &'a Vec<tweet::Tweet>) -> TweetListSerializer<'a> {
        TweetListSerializer {
            tweets: tweets
        }
    }
}

impl<'a> jsonway::ArraySerializer for TweetListSerializer<'a> {
    fn root(&self) -> Option<&str> { Some("tweets") }
    fn build(&self, array: &mut jsonway::ArrayBuilder) {
        for tweet in self.tweets.iter() {
            array.push(TweetSerializer.serialize(tweet, false));
        }
    }
}


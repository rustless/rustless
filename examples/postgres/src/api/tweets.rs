use rustless;
use rustless::prelude::*;
use jsonway::{ObjectSerializer, ArraySerializer};
use uuid;
use url;

use super::super::db::DatabaseExt;
use super::super::models::tweet;
use super::super::serializers::tweet_serializer;

pub fn tweets(path: &str) -> rustless::Namespace {
    rustless::Namespace::build(path, |tweets| {
        
        tweets.get("latest", |endpoint| {
            endpoint.desc("Get latest tweets");
            endpoint.handle(|client, _params| {
                // Note that .db() is an extension methods that we created with DatabaseExt
                let cn = client.app.db();
                let tweets = tweet::Tweet::latest(&*cn);
                client.json(&tweet_serializer::TweetListSerializer::new(&tweets).serialize(true))
            })
        });

        tweets.post("", |endpoint| {
            endpoint.desc("Create new tweet");
            endpoint.params(|params| {
                params.req("tweet", |tweet| {
                    tweet.desc("Tweet model in JSON format");
                    tweet.schema(|tweet| {
                        tweet.id("http://tweet.example.com/tweet-short");
                        tweet.object();
                        tweet.properties(|props| {
                            props.insert("author_name", |author_name| {
                                author_name.string();
                                author_name.max_length(30);
                            });
                            props.insert("content", |content| {
                                content.string();
                                content.max_length(140);
                            })
                        });
                        tweet.required(vec![
                            "author_name".to_string(), 
                            "content".to_string()
                        ]);
                        tweet.additional_properties(false);
                    })
                })
            });
            endpoint.handle(|client, params| {
                // Note that .db() is an extension methods that we created with DatabaseExt
                let cn = client.app.db();
                let tweet = params.find("tweet").unwrap();

                let tweet = tweet::Tweet::new(
                    tweet.find("author_name").unwrap().as_string().unwrap().to_string(),
                    tweet.find("content").unwrap().as_string().unwrap().to_string()
                );

                tweet.create(&*cn);
                client.json(&tweet_serializer::TweetSerializer.serialize(&tweet, true))
            })
        });

        tweets.namespace(":tweet_id", |single| {
            single.params(|params| {
                params.req("tweet_id", |tweet_id| {
                    tweet_id.desc("Tweet ID in UUID format");
                    tweet_id.schema(|schema| {
                        schema.format("uuid");
                    })
                })
            });

            single.get("", |endpoint| {
                endpoint.desc("Get tweet by ID");
                endpoint.handle(|mut client, params| {
                    // Note that .db() is an extension methods that we created with DatabaseExt
                    let cn = client.app.db();
                    let id: uuid::Uuid = params.find("tweet_id").unwrap().as_string().unwrap().parse().unwrap();
                    let tweet = tweet::Tweet::find(id, &*cn);

                    if tweet.is_some() {
                        client.json(&tweet_serializer::TweetSerializer.serialize(&tweet.unwrap(), true))
                    } else {
                        client.not_found();
                        client.empty()
                    }
                })
            });

            single.put("", |endpoint| {
                endpoint.desc("Update tweet");
                endpoint.params(|params| {
                    params.req("tweet", |tweet| {
                        tweet.desc("Tweet model in JSON format");
                        tweet.schema_id(url::Url::parse("http://tweet.example.com/tweet-short").unwrap())
                    })
                });
                endpoint.handle(|mut client, params| {
                    // Note that .db() is an extension methods that we created with DatabaseExt
                    let cn = client.app.db();
                    let tweet_params = params.find("tweet").unwrap();
                    let id: uuid::Uuid = params.find("tweet_id").unwrap().as_string().unwrap().parse().unwrap();

                    let tweet = tweet::Tweet::find(id, &*cn);

                    if tweet.is_some() {
                        let mut tweet = tweet.unwrap();
                        let new_author_name = tweet_params.find("author_name").unwrap().as_string().unwrap().to_string();
                        let new_content = tweet_params.find("content").unwrap().as_string().unwrap().to_string();
                        tweet.set_author_name(new_author_name);
                        tweet.set_content(new_content);

                        tweet.update(&*cn);
                        client.json(&tweet_serializer::TweetSerializer.serialize(&tweet, true))
                    } else {
                        client.not_found();
                        client.empty()
                    }
                })
            });

            single.delete("", |endpoint| {
                endpoint.desc("Delete tweet");
                endpoint.handle(|mut client, params| {
                    // Note that .db() is an extension methods that we created with DatabaseExt
                    let cn = client.app.db();
                    let id: uuid::Uuid = params.find("tweet_id").unwrap().as_string().unwrap().parse().unwrap();

                    let tweet = tweet::Tweet::find(id, &*cn);

                    if tweet.is_some() {
                        let tweet = tweet.unwrap();
                        tweet.delete(&*cn);
                        client.json(&tweet_serializer::TweetSerializer.serialize(&tweet, true))
                    } else {
                        client.not_found();
                        client.empty()
                    }
                })
            });

        })
        
    })
}
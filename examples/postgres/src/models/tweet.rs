use uuid;
use time;

use super::super::db;

pub struct Tweet {
    id: uuid::Uuid,
    author_name: String,
    content: String,
    created_at: time::Timespec,
    updated_at: time::Timespec
}

impl Tweet {

    pub fn get_id(&self) -> &uuid::Uuid { &self.id }
    pub fn get_author_name(&self) -> &str { self.author_name.as_slice() }
    pub fn get_content(&self) -> &str { self.content.as_slice() }
    pub fn get_created_at(&self) -> &time::Timespec { &self.created_at }
    pub fn get_updated_at(&self) -> &time::Timespec { &self.updated_at }

    pub fn set_author_name(&mut self, author_name: String) { self.author_name = author_name; }
    pub fn set_content(&mut self, content: String) { self.content = content; }

    pub fn new(author_name: String, content: String) -> Tweet {
        Tweet {
            id: uuid::Uuid::new_v4(),
            author_name: author_name,
            content: content,
            created_at: time::get_time(),
            updated_at: time::get_time()
        }
    }

    pub fn latest(cn: &db::Connection) -> Vec<Tweet> {
        let st = cn.prepare("SELECT * FROM tweets ORDER BY updated_at DESC LIMIT 10").unwrap();
        let mut tweets = vec![];
        for tweet_row in st.query(&[]).unwrap() {
            let tweet = Tweet {
                id: tweet_row.get("id"),
                author_name: tweet_row.get("author_name"),
                content: tweet_row.get("content"),
                created_at: tweet_row.get("created_at"),
                updated_at: tweet_row.get("updated_at"),
            };

            tweets.push(tweet);
        }

        tweets
    }

    pub fn find(id: uuid::Uuid, cn: &db::Connection) -> Option<Tweet> {
        let st = cn.prepare("SELECT * FROM tweets WHERE id = $1 LIMIT 1").unwrap();
        for tweet_row in st.query(&[&id]).unwrap() {
            return Some(Tweet {
                id: tweet_row.get("id"),
                author_name: tweet_row.get("author_name"),
                content: tweet_row.get("content"),
                created_at: tweet_row.get("created_at"),
                updated_at: tweet_row.get("updated_at"),
            })
        }

        None
    }

    pub fn create(&self, cn: &db::Connection) {
        cn.execute("INSERT INTO tweets VALUES ($1, $2, $3, $4, $5);", &[
            &self.id,
            &self.author_name,
            &self.content,
            &self.created_at,
            &self.updated_at
        ]).unwrap();
    }

    pub fn update(&mut self, cn: &db::Connection) {
        cn.execute("UPDATE tweets SET (author_name, content, created_at, updated_at) = ($2, $3, $4, $5) WHERE tweets.id = $1;", &[
            &self.id,
            &self.author_name,
            &self.content,
            &self.created_at,
            &self.updated_at
        ]).unwrap();
    }

    pub fn delete(&self, cn: &db::Connection) {
        cn.execute("DELETE FROM tweets WHERE tweets.id = $1;", &[
            &self.id,
        ]).unwrap();
    }
}
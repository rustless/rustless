pub use postgres::Connection;
pub use deuterium_orm::migration::RawMigration;

#[derive(Debug)]
pub struct CreateTweets;
impl RawMigration<Connection> for CreateTweets {
    fn up(&self, cn: &Connection) {
        let t = cn.transaction().unwrap();
        t.execute("CREATE TABLE tweets (
            id               UUID PRIMARY KEY,
            author_name       VARCHAR(30),
            content          VARCHAR(140),
            created_at       timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at       timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL
        )", &[]).unwrap();

        t.set_commit();
        t.finish().unwrap();
    }

    fn down(&self, cn: &Connection) {
        cn.execute("DROP TABLE tweets;", &[]).unwrap();
    }
}
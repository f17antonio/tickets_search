use std::ops::Deref;
use rocket::http;
use rocket::request;
use rocket::Outcome;
use rocket::State;
use r2d2;
use r2d2_redis::RedisConnectionManager;

const REDIS_ADDRESS: &'static str = "redis://localhost:6379";

pub fn pool() -> Pool {
    let manager = RedisConnectionManager::new(REDIS_ADDRESS).expect("connection manager");
    r2d2::Pool::new(manager).expect("db pool")
}

pub struct RedisConnection(pub r2d2::PooledConnection<RedisConnectionManager>);

type Pool = r2d2::Pool<RedisConnectionManager>;

impl<'a, 'r> request::FromRequest<'a, 'r> for RedisConnection {
    type Error = ();

    fn from_request(request: &'a request::Request<'r>) -> request::Outcome<RedisConnection, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(RedisConnection(conn)),
            Err(_) => Outcome::Failure((http::Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for RedisConnection {
    type Target = r2d2::PooledConnection<RedisConnectionManager>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
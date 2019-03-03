use rocket_contrib::json::Json;
use serde_json::{json, Value};
use crate::redis_con::RedisConnection;
use crate::search::{Search};
use crate::tickets::{Tickets, Ticket};

#[post("/", data = "<tickets>")]
pub fn create(tickets: Json<Tickets>, connection: RedisConnection,) -> Json<Value> {
    let tickets = Tickets { ..tickets.into_inner() };
    for ticket in tickets.tickets {
        Ticket::insert(&connection, ticket);
    }
    Json(json!({"status": "success"}))
}

#[post("/", data = "<search>")]
pub fn search(search: Json<Search>, connection: RedisConnection) -> Json<Value> {
    let search2 = Search { ..search.into_inner() };
    let search_result = Search::search(&connection, search2.departure_code, search2.arrival_code, search2.departure_date, search2.limit);
    Json(json!(search_result))
}
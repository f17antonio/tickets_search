use std::string::ToString;
use serde::{Deserialize, Serialize};
use redis::{Connection, Commands};

#[derive(Serialize, Deserialize)]
pub struct Tickets {
    pub tickets: Vec<Ticket>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Ticket {
    pub id: String,
    pub departure_code: String,
    pub arrival_code: String,
    pub departure_time: i32,
    pub arrival_time: i32,
    pub price: i32
}

impl Ticket {
    pub fn insert(con: &Connection, ticket: Ticket) {
        let sorted_set_value = format!("{}_{}", ticket.arrival_code, ticket.arrival_time);
        let _: () = con.zadd(format!("times_{}", ticket.departure_code), sorted_set_value, ticket.departure_time).unwrap();
        let _: () = con.set(format!("{}_{}", ticket.arrival_code, ticket.departure_time), &ticket.id).unwrap();
        let _: () = con.hset_multiple(format!("ticket_{}", ticket.id), &[
            ("id", ticket.id),
            ("departure_code", ticket.departure_code),
            ("arrival_code", ticket.arrival_code),
            ("departure_time", ticket.departure_time.to_string()),
            ("arrival_time", ticket.arrival_time.to_string()),
            ("price", ticket.price.to_string())
        ]).unwrap();
    }
}
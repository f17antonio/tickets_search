#![feature(proc_macro_hygiene, decl_macro, plugin)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate log;
extern crate r2d2;
extern crate r2d2_redis;
extern crate simple_logger;
extern crate redis;
extern crate chrono;

mod search;
mod tickets;
mod redis_con;
mod handlers;

use log::Level;

fn main() {
    simple_logger::init_with_level(Level::Info).unwrap();
    rocket::ignite()
        .manage(redis_con::pool())
        .mount("/batch_insert", routes![handlers::create])
        .mount("/search", routes![handlers::search])
        .launch();
}
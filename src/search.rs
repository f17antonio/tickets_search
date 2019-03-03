use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use chrono::TimeZone;
use redis::{Connection, Commands};
use std::collections::HashMap;
use crate::tickets::Ticket;

#[derive(Serialize, Deserialize)]
pub struct Search {
    pub departure_code: String,
    pub arrival_code: String,
    pub departure_date: String,
    pub limit: i32
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub solutions: Vec<SearchResultSolution>
}

#[derive(Serialize, Deserialize)]
pub struct SearchResultSolution {
    pub ticket_ids: Vec<String>,
    pub price: i32
}

struct ArrivalData {
    pub arrival_code: String,
    pub arrival_time: i32
}

impl Search {
    pub fn search(con: &Connection, departure_code: String, arrival_code: String, departure_date: String, limit: i32) -> SearchResult {
        let departure_time = get_timestamp_from_str(departure_date);
        let all_tickets: Vec<Vec<Ticket>> = get_all_tickets(con, departure_code, arrival_code, departure_time);
        let tickets = prepare_tickets(all_tickets, limit);

        return SearchResult{ solutions : tickets};
    }
}

fn get_all_tickets(con: &Connection, departure_code: String, arrival_code: String, departure_time: i64) -> Vec<Vec<Ticket>>{
    let mut tickets : Vec<Vec<Ticket>> = Vec::new();
    let direct_tickets : HashMap<String, i32> = get_tickets_by_time(con, &departure_code,departure_time, departure_time + 86400);
    if direct_tickets.len() > 0 {
        for (dirrect_arrival_data_string, departure_time) in direct_tickets {
            let direct_arrival_data = get_arrival_data(dirrect_arrival_data_string);

            if direct_arrival_data.arrival_code == arrival_code {
                tickets.push(vec![get_ticket_by_time(con, &direct_arrival_data.arrival_code, &departure_time)]);
            } else {
                let transfer_tickets =
                    get_transfer_tickets(con, &arrival_code, direct_arrival_data.arrival_code, direct_arrival_data.arrival_time, departure_time);
                tickets.extend(transfer_tickets);
            }
        }
    }
    return tickets;
}

fn get_transfer_tickets(con: &Connection, arrival_code: &String, dirrect_arrival_code: String, arrival_time: i32, departure_time: i32) -> Vec<Vec<Ticket>> {
    let mut transfer_tickets: Vec<Vec<Ticket>> = Vec::new();
    let start_time = arrival_time as i64 + 10800;
    let end_time = arrival_time as i64 + 82800;
    let tickets_with_transfer = get_tickets_by_time(con, &dirrect_arrival_code, start_time, end_time);
    if tickets_with_transfer.len() > 0 {
        for (transfer_arrival_data_string, transfer_departure_time) in tickets_with_transfer {
            let transfer_arrival_data = get_arrival_data(transfer_arrival_data_string);
            if &transfer_arrival_data.arrival_code == arrival_code {
                transfer_tickets.push(vec![
                    get_ticket_by_time(con, &dirrect_arrival_code, &departure_time),
                    get_ticket_by_time(con, &transfer_arrival_data.arrival_code, &transfer_departure_time)
                ]);
            }
        }
    }
    return transfer_tickets;
}

fn get_tickets_by_time(con: &Connection, departure_code: &String, start_time: i64, end_time: i64) -> HashMap<String, i32> {
    let key = format!("times_{}", departure_code);
    let result: HashMap<String, i32> = con.zrangebyscore_withscores(key, start_time, end_time).unwrap();
    return result;
}

fn get_ticket_by_time(con: &Connection, arrival_code: &String, departure_time: &i32) -> Ticket {
    let key = format!("{}_{}", arrival_code, departure_time);
    let id: String = con.get(key).unwrap();
    let ticket: Ticket = get_ticket_by_id(con, id);
    return ticket;
}

fn get_ticket_by_id(con: &Connection, id: String) -> Ticket {
    let key = format!("ticket_{}", id);
    let ticket_map: HashMap<String, String> = con.hgetall(key).unwrap();
    let ticket: Ticket = Ticket {
        id: ticket_map.get("id").expect("id is empty").clone(),
        departure_code: ticket_map.get("departure_code").expect("departure_code is empty").clone(),
        arrival_code: ticket_map.get("arrival_code").expect("arrival_code is empty").clone(),
        departure_time: ticket_map.get("departure_time").expect("departure_time is empty").parse::<i32>().unwrap(),
        arrival_time: ticket_map.get("arrival_time").expect("arrival_time is empty").parse::<i32>().unwrap(),
        price: ticket_map.get("price").expect("price is empty").parse::<i32>().unwrap(),
    };
    return ticket;
}

fn prepare_tickets(all_tickets: Vec<Vec<Ticket>>, limit: i32) -> Vec<SearchResultSolution>{
    let mut prepared_tickets = Vec::new();
    for related_tickets in &all_tickets {
        let mut price = 0;
        let mut ids = Vec::new();
        for ticket in related_tickets {
            price = price + ticket.price;
            ids.push(ticket.id.clone());
        }
        prepared_tickets.push(SearchResultSolution{
            ticket_ids: ids,
            price: price
        });
    }
    prepared_tickets.sort_by(|a, b| a.price.cmp(&b.price));
    prepared_tickets.truncate(limit as usize);

    return prepared_tickets;
}

fn get_arrival_data(arrival_data_string: String) -> ArrivalData {
    let arrival_data: Vec<&str> = arrival_data_string.split('_').collect();
    let arrival_code = arrival_data[0].to_string();
    let arrival_time = arrival_data[1].parse::<i32>().unwrap();
    return ArrivalData{arrival_code, arrival_time};
}

fn get_timestamp_from_str(date_string : String) -> i64 {
    let datetime =
        match Utc.datetime_from_str(&format!("{} 00:00:00", date_string), "%Y-%m-%d %H:%M:%S") {
            Ok(value) => value,
            Err(e) =>
                {
                    warn!("ERROR: {}", e);
                    panic!("Incorrect date format")
                }
        };

    return datetime.timestamp();
}

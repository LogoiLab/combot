use crate::types::*;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, Ipv4Addr};
use std::str::SplitWhitespace;

use chrono::{DateTime, FixedOffset, TimeZone};

pub fn parse(input: &str, uri_path: &str, ua_path: &str) -> Vec<BotData> {
    let file = match File::open(input) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to open input file: {}", e);
            std::process::exit(1);
        }
    };
    let reader = BufReader::new(file);

    let mut founds: Vec<BotData> = vec!();
    for (_, line) in reader.lines().enumerate() {
        let mut found: BotData = BotData{
            name: String::new(),
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            date: FixedOffset::east(0).ymd(1970, 1, 1).and_hms(0, 0, 0),
            uri: String::new(),
            user_agent: String::new(),
            triggered_on: Trigger::Unassigned
        };

        let line = line.unwrap();
        //println!("{}", line);
        let mut split_line: SplitWhitespace = line.as_str().split_whitespace();

        let ip_addr = split_line.next().unwrap();
        if ip_addr.contains(":") {
            found.ip = IpAddr::V6(ip_addr.parse().unwrap());
        } else { 
            found.ip = IpAddr::V4(ip_addr.parse().unwrap());
        }

        let mut date_split = split_line.skip(2);
        let log_date = format!("{} {}", date_split.next().unwrap(), date_split.next().unwrap());
        found.date = match DateTime::parse_from_str(log_date.as_str(), "[%d/%b/%Y:%T %#z]") {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Failed to parse log date-time: {}, {}", log_date, e);
                std::process::exit(1);
            }
        };

        let mut uri_split = date_split.skip(1);

        let uri = uri_split.next().unwrap().to_string();
        found.uri = uri.clone();
        match crate::regexes::bot_uris(uri, uri_path) {
            Some(s) => {
                found.triggered_on = Trigger::UriPath;
                found.name = s;
            },
            None => ()
        }

        let ua_split = uri_split.skip(4);
        for (_, ua_part) in ua_split.enumerate() {
            found.user_agent.push_str(format!("{} ", ua_part).replace("\"", "").trim());
        }

        match crate::regexes::bot_uas(&found.user_agent, ua_path) {
            Some(s) => {
                found.triggered_on = Trigger::UserAgent;
                found.name = s;
            },
            None => ()
        }

        match found.triggered_on {
            Trigger::Unassigned => (),
            _ => founds.push(found)
        }
    }

    return founds;
}

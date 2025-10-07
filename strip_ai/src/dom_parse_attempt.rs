use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tl::{HTMLTag, NodeHandle, Parser, VDom};

const EVENT_STRING: &str = "event           : [{";

fn main() {
    let page = reqwest::blocking::get("https://2025.allthingsopen.org/schedule")
        .unwrap()
        .text()
        .unwrap();

    let mut dom = tl::parse(&page, tl::ParserOptions::default()).unwrap();
    //let event_node = get_event_node(&dom);
    //strip_ai_sessions(event_node, &mut dom);

    let html_tag = dom
        .children()
        .iter()
        .flat_map(|child| child.get(dom.parser()).and_then(|child| child.as_tag()))
        .find(|tag| tag.name().as_bytes() == "html".as_bytes())
        .unwrap();

    //println!("{}", html_tag.outer_html(dom.parser()));
    println!("{page}");
}

pub fn get_event_node(dom: &VDom<'_>) -> NodeHandle {
    let parser = dom.parser();

    let html_tag = dom
        .children()
        .iter()
        .flat_map(|child| child.get(parser).and_then(|child| child.as_tag()))
        .find(|tag| tag.name().as_bytes() == "html".as_bytes())
        .unwrap();
    let body_tag = html_tag
        .children()
        .top()
        .iter()
        .flat_map(|child| child.get(parser).and_then(|child| child.as_tag()))
        .find(|tag| tag.name().as_bytes() == "body".as_bytes())
        .unwrap();

    let javascript = body_tag
        .children()
        .all(parser)
        .iter()
        .find(|child| child.inner_text(parser).contains(EVENT_STRING))
        .unwrap();

    //let mut_parser = dom.parser_mut();
    //println!("{javascript:#?}");
    let event_js = javascript
        .children()
        .unwrap()
        .all(parser)
        .iter()
        .find(|child| child.inner_text(parser).contains(EVENT_STRING))
        .unwrap();

    event_js
        .find_node(parser, &mut |child| {
            child.inner_text(parser).contains(EVENT_STRING)
        })
        .unwrap()
}

pub fn strip_ai_sessions(event_node: NodeHandle, dom: &mut VDom<'_>) {
    let event_bytes = event_node
        .get_mut(dom.parser_mut())
        .unwrap()
        .as_raw_mut()
        .unwrap();

    let event_str = event_bytes.as_utf8_str();

    let (split_idx, event_line) = event_str
        .lines()
        .enumerate()
        .find(|(_idx, line)| line.contains(EVENT_STRING))
        .unwrap();
    let event_line = &event_line[event_line.find("[").unwrap()..event_line.rfind(",").unwrap()];
    let mut days: Vec<Day> = serde_json::from_str(event_line).unwrap();

    days.iter_mut()
        .flat_map(|day| day.schedules.iter_mut())
        .flat_map(|schedules| schedules.timeblocks.iter_mut())
        .for_each(|timeblock| timeblock.sessions.retain(|session| true));

    /*
    let mut new_event_bytes = Vec::with_capacity(event_bytes.as_bytes().len());
    let mut event_str_lines = event_str.lines();

    // Add lines before the replaced events
    new_event_bytes.extend(
        event_str_lines
            .by_ref()
            .take(split_idx)
            .flat_map(str::as_bytes),
    );

    // Reinsert with the same formatting
    new_event_bytes.extend_from_slice("event : ".as_bytes());
    new_event_bytes.append(&mut serde_json::to_vec(&days).unwrap());
    new_event_bytes.push(b',');

    // Discard replaced line and add end
    let _ = event_str_lines.next();
    new_event_bytes.extend(event_str_lines.flat_map(str::as_bytes));

    let _old_value = event_bytes.set(new_event_bytes).unwrap();
    */
}

#[derive(Debug, Serialize, Deserialize)]
struct Day<'a> {
    #[serde(borrow)]
    schedules: Vec<Schedule<'a>>,
    #[serde(borrow)]
    #[serde(flatten)]
    remainder: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Schedule<'a> {
    #[serde(borrow)]
    timeblocks: Vec<Timeblock<'a>>,
    #[serde(borrow)]
    #[serde(flatten)]
    remainder: HashMap<Cow<'a, str>, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Timeblock<'a> {
    #[serde(borrow)]
    sessions: Vec<Session<'a>>,
    #[serde(borrow)]
    #[serde(flatten)]
    remainder: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Session<'a> {
    #[serde(borrow)]
    title: Cow<'a, str>,
    #[serde(borrow)]
    #[serde(flatten)]
    remainder: HashMap<Cow<'a, str>, Value>,
}

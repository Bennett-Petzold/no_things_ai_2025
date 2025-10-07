/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Finds the line with a JSON event object.
///
/// Update this if the webpage formatting changes.
const EVENT_STRING: &str = "event           : [{";

/// Any talk with this phrase will get removed.
///
/// Update this for any talks that escaped the purge.
const BANNED_PHRASES: &[&str] = &[
    "AI",
    "LLM",
    "ML",
    "Agentic",
    "The Owned Algorithm: Open Sourcing Myself",
    "Vibe Coding",
    "Building Agents",
    "Talk Less, Merge More: Building a GitLab Sidecar that Reasons and Responds",
    "Language Models",
    "Knowledge Graph",
];

fn main() {
    let page = reqwest::blocking::get("https://2025.allthingsopen.org/schedule")
        .unwrap()
        .text()
        .unwrap();

    let event_start = page.find(EVENT_STRING).unwrap();
    let event_end = event_start + page[event_start..].find('\n').unwrap();
    let event_replaced = strip_ai_sessions(&page[event_start..event_end]);
    println!(
        "{}{}{}",
        &page[..event_start],
        event_replaced,
        &page[event_end..],
    );
}

/// Removes all AI sessions from the event object.
pub fn strip_ai_sessions(event_line: &str) -> String {
    let event_line = &event_line[event_line.find("[").unwrap()..event_line.rfind(",").unwrap()];
    // Day is a nest of minimal Serde definitions for our purposes.
    let mut days: Vec<Day> = serde_json::from_str(event_line).unwrap();

    days.iter_mut()
        .flat_map(|day| day.schedules.iter_mut())
        .flat_map(|schedules| schedules.timeblocks.iter_mut())
        .for_each(|timeblock| {
            // Keynote sessions need to stay in for spacing, so they get
            // special handling.

            timeblock.sessions.retain(|session| {
                session.cat.contains("Keynote")
                    || BANNED_PHRASES
                        .iter()
                        .all(|word| !session.title.contains(word))
            });
            timeblock.sessions.iter_mut().for_each(|session| {
                if BANNED_PHRASES
                    .iter()
                    .any(|word| session.title.contains(word))
                {
                    session.title = "Hallway Track".into();
                    session.speakers = vec!["Who Cares?".into()];
                }
            });
        });

    // Recreate with the same formatting
    "event : ".to_string() + &serde_json::to_string(&days).unwrap() + ","
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
    cat: Cow<'a, str>,
    #[serde(borrow)]
    speakers: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    #[serde(flatten)]
    remainder: HashMap<Cow<'a, str>, Value>,
}

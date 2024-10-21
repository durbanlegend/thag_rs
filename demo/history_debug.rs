/*[toml]
[dependencies]
regex = "1.10.6"
serde = { version = "1.0.208", features = ["derive"] }
serde_json = "1.0.132"
*/

/// Debug the history handling logic of the `stdin` module and display the effects.
/// Using this abstraction because displays don't work nicely in a TUI editor.
//# Purpose: Debug and demo history ordering.
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::io::{self, BufRead, IsTerminal};
use std::{collections::VecDeque, fs, path::PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
struct History {
    entries: VecDeque<usize>,
    current_index: Option<usize>,
}

impl History {
    fn new() -> Self {
        History {
            entries: VecDeque::with_capacity(20),
            current_index: None,
        }
    }

    fn load_from_file(path: &PathBuf) -> Self {
        if let Ok(data) = fs::read_to_string(path) {
            println!("data={data}");
            serde_json::from_str(&data).unwrap_or_else(|_| History::new())
        } else {
            History::default()
        }
    }

    fn save_to_file(&self, path: &PathBuf) {
        if let Ok(data) = serde_json::to_string(self) {
            let _ = fs::write(path, data);
        }
    }

    fn add_entry(&mut self, entry: usize) {
        // Remove prior duplicates
        self.entries.retain(|f| f != &entry);
        self.entries.push_front(entry);
    }

    fn get_current(&mut self) -> Option<usize> {
        if self.entries.is_empty() {
            return None;
        }

        self.current_index = match self.current_index {
            Some(index) => {
                println!("index={index}, index + 1 = {}", index + 1);
                Some(index + 1)
            },
            _ => Some(0),
        };

        self.entries.get(0).copied()
    }

    fn get_previous(&mut self) -> Option<usize> {
        if self.entries.is_empty() {
            return None;
        }

        self.current_index = match self.current_index {
            Some(index) => {
                println!("index={index}, index + 1 = {}", index + 1);
                Some(index + 1)
            },
            _ => Some(0),
        };

        self.current_index.and_then(|index| self.entries.get(index)).copied()
    }

    fn get_next(&mut self) -> Option<usize> {
        if self.entries.is_empty() {
            return None;
        }

        self.current_index = match self.current_index {
            Some(index) if index > 0 => Some(index - 1),
            Some(index) if index == 0 => Some(index + self.entries.len() - 1),
            _ => Some(self.entries.len() - 1),
        };

        self.current_index.and_then(|index| self.entries.get(index)).copied()
    }
}

fn f1(val: &mut usize, history: &mut History, saved_to_history: &mut bool) {
    println!("Before f1: history={history:?}, curr={val}");
    let x = val.clone();
    let mut found = false;
    // 6 5,4,3,2,1 -> >5,4,3,2,1
    if *saved_to_history {
        println!("Already saved to history: calling history.get_previous()");
        if let Some(entry) = history.get_previous() {
            found = true;
            *val = entry;
        }
    } else {
        println!("Not already saved to history: calling history.get_current()");
        if let Some(entry) = history.get_current() {
            found = true;
            *val = entry;
        }
    }
    if found && !*saved_to_history {
        // 5 >5,4,3,2,1 -> 5 6,>5,4,3,2,1
        history
            .add_entry(x);
        // history.entries.rotate_left(1);
        *saved_to_history = true;
    }
    println!("After f1: history={history:?}, curr={val}");
}

fn f2(val: &mut usize, history: &mut History, saved_to_history: &mut bool) {
    println!("Before f2: history={history:?}, curr={val}");
    if let Some(entry) = history.get_next() {
        *val = entry;
    }
    println!("After f2: history={history:?}, curr={val}");
}

let mut curr: usize;

let history_path = PathBuf::from("hist_debug.json");

let mut history = History::load_from_file(&history_path);

let mut saved_to_history = false;

let max: usize = match history.entries.iter().max() {
    Some(max) => *max,
    None => 0,
};

curr = max + 1;

println!("Before f1: curr={curr}");

f1(&mut curr, &mut history, &mut saved_to_history);
println!("After f1: curr={curr}");

f2(&mut curr, &mut history, &mut saved_to_history);
println!("After f2: curr={curr}");

f2(&mut curr, &mut history, &mut saved_to_history);
println!("After f2: curr={curr}");

let mut save_and_exit = |n: usize| {
    println!("Before save_and_exit: history={history:?}");
    history.add_entry(n);
    // history.entries.rotate_left(1);
    history.current_index = Some(0);
    println!("save_and_exit: history={history:?}");
    history.save_to_file(&history_path);
};

save_and_exit(curr);

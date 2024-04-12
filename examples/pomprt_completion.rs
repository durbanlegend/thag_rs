//! [dependencies]
//! pomprt = "0.5.3"

// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

struct MiniShell;

impl pomprt::Editor for MiniShell {
    fn complete(&self, buffer: &str, cursor: usize) -> Option<pomprt::Completion> {
        let mut start = buffer[..cursor].rfind(' ').map_or(0, |c| c + 1);
        let end = buffer[cursor..]
            .find(' ')
            .map_or(buffer.len(), |c| cursor + c);
        let word = &buffer[start..end];

        let results = match buffer[..start].find(|c| c != ' ') {
            Some(_) => match word.rsplit_once('/') {
                Some((dir, name)) => {
                    start += dir.len() + 1;
                    complete_file(if dir.is_empty() { "/" } else { dir }, name)
                }
                None => complete_file(".", word),
            },
            None => complete_file("/usr/bin", word),
        };

        Some(pomprt::Completion(start..end, results))
    }
}

fn complete_file(dir: &str, prefix: &str) -> Vec<String> {
    let mut entries = std::fs::read_dir(dir).map_or(Vec::new(), |dir| {
        dir.filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().into_string().ok()?;
            let suffix = if entry.path().is_dir() { "/" } else { " " };
            name.starts_with(prefix).then(|| name + suffix)
        })
        .collect()
    });
    entries.sort_unstable();
    entries
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sh = pomprt::with(MiniShell, "% ");

    loop {
        let ok = match sh.read() {
            Ok(input) => {
                let mut args = input.split_ascii_whitespace();
                if let Some(cmd) = args.next() {
                    Command::new(cmd).args(args).spawn()?.wait()?.success()
                } else {
                    true
                }
            }
            Err(pomprt::Interrupt) => false,
            Err(pomprt::Eof) => return Ok(()),
            Err(e) => Err(e)?,
        };

        sh.set_prompt(if ok { "% " } else { "! " });
    }
}

#! /usr/bin/env rs_script
/*[toml]
[dependencies]
[features]
duration_constructors = []
*/
#![feature(duration_constructors)]
use std::time::Duration;
Duration::from_days(10)

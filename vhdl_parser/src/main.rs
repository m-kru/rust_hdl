// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2018, Olof Kraigher olof.kraigher@gmail.com

// Allowing this, since there is an open issue with this lint
// Track here: https://github.com/rust-lang/rust-clippy/issues/1981
// Track here: https://github.com/rust-lang/rust-clippy/issues/1981
#![allow(clippy::ptr_arg)]

#[macro_use]
extern crate clap;

use std::path::Path;
use vhdl_parser::{Config, Diagnostic, Project};

fn main() {
    use clap::{App, Arg};

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("show")
                .long("show")
                .help("Show information about design units"),
        )
        .arg(
            Arg::with_name("num-threads")
                .short("-p")
                .long("--num-threads")
                .default_value("4")
                .help("The number of threads to use"),
        )
        .arg(
            Arg::with_name("config")
                .help("Config file in TOML format containing libraries and settings")
                .short("-c")
                .long("--config")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let num_threads = value_t_or_exit!(matches.value_of("num-threads"), usize);

    if let Some(file_name) = matches.value_of("config") {
        let config =
            Config::read_file_path(Path::new(file_name)).expect("Failed to read config file");

        let mut messages = Vec::new();
        let mut project = Project::from_config(&config, num_threads, &mut messages);
        if !messages.is_empty() {
            for message in messages {
                println!("{}", message);
            }
        }
        show_diagnostics(&project.analyse());
    }
}

fn show_diagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        println!("{}", diagnostic.show());
    }
}

/*
 * Copyright 2019 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//#![warn(missing_docs)]

// TODO: add some docs

use clap::{App, AppSettings, Arg, SubCommand};
use exitfailure::ExitFailure;
use failure::err_msg;
use parity_wasm;
use parity_wasm::elements::{ModuleNameSubsection, NameSection, Serialize};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

const MODULE_PATH: &str = "module_path";
const NEW_MODULE_NAME: &str = "new_module_name";

fn show_module_name_subcommand<'a, 'b>() -> App<'a, 'b> {
    let arg = &[Arg::with_name(MODULE_PATH)
        .required(true)
        .takes_value(true)
        .help("path to the wasm file")];

    SubCommand::with_name("show")
        .about("Show Wasm file name")
        .args(arg)
}

fn set_module_name_subcommand<'a, 'b>() -> App<'a, 'b> {
    let args = &[
        Arg::with_name(MODULE_PATH)
            .required(true)
            .takes_value(true)
            .help("path to the Wasm file"),
        Arg::with_name(NEW_MODULE_NAME)
            .required(true)
            .takes_value(true)
            .help("a new module name"),
    ];

    SubCommand::with_name("set")
        .about("Set Wasm file name")
        .args(args)
}

fn main() -> Result<(), ExitFailure> {
    let app = App::new("Fluence wasm-module-name")
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(show_module_name_subcommand())
        .subcommand(set_module_name_subcommand());

    match app.get_matches().subcommand() {
        ("show", Some(arg)) => {
            let module_path = arg.value_of(MODULE_PATH).unwrap();

            let module =
                parity_wasm::deserialize_file(module_path).expect("Error while deserializing file");
            let module = module.parse_names().expect("Error while parsing names");
            let name_section = module.names_section();

            let no_module_name: &str = "<no-name>";

            let module_name = match name_section {
                Some(name_section) => match name_section.module() {
                    Some(name_section) => name_section.name(),
                    None => no_module_name,
                },
                None => no_module_name,
            };

            println!("The module name is {}", module_name);
            Ok(())
        }

        ("set", Some(args)) => {
            let module_path = args.value_of(MODULE_PATH).unwrap();
            let new_module_name = args.value_of(NEW_MODULE_NAME).unwrap();

            let module =
                parity_wasm::deserialize_file(module_path).expect("Error while deserializing file");
            let mut module = module.parse_names().expect("Error while parsing names");
            let name_section = module.names_section_mut();

            let new_module_name_subsection = ModuleNameSubsection::new(new_module_name);
            match name_section {
                Some(name_section) => *name_section.module_mut() = Some(new_module_name_subsection),
                None => {
                    let name_section =
                        NameSection::new(Some(new_module_name_subsection), None, None);

                    let mut buffer = vec![];
                    name_section
                        .serialize(&mut buffer)
                        .expect("Error while serializing name section");

                    module.set_custom_section("name", buffer);
                }
            };

            parity_wasm::serialize_to_file(module_path, module)
                .expect("Error while serializing file");
            Ok(())
        }

        c => Err(err_msg(format!("Unexpected command: {}", c.0)))?,
    }
}

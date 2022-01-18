mod database;
mod internal;
mod operations;

use crate::internal::{init, sort, structs::Options};
use clap::{App, AppSettings, Arg, ArgSettings, SubCommand};
use std::process::exit;

fn main() {
    extern "C" {
        fn geteuid() -> u32;
    }

    if unsafe { geteuid() } == 0 {
        panic!("Running amethyst as root is disallowed as it can lead to system breakage. Instead, amethyst will prompt you when it needs superuser permissions.")
    }

    let matches = App::new("Amethyst")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .set(ArgSettings::Global)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("noconfirm")
                .long("noconfirm")
                .set(ArgSettings::Global)
                .help("Complete operation without prompting user"),
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Installs a package from either the AUR or the PacMan-defined repositories")
                .aliases(&["-S", "ins"])
                .arg(
                    Arg::with_name("package(s)")
                        .help("The name of the package(s) to install")
                        .required(true)
                        .multiple(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Removes a previously installed package")
                .aliases(&["-R", "-Rs", "rm"])
                .arg(
                    Arg::with_name("package(s)")
                        .help("The name of the package(s) to remove")
                        .required(true)
                        .multiple(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("search")
                .about("Searches for the relevant packages in both the AUR and repos.")
                .aliases(&["-Ss", "sea"])
                .arg(
                    Arg::with_name("aur")
                        .short("a")
                        .long("aur")
                        .help("Search only the AUR for the package"),
                )
                .arg(
                    Arg::with_name("repo")
                        .short("r")
                        .long("repo")
                        .help("Searches only local repos for the package"),
                )
                .arg(
                    Arg::with_name("package(s)")
                        .help("The name of the package to search for")
                        .required(true)
                        .multiple(false)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("upgrade")
                .about("Upgrades locally installed packages to their latest versions")
                .aliases(&["-Syu", "upg"]),
        )
        .settings(&[
            AppSettings::GlobalVersion,
            AppSettings::VersionlessSubcommands,
            AppSettings::ArgRequiredElseHelp,
            AppSettings::InferSubcommands,
        ])
        .get_matches();

    let verbosity: i32 = matches.occurrences_of("verbose") as i32;
    let noconfirm: bool = matches.is_present("noconfirm");

    let options = Options {
        verbosity,
        noconfirm,
        asdeps: false,
    };

    init(options);

    let packages: Vec<String> = matches
        .subcommand()
        .1
        .unwrap()
        .values_of("package(s)")
        .unwrap()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    if let true = matches.is_present("install") {
        let sorted = sort(&packages, options);

        if !sorted.repo.is_empty() {
            operations::install(sorted.repo, options);
        }
        if !sorted.aur.is_empty() {
            operations::aur_install(sorted.aur, options);
        }
        if !sorted.nf.is_empty() {
            eprintln!(
                "Couldn't find packages: {} in repos or the AUR.",
                sorted.nf.join(", ")
            );
        }
        exit(0);
    }

    if let true = matches.is_present("remove") {
        operations::uninstall(packages, options);
        exit(0);
    }

    if let true = matches.is_present("upgrade") {
        operations::upgrade(options);
        exit(0);
    }

    if let true = matches.is_present("search") {
        if matches
            .subcommand_matches("search")
            .unwrap()
            .is_present("aur")
        {
            operations::aur_search(&packages[0], options);
        }
        if matches
            .subcommand_matches("search")
            .unwrap()
            .is_present("repo")
        {
            operations::search(&packages[0], options);
        }

        if !matches
            .subcommand_matches("search")
            .unwrap()
            .is_present("repo")
            && !matches
                .subcommand_matches("search")
                .unwrap()
                .is_present("aur")
        {
            operations::search(&packages[0], options);
            operations::aur_search(&packages[0], options);
        }
        exit(0);
    }
}

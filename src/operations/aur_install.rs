use crate::internal::rpc::rpcinfo;
use crate::Options;
use std::env;
use std::env::set_current_dir;
use std::fs::remove_dir_all;
use std::path::Path;
use std::process::Command;

pub fn aur_install(a: Vec<String>, options: Options) {
    let url = crate::internal::rpc::URL;
    let cachedir = format!("{}/.cache/ame/", env::var("HOME").unwrap());
    let verbosity = options.verbosity;
    let noconfirm = options.noconfirm;
    match verbosity {
        0 => {}
        1 => {
            eprintln!("Installing from AUR:");
            eprintln!("{:?}", &a);
        }
        _ => {
            eprintln!("Installing from AUR:");
            for b in &a {
                eprintln!("{:?}", b);
            }
        }
    }

    for package in a {
        let rpcres = rpcinfo(package);

        if !rpcres.found {
            break;
        }

        let pkg = &rpcres.package.as_ref().unwrap().name;

        if verbosity >= 1 {
            eprintln!("Cloning {} into cachedir", pkg);
        }

        // cloning
        set_current_dir(Path::new(&cachedir)).unwrap();
        Command::new("git")
            .arg("clone")
            .arg(format!("{}/{}", url, pkg))
            .status()
            .expect("Something has gone wrong");

        if verbosity >= 1 {
            eprintln!(
                "Cloned {} into cachedir, moving on to resolving dependencies",
                pkg
            );
            eprintln!(
                "Raw dependencies for package {} are:\n{:?}",
                pkg,
                rpcres.package.as_ref().unwrap().depends.join(", ")
            )
        }

        // dep sorting
        let sorted = crate::internal::sort(&rpcres.package.as_ref().unwrap().depends, options);

        if verbosity >= 1 {
            eprintln!("Sorted depndencies for {} are:\n{:?}", pkg, &sorted)
        }

        let newopts = Options {
            verbosity,
            noconfirm,
            asdeps: true,
        };

        if !sorted.nf.is_empty() {
            panic!(
                "Could not find dependencies {} for package {}, aborting",
                sorted.nf.join(", "),
                pkg
            );
        }

        if !noconfirm {
            let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".parse().unwrap());

            Command::new(editor)
                .arg(format!("{}/PKGBUILD", pkg))
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }

        // dep installing
        crate::operations::install(sorted.repo, newopts);
        crate::operations::aur_install(sorted.aur, newopts);

        let mut makepkg_args = vec!["-rsic", "--needed"];
        if options.asdeps {
            makepkg_args.push("--asdeps")
        }
        if options.noconfirm {
            makepkg_args.push("--noconfirm")
        }

        // package building and installing
        set_current_dir(format!("{}/{}", cachedir, pkg)).unwrap();
        Command::new("makepkg")
            .args(&makepkg_args)
            .status()
            .expect("Something has gone wrong");

        if makepkg_args.contains(&"--asdeps") {
            set_current_dir(&cachedir).unwrap();
            remove_dir_all(format!("{}/{}", cachedir, pkg)).unwrap();
        }

        // pushes package to database
        crate::database::add(rpcres.package.unwrap(), options);
    }
}

use crate::{err_unrec, inf};
use std::fs::File;
use std::io::{Error, Write};
use toml_edit::{value, Document};
use crate::mods::strs::{err_rec};

pub fn rem_pkg(pkgs: &Vec<String>) {
    let file = format!("{}/.local/ame/aurPkgs.db", std::env::var("HOME").unwrap());
    let database = String::new();
    if std::path::Path::new(&file).exists() {
        let database = std::fs::read_to_string(&file).expect("Can't Open Database");
    } else {
        err_rec(String::from("Database wasn't found, creating new one"));
        let dbFile = File::create(&file);
        let database = String::new();
    }

    let mut update_database = database;
    for i in pkgs {
        if update_database.contains(i) {
            let results = raur::search(&i);
            for res in &results {
                let database_entry = format!(
                    "{} = {{ name = \"{}\", version = \"{}\"}}\n",
                    &res[0].name, &res[0].name, &res[0].version
                );
                update_database = format!("{}", update_database.replace(&database_entry, ""));
            }
        }
    }
    let file_as_path = File::create(std::path::Path::new(&file)).unwrap();
    let db_update_res = write!(&file_as_path, "{}", update_database);
    match db_update_res {
        Ok(_) => inf(format!("Database update successful")),
        Err(_) => err_unrec(format!("Couldn't update database")),
    }
}

pub fn add_pkg(from_repo: bool, pkg: &str) -> Result<(), Error> {
    let file = format!("{}/.local/ame/aurPkgs.db", std::env::var("HOME").unwrap());
    let database = std::fs::read_to_string(&file).expect("cant open database");
    let mut file_as_path = File::create(std::path::Path::new(&file))?;

    let mut db_parsed = database.parse::<Document>().expect("invalid Database");
    if from_repo == false {
        let results = raur::search(&pkg);
        for res in &results {
            for r in res {
                db_parsed[&r.name]["name"] = value(&r.name);
                db_parsed[&r.name]["version"] = value(&r.version);
            }
        }
    } else {
        db_parsed[&pkg]["name"] = value(pkg);
        db_parsed[&pkg]["version"] = value(pkg);
    }
    file_as_path
        .write_all(format!("{}", db_parsed).as_bytes())
        .unwrap();
    Ok(())
}

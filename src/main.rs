extern crate clap;

use console::style;

use sqlite::*;
use sqlite::app::*;
use sqlite::utils::*;
use sqlite::define_table::*;
use sqlite::insert_row::*;


fn main() {
    let matches = get_matches();

    let mut app = App::new();

    if let Some(path) = matches.value_of("path") {
        if let Err(err) = app.connect_in_file(path) {
            println!("Cannot connect to sqlite. Error: {}", err);
            std::process::exit(1);
        }
    } else {
        if let Err(err) = app.connect_in_memory() {
            println!("Cannot connect to sqlite. Error: {}", err);
            std::process::exit(1);
        }
    }

    let (in_memory, path_text) = get_sqlite_path(&app);

    loop {
        clear();

        println!("Welcome to sqlite interactive demo.");
        println!("Sqlite is running in {}\n", color_sqlite_path(in_memory, path_text.as_str()));

        if let Some(table) = app.active_table() {
            println!("Current table: \'{}\'", style(table).green());
        } else {
            println!("{} table selected", style("No").red());
        }

        use sqlite::MainMenuOption::*;
        match ask_main_menu(&app).expect("IO error") {
            DefineTable => { define_table(&mut app); },
            SelectTable => { set_active_table(&mut app); },
            InsertRow => { insert_row(&mut app); },
            Display => {},
            Quit => { break; },
        }
    }
}

fn get_matches<'a>() -> clap::ArgMatches<'a> {
    use clap::*;
    App::new("Sqlite demonstration")
        .version("0.1.0")
        .author("Karol Milewczyk <kmilewczyk96@gmail.com>")
        .about("Implementation of uni assignment. It is interactive demonstration of sqlite basic features")
        .arg(Arg::with_name("path")
            .short("p")
            .long("path")
            .takes_value(true)
            .help("Forces sqlite to work on a file in the specified path"))
        .get_matches()
}

fn get_sqlite_path(app: &App) -> (bool, String) {
    if let Some(path) = app.path() {
        let abs_path = match std::fs::canonicalize(path) {
            Ok(result) => result,
            Err(err) => {
                println!("Failed to find a file. Error: {}", err);
                std::process::exit(1);
            }
        };

        (false, String::from(abs_path.to_str().expect("Non valid Unicode in path")))

    } else {
        (true, String::from("memory"))
    }
}

fn color_sqlite_path(in_memory: bool, text: &str) -> console::StyledObject<&str>{
    match in_memory {
        true => style(text).red(),
        false => style(text).green(),
    }
}

use console::style;

use sqlite::*;
use sqlite::app::*;
use sqlite::utils::*;
use sqlite::define_table::*;


fn main() {
    let mut app = App::new();
    if let Err(err) = app.connect_in_memory() {
        println!("Cannot connect to sqlite. Error: {}", err);
        std::process::exit(1);
    }

    loop {
        clear();

        println!("Welcome to sqlite interactive demo.\n");
        if let Some(table) = &app.active_table {
            println!("Current table: \'{}\'", style(table.as_str()).green());
        } else {
            println!("{} table selected", style("No").red());
        }

        use sqlite::MainMenuOption::*;
        match ask_main_menu(&app).expect("IO error") {
            DefineTable => { define_table(&mut app); },
            InsertRow => {},
            Display => {},
            Quit => { break; },
        }
    }
}

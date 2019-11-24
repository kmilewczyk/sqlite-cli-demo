extern crate rusqlite;
extern crate prettytable;
extern crate dialoguer;
extern crate console;
#[macro_use] extern crate enum_primitive_derive;
extern crate num_traits;
#[macro_use] extern crate lazy_static;
extern crate regex;

use dialoguer::{Select, Input};
use num_traits::FromPrimitive;
use console::style;

pub mod define_table;
pub mod utils;
pub mod app;
pub mod insert_row;
pub mod display;

use crate::app::App;
use crate::utils::{ValidatorAdaptor, validate_table_name, wait_for_keypress, clear};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum MainMenuOption {
    DefineTable = 0,
    SelectTable = 1,
    InsertRow = 2,
    Display = 3,
    Quit = 4,
}


pub fn ask_main_menu(app: &App) -> Result<MainMenuOption, std::io::Error>{
    let option = Select::with_theme(&app.view.dialog_theme)
        .default(0)
        .item("Define new table")
        .item("Select existing table")
        .item("Insert row")
        .item("Display or remove rows")
        .item("Quit")
        .interact();

    match option {
        Ok(opt) => Ok(MainMenuOption::from_usize(opt).unwrap()),
        Err(err) => Err(err),
    }
}

pub fn set_active_table(app: &mut App) {
    clear();
    println!("Give table name\n");

    let name: String = Input::with_theme(&app.view.dialog_theme)
        .with_prompt("name")
        .validate_with(ValidatorAdaptor::new(validate_table_name, format!("Table name must be alphanumeric")))
        .interact().expect("IO error");

    match app.set_active_table(name.as_str()) {
        Ok(_) => {},
        Err(err) => {
            println!("Could not set \'{}\' table", style(name).cyan());
            println!("Reason: {}\n", err);

            wait_for_keypress();
        }
    }

}

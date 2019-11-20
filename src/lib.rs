extern crate rusqlite;
extern crate prettytable;
extern crate dialoguer;
extern crate console;
#[macro_use] extern crate enum_primitive_derive;
extern crate num_traits;
#[macro_use] extern crate lazy_static;
extern crate regex;

use dialoguer::{Select};
use num_traits::FromPrimitive;

pub mod define_table;
pub mod utils;
pub mod app;

use crate::app::App;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
pub enum MainMenuOption {
    DefineTable = 0,
    InsertRow = 1,
    Display = 2,
    Quit = 3,
}


pub fn ask_main_menu(app: &App) -> Result<MainMenuOption, std::io::Error>{
    let option = Select::with_theme(&app.view.dialog_theme)
        .default(0)
        .item("Define table")
        .item("Insert row")
        .item("Display")
        .item("Quit")
        .interact();

    match option {
        Ok(opt) => Ok(MainMenuOption::from_usize(opt).unwrap()),
        Err(err) => Err(err),
    }
}

// Make two modes
// One will prompt for each field a value
// Second will try add consecutive rows by in-app incrementing their fields.
// It will prepare statement for each table
//

use crate::app::App;
use crate::utils::*;

use dialoguer::{ Select, Input };
use num_traits::FromPrimitive;

use rusqlite::{NO_PARAMS, Rows, types::Value};

pub fn insert_row(app: &mut App) {
    clear();

    if app.active_table().is_none() {
        println!("No active table selected\n");
        wait_for_keypress();
        return;
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
    enum Answer{
        UserInsertRow = 0,
        ConsecutiveInsert = 1,
        Back = 2,
    }
    use Answer::*;

    loop {
        let option = Select::with_theme(&app.view.dialog_theme)
            .default(0)
            .item("User defined insert row")
            .item("Add consecutive row")
            .item("Back")
            .interact().expect("IO error");

        match Answer::from_usize(option).unwrap() {
            UserInsertRow => { user_defined_insert(app); },
            ConsecutiveInsert => { consecutive_rows(app); },
            Back => { break; },
        }
    }
}


pub struct Column {
    pub name: String,
    pub sqltype: String,
}

fn user_defined_insert(app: &mut App) {
    clear();
    let mut columns_info = match get_table(app) {
        Ok(info) => info,
        Err(err) => {
            println!("{}", err);
            wait_for_keypress();
            return;
        }
    };

    let mut insert_query = format!("INSERT INTO {} VALUES (", app.active_table().expect("No active table"));
    let column_count = columns_info.len();

    for (i, column) in columns_info.iter_mut().enumerate() {
        clear();
        println!("Your query: {}", insert_query);

        // TODO: No validation. Its open to sql injection. Regex would be complicated. Its quick project so low chances.
        let value: String = Input::with_theme(&app.view.dialog_theme)
            .with_prompt(
                format!("Set value for column {} (type {})", column.name, column.sqltype).as_str()
            )
            .interact().expect("IO error");

        insert_query.push_str(value.as_str());

        if i < column_count-1 {
            insert_query.push(',');
        }
    }

    insert_query.push(')');
    clear();
    if ask_for_confirmation_before_query(app, &insert_query) {
        match app.connection.as_ref().expect("No connection").execute(insert_query.as_str(), NO_PARAMS) {
            Ok(_) => {},
            Err(err) => {
                println!("Could not insert rows. Error: {}", err);
                wait_for_keypress();
            }
        }
    }
}

pub fn save_column_info(rows: Rows) -> Result<Vec<Column>, String> {
    let mut info: Vec<Column> = Vec::new();

    let columns = match rows.columns() {
        Some(cols) => cols,
        None => {
            return Err(format!("No columns could be retieved from table"));
        }
    };

    for column in columns {
        if let Some(sqltype) = column.decl_type() {
            info.push(Column {name: String::from(column.name()), sqltype: String::from(sqltype)});
        } // skip expression columns
    }

    if info.is_empty() {
        return Err(format!("No columns could be retieved from table"));
    }


    Ok(info)
}


pub fn get_table(app: &App) -> Result<Vec<Column>, String>{
    let connection = app.connection.as_ref().expect("No defined connection to sqlite");
    let name = app.active_table().expect("No active table was chosen");

    match connection.prepare(format!("SELECT * FROM {} LIMIT 0;", name).as_str()) {
        Ok(mut statement) => {
            match statement.query(NO_PARAMS) {
                Ok(table) => {
                    // Not the most efficient way of doing things.
                    // Could move statement to some lifetimed struct that would get some method to
                    // getting columns. It's acceptable though because its user interaction with
                    // string data in terms of at most tens of &str to copy.
                    return save_column_info(table);
                },
                Err(err) => { return Err(format!("Could not execute query. Error: {}", err)) },
            }
        }
        Err(err) => { return Err(format!("Could not prepare query: {}", err)); },
    };
}

fn consecutive_rows(app: &App) {
    clear();

    if let Err(err) = insert_consecutive_rows(app) {
        println!("Failed to insert consequtive row. {}\n", err);
    } else {
        println!("Inserted row");
    }

    wait_for_keypress();
}

// Why (x+1)||1
// Mostly beacuse doing 'a'+1 is considered 1
// x'00' || 1 is NULL
// combination (x+1)||1 mostly works for all literals

fn insert_consecutive_rows(app: &App) -> Result<(), String> {
    let column_definitions = get_table(app)?;

    let connection = app.connection.as_ref().expect("No defined connection to sqlite");
    let name = app.active_table().expect("No active table was chosen");

    let column_count = column_definitions.len();
    let mut query = format!("INSERT INTO {} SELECT", name);
    for (i, column) in column_definitions.iter().enumerate() {
        query.push_str(format!(" MAX(({}+1)||1)", column.name).as_str());
        if i < column_count-1 {
            query.push(',');
        }
    }

    query.push_str(format!(" FROM {};", name).as_str());

    let count = connection.execute(query.as_str(), NO_PARAMS).map_err(|err| format!("{}", err))?;
    if count != 1 {
        return Err(String::from("Row was not inserted"));
    }

    Ok(())
}

use crate::app::App;

use rusqlite::{Rows, ToSql, NO_PARAMS, types::Value};

use prettytable::{Table, cell};

use crate::utils::{ wait_for_keypress, clear, truncate };

use dialoguer::Select;

use num_traits::FromPrimitive;

use std::collections::HashMap;

use crate::insert_row::{ Column, get_table };

pub fn draw_query(app: &App, query: &str, params: &[&dyn ToSql]) -> Result<(), String> {
    let connection = app.connection.as_ref().ok_or(format!("No connection is set to sqlite"))?;

    let mut statement = match connection.prepare(query) {
        Ok(stmt) => stmt,
        Err(err) => {
            return Err(format!("Could not display table {}", err));
        }
    };

    let mut rows = match statement.query(params) {
        Ok(rs) => rs,
        Err(err) => {
            return Err(format!("Failed to execute query. {}", err));
        },
    };

    draw_from_rows(&mut rows).map_err(|err| format!("{}", err))?;

    Ok(())
}

fn draw_from_rows(rows: &mut Rows) -> rusqlite::Result<()> {
    use prettytable::*;

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.set_titles(
        rows.columns().unwrap_or(Vec::new())
        .iter().map(|c| cell!(format!("{}:{}",c.name(), c.decl_type().unwrap_or("none"))))
        .collect()
    );

    while let Some(row) = rows.next()? {
        let mut i = 0;
        let mut cells: Vec<Cell> = Vec::new();

        while let Ok(data) = row.get::<usize, Value>(i) {
            cells.push(cell!(value_repr(&data)));
            i += 1;
        }

        table.add_row(Row::new(cells));
    }

    table.printstd();

    Ok(())
}

pub fn draw_paginate(app: &App, at_once: usize, page: usize, sort_options: &HashMap<String, bool>) -> Result<(), String>{
    // LIMIT x, y is not optimal as it reads whole table anyway.
    // TODO: optimize it by using WHERE condition
    let name = app.active_table().ok_or(format!("No active table was defined"))?;

    let sorting_query_part = {
        if sort_options.is_empty() {
            String::default()
        } else {
            let mut query = String::from("ORDER BY");
            let option_count = sort_options.len();

            for (i, (col, ascending)) in sort_options.iter().enumerate() {
                let direction = if *ascending { "ASC" } else { "DESC" };
                query.push_str(format!(" {} {}", col, direction).as_str());
                if i < option_count-1 {
                    query.push(',');
                }
            }

            query
        }
    };

    println!("{}", sorting_query_part);

    draw_query(app, format!("SELECT * FROM {} {} LIMIT {}, {}", name, sorting_query_part, page*at_once, (page+1)*at_once).as_str(), NO_PARAMS)?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
enum DisplayAnswer{
    NextPage = 0,
    PreviousPage = 1,
    DefineSorting = 2,
    DeleteRows = 3,
    GoBack = 4,
}


pub fn display_table(app: &App) {
    static ROWS_PER_PAGE: usize = 50;
    let mut starting_row: usize = 0;
    let mut last_chosen = 0;
    let mut sorting_options: HashMap<String, bool> = HashMap::new();

    if app.active_table().is_none() {
        clear();
        println!("No active table is chosen");
        wait_for_keypress();
        return;
    }

    let columns: Vec<Column> = {
        match get_table(app) {
            Ok(cols) => cols,
            Err(err) => {
                clear();
                println!("{}", err);
                wait_for_keypress();
                return;
            },
        }
    };

    loop {
        clear();

        println!("Rows from {} to {}", starting_row, starting_row+ROWS_PER_PAGE);

        if let Err(err) = draw_paginate(app, ROWS_PER_PAGE, starting_row, &sorting_options) {
            println!("Could display table!. {}", err);
            wait_for_keypress();
        }

        println!("");

        last_chosen = Select::with_theme(&app.view.dialog_theme)
            .default(last_chosen)
            .item("Next 50 rows")
            .item("Previous 50 rows")
            .item("Define sorting criteria")
            .item("Delete rows on condition")
            .item("Back")
            .interact().expect("IO error");

        use DisplayAnswer::*;
        match DisplayAnswer::from_usize(last_chosen).unwrap() {
            NextPage => { starting_row += ROWS_PER_PAGE; },
            PreviousPage => { if starting_row > 0 { starting_row -= ROWS_PER_PAGE; }; },
            DefineSorting => { set_sorting_options(app, &columns, &mut sorting_options); },
            DeleteRows => { },
            GoBack => { break; },
        }
    }
}

fn set_sorting_options(app: &App, columns: &Vec<Column>, sorting_options: &mut HashMap<String, bool>) {
    loop {
        let mut select = Select::with_theme(&app.view.dialog_theme);
        select.default(0);

        for column in columns {
            select.item(
                format!(
                    "Column {} ({})",
                    column.name,
                    if let Some(ascending) = sorting_options.get(&column.name) {
                        if *ascending { "ASC" } else { "DESC" }
                    } else {
                        "NONE"
                    }
                ).as_str()
            );
        }

        select.item("Back");
        let option = select.interact().expect("IO Error");

        // exit if back
        if option >= columns.len() {
            break;
        } else {
            let column_name = &columns.get(option).unwrap().name;
            if let Some(ascending) = sorting_options.get_mut(column_name) {
                if *ascending {
                    *ascending = false;
                } else {
                    sorting_options.remove(column_name);
                }
            } else {
                sorting_options.insert(column_name.clone(), true);
            }
        }
    }

}

fn value_repr(val: &Value) -> String {
    use rusqlite::types::Value::*;

    let val = match val {
        Null => format!("NULL"),
        Integer(i) => format!("{}", i),
        Real(i) => format!("{}", i),
        Text(t) => t.clone(),
        Blob(v) => format!("{:?}", v),
    };

    String::from(truncate(val.trim_start(), 20))
}

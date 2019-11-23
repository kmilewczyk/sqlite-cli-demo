use crate::app::App;

use rusqlite::{Rows, ToSql, NO_PARAMS, types::Value};

use prettytable::{Table, cell};

use crate::utils::{ wait_for_keypress, clear, truncate };


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

pub fn draw_paginate(app: &App, at_once: usize, page: usize) -> Result<(), String>{
    // LIMIT x, y is not optimal as it reads whole table anyway.
    // TODO: optimize it by using WHERE condition
    let name = app.active_table().ok_or(format!("No active table was defined"))?;

    draw_query(app, format!("SELECT * FROM {} LIMIT {}, {}", name, page*at_once, (page+1)*at_once).as_str(), NO_PARAMS)?;

    Ok(())
}

pub fn display_table(app: &App) {
    clear();

    if let Err(err) = draw_paginate(app, 50, 0) {
        println!("Could display table!. {}", err);
        wait_for_keypress();
        return;
    }

    println!("\n");
    wait_for_keypress();
}

fn value_repr(val: &Value) -> String {
    use rusqlite::types::Value::*;

    let mut val = match val {
        Null => format!("NULL"),
        Integer(i) => format!("{}", i),
        Real(i) => format!("{}", i),
        Text(t) => t.clone(),
        Blob(v) => format!("{:?}", v),
    };

    String::from(truncate(val.trim_start(), 20))
}

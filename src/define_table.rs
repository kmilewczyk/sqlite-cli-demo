use std::io;

use console::style;
use dialoguer::{Input, Select};

use num_traits::FromPrimitive;

use rusqlite::{params};

use crate::App;
use crate::utils::*;

struct TableDefinition {
    name: Option<String>,
    columns: Vec<ColumnDefinition>,
}

impl TableDefinition {
    fn new() -> Self {
        Self {
            name: None,
            columns: Vec::new(),
        }
    }
}

#[derive(Default)]
struct ColumnDefinition {
    name: String,
    sql_type: String,
}

#[derive(Clone)]
enum DefineTablePromptOption {
    SetName,
    AddColumn,
    SetColumn(String),
    Create,
    Cancel,
}

pub fn define_table(app: &mut App) {
    let mut table = TableDefinition::new();
    let mut query: String;

    loop {
        clear();

        println!("Create new table\n");

        print_preview(app, &table);
        println!("\n");

        use DefineTablePromptOption::*;
        match define_table_prompt(app, &table).expect("IO error") {
            SetName => { set_name(app, &mut table) },
            AddColumn => { add_column(app, &mut table); },
            SetColumn(name) => { clear(); update_or_delete_column(app, &mut table, name); }
            Create => {
                query = create_query_from_definition(&table);
                if ask_for_confirmation_before_query(app, &query) {
                    break;
                }
            },
            Cancel => { return; },
        }
    }

    match app.connection.as_ref().unwrap().execute(query.as_str(), params![]) {
        Ok(_) => {
            match app.set_active_table(table.name.unwrap().as_str()) {
                Ok(_) => {},
                Err(err) => panic!(err),
            }
        },
        Err(err) => {
            println!("Could not execute a query! Reason: {}", err);
            wait_for_keypress();
        },
    }
}


fn define_table_prompt(app: &App, table: &TableDefinition) -> Result<DefineTablePromptOption, io::Error>{
    use DefineTablePromptOption::*;

    let mut select = Select::with_theme(&app.view.dialog_theme);
    let mut options: Vec<DefineTablePromptOption> = Vec::new();

    options.push(SetName);
    if let Some(_) = table.name {
        select.item("Change name");

        options.push(AddColumn);
        select.item("Add column");
        select.default(1);

        if !table.columns.is_empty() {
            for column in &table.columns {
                options.push(SetColumn(column.name.clone()));
                select.item(format!("Modify or remove \'{}\' column", &column.name).as_str());
            }

            options.push(Create);
            select.item("Create table");
        }
    } else {
        select.item("Set name");
        select.default(0);
    }

    options.push(Cancel);
    select.item("Cancel");

    match select.interact() {
        Ok(option) => Ok(options.get(option).expect("define table option not in vector").clone()),
        Err(err) => Err(err),
    }
}

fn set_name(app: &App, table: &mut TableDefinition) {
    println!("Set name for a table\n");

    let default_name = if let Some(name) = &table.name {
        name.clone()
    } else {
        String::from("")
    };

    let name = Input::with_theme(&app.view.dialog_theme)
        .with_prompt("Set name")
        .default(default_name)
        .validate_with(ValidatorAdaptor::new(validate_table_name, format!("Table name must be alphanumeric")))
        .interact().expect("IO error");

    table.name = Some(name);
}

fn add_column(app: &App, table: &mut TableDefinition) {
    let mut column = ColumnDefinition::default();

    println!("Adding new column\n");

    column.name = Input::with_theme(&app.view.dialog_theme)
        .with_prompt("Set column name")
        .validate_with(ValidatorAdaptor::new(validate_column_name, String::from("Column name must be alphanumeric")))
        .interact().expect("IO error");

    column.sql_type = Input::with_theme(&app.view.dialog_theme)
        .with_prompt("Set column type with associated keywords")
        .validate_with(ValidatorAdaptor::new(validate_sql_type, String::from("SQL type must be alphanumeric")))
        .interact().expect("IO error");

    table.columns.push(column);
}

fn update_or_delete_column(app: &App, table: &mut TableDefinition, column_name: String) {
    println!("What do you want to do with \"{}\" column?\n", column_name);

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
    enum Answer{
        SetColumn = 0,
        Delete = 1,
        Nothing = 2,
    }
    use Answer::*;

    let option = Select::with_theme(&app.view.dialog_theme)
        .default(0)
        .item("Modify definition")
        .item("Delete")
        .item("Nothing")
        .interact().expect("IO error");

    match Answer::from_usize(option).unwrap() {
        SetColumn => {
            clear();
            set_column(app, table, column_name);
        }
        Delete => {
            let pos = table.columns.iter().position(|x| x.name == column_name)
                .expect(format!("Column \'{}\'is not found in the table", column_name).as_str());
            table.columns.drain(pos..pos+1);
        }
        Nothing => { return; }
    }

}

fn set_column(app: &App, table: &mut TableDefinition, column_name: String) {
    println!("Editing \"{}\" column\n", column_name);

    let mut column: &mut ColumnDefinition = table.columns.iter_mut()
        .find(|col| col.name == column_name)
        .expect("Column not in vector");

    column.name = Input::with_theme(&app.view.dialog_theme)
        .with_prompt("Set column name")
        .default(column_name)
        .interact().expect("IO error");

    column.sql_type = Input::with_theme(&app.view.dialog_theme)
        .with_prompt("Set column type")
        .default(column.sql_type.clone())
        .interact().expect("IO error");
}

fn print_preview(_app: &App, table_definition: &TableDefinition) {
    use prettytable::*;

    let name = if let Some(name) = &table_definition.name { name } else { return };

    let columns = if !table_definition.columns.is_empty() { &table_definition.columns } else { return };

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    println!("{}", style(format!("\'{}\'", name).as_str()).cyan());
    table.set_titles(
        Row::new(columns.iter().map(|c| Cell::new(c.name.as_str())).collect())
    );
    table.add_row(
        Row::new(columns.iter().map(|c| Cell::new(c.sql_type.as_str())).collect())
    );
    table.printstd();
}

fn create_query_from_definition(table: &TableDefinition) -> String {
    let mut query = String::from("CREATE TABLE IF NOT EXISTS ");
    query.push_str(table.name.as_ref().unwrap().as_str());
    query.push('(');
    for (i, column) in table.columns.iter().enumerate() {
        query.push_str(&column.name);
        query.push(' ');
        query.push_str(&column.sql_type);
        if i < table.columns.len()-1 {
            query.push(',');
        }
    }
    query.push(')');

    query
}

use console::{Style};
use rusqlite::{Connection};
use dialoguer::{theme::ColorfulTheme};

pub struct AppView {
    pub dialog_theme: ColorfulTheme,
}

impl AppView {
    fn new() -> Self {
        Self {
            dialog_theme: ColorfulTheme {
                values_style: Style::new().yellow().dim(),
                indicator_style: Style::new().yellow().bold(),
                yes_style: Style::new().yellow().dim(),
                no_style: Style::new().yellow().dim(),
                ..ColorfulTheme::default()
            },
        }
    }
}

#[derive(PartialEq, Eq)]
enum SqliteConnection {
    File(String),
    Memory,
}

pub struct App {
    pub view: AppView,
    pub connection: Option<Connection>,
    connection_type: SqliteConnection,
    pub active_table: Option<String>,
}

impl App {
    pub fn new() -> Self {
        App {
            view: AppView::new(),
            connection: None,
            connection_type: SqliteConnection::Memory,
            active_table: None,
        }
    }

    pub fn connect_in_file(&mut self, path: &str) -> rusqlite::Result<()> {
        self.connection_type = SqliteConnection::File(String::from(path));
        self.connection = Some(Connection::open(path)?);
        Ok(())
    }

    pub fn connect_in_memory(&mut self) -> rusqlite::Result<()> {
        self.connection_type = SqliteConnection::Memory;
        self.connection = Some(Connection::open_in_memory()?);
        Ok(())
    }

    pub fn is_in_memory(&self) -> bool {
        self.connection_type == SqliteConnection::Memory
    }

    pub fn path(&self) -> Option<&str> {
        match &self.connection_type {
            SqliteConnection::File(path) => Some(path.as_str()),
            SqliteConnection::Memory => None,
        }
    }
}

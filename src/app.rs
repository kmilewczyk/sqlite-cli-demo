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

pub struct App {
    pub view: AppView,
    pub connection: Option<Connection>,
    pub active_table: Option<String>,
}

impl App {
    pub fn new() -> Self {
        App {
            view: AppView::new(),
            connection: None,
            active_table: None,
        }
    }

    pub fn connect_in_memory(&mut self) -> rusqlite::Result<()> {
        self.connection = Some(Connection::open_in_memory()?);
        Ok(())
    }
}

use console::Term;
use crate::app::App;
use dialoguer::Confirmation;

pub fn clear() {
    Term::stdout().clear_screen().expect("IO error");
}

pub fn wait_for_keypress() {
    use std::io;
    use std::io::prelude::*;

    println!("Press any key to continue...");
    io::stdin().read(&mut [0u8]).unwrap();
}

pub fn validate_column_name(input: &str) -> bool {
    use regex::Regex;

    lazy_static! {
        static ref COLUMN_REGEX: Regex = Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    }

    COLUMN_REGEX.is_match(input)
}

pub fn validate_table_name(input: &str) -> bool {
    use regex::Regex;

    lazy_static! {
        static ref TABLE_REGEX: Regex = Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    }

    TABLE_REGEX.is_match(input)
}

pub fn validate_sql_type(input: &str) -> bool {
    use regex::Regex;

    lazy_static! {
        static ref TYPE_REGEX: Regex = Regex::new("^[a-zA-Z_][a-zA-Z0-9_ ]*$").unwrap();
    }

    TYPE_REGEX.is_match(input)
}

#[derive(Debug)]
pub struct ValidationError {
    cause: String,
}

impl ValidationError {
    pub fn new(cause: String) -> Self {
        Self {
            cause: cause
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt (&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cause)
    }
}

pub struct ValidatorAdaptor {
    validate_function: fn(&str) -> bool,
    error_reason: String,
}

impl ValidatorAdaptor {
    pub fn new(validate_function: fn(&str)->bool, error_reason: String) -> Self {
        Self {
            validate_function: validate_function,
            error_reason: error_reason,
        }
    }
}

impl dialoguer::Validator for ValidatorAdaptor {
    type Err = ValidationError;

    fn validate(&self, text: &str) -> Result<(), ValidationError > {
        if (self.validate_function)(text) {
            Ok(())
        } else {
            Err(ValidationError::new(self.error_reason.clone()))
        }
    }
}


pub fn ask_for_confirmation_before_query(app: &App, query: &String) -> bool{
    println!("You are about to execute following query:");
    println!("{}\n", query);

    Confirmation::with_theme(&app.view.dialog_theme).with_text("Do you proceed?").interact().expect("IO error")
}

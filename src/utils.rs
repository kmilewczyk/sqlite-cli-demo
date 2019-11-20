use console::Term;

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

pub fn validate_sql_type(input: &str) -> bool {
    use regex::Regex;

    lazy_static! {
        static ref TYPE_REGEX: Regex = Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    }

    TYPE_REGEX.is_match(input)
}

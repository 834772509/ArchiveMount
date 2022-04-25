use console::style;

use crate::QUIET;

pub enum ConsoleType {
    Info,
    Success,
    Warning,
    Err,
}

pub fn writeConsole(consoleType: ConsoleType, message: &str) {
    if *QUIET.lock().unwrap() { return; }
    let title = match &consoleType {
        ConsoleType::Info => style("Info   ").cyan(),
        ConsoleType::Success => style("Success").green(),
        ConsoleType::Warning => style("Warning").yellow(),
        ConsoleType::Err => style("Err    ").red(),
    };
    println!("  {}      {}", &title, message);
}

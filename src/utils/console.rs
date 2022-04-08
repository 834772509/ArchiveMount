use console::style;

pub enum ConsoleType {
    Info,
    Success,
    Warning,
    Err,
}

pub fn writeConsole(consoleType: ConsoleType, message: &str) {
    let title = match &consoleType {
        ConsoleType::Info => style("Info   ").cyan(),
        ConsoleType::Success => style("Success").green(),
        ConsoleType::Warning => style("Warning").yellow(),
        ConsoleType::Err => style("Err  ").red().on_black().bold(),
    };
    println!("  {}      {}", &title, message);
}

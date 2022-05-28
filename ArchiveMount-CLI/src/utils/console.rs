use console::style;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleType {
    Info,
    Success,
    Warning,
    Error,
}


impl std::str::FromStr for ConsoleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(ConsoleType::Info),
            "success" => Ok(ConsoleType::Success),
            "warning" => Ok(ConsoleType::Warning),
            "error" => Ok(ConsoleType::Error),
            _ => Err(format!("'{}' is not a valid value for ConsoleType", s))
        }
    }
}

pub fn writeConsole(consoleType: ConsoleType, message: &str) {
    let title = match &consoleType {
        ConsoleType::Info => style("Info   ").cyan(),
        ConsoleType::Success => style("Success").green(),
        ConsoleType::Warning => style("Warning").yellow(),
        ConsoleType::Error => style("Err    ").red(),
    };
    println!("  {}      {}", &title, message);
}

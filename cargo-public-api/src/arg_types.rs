use anyhow::anyhow;

use std::str::FromStr;

use crate::{markdown::Markdown, output_formatter::OutputFormatter, plain::Plain};

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ArgEnum)]
#[clap(rename_all = "lower")]
pub enum DenyMethod {
    /// All forms of API diffs are denied: additions, changes, deletions.
    All,
}

#[derive(Debug)]
pub enum OutputFormat {
    Plain,
    Markdown,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(OutputFormat::Plain),
            "markdown" => Ok(OutputFormat::Markdown),
            _ => Err(anyhow!("See --help")),
        }
    }
}

impl OutputFormat {
    pub fn formatter(&self) -> Box<dyn OutputFormatter> {
        match self {
            OutputFormat::Plain => Box::new(Plain),
            OutputFormat::Markdown => Box::new(Markdown),
        }
    }
}

#[derive(Debug)]
pub enum Color {
    Auto,
    Never,
    Always,
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Color::Auto),
            "never" => Ok(Color::Never),
            "always" => Ok(Color::Always),
            _ => Err(anyhow!("See --help")),
        }
    }
}

impl Color {
    pub fn active(&self) -> bool {
        match self {
            Color::Auto => atty::is(atty::Stream::Stdout), // We should not assume Stdout here, but good enough for now
            Color::Never => false,
            Color::Always => true,
        }
    }
}

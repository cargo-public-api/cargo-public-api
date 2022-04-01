use anyhow::anyhow;

use std::str::FromStr;

use crate::{markdown::Markdown, output_formatter::OutputFormatter, plain::Plain};

#[derive(Debug)]
pub enum OutputFormat {
    Plain,
    Markdown,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "plain" {
            Ok(OutputFormat::Plain)
        } else if s == "markdown" {
            Ok(OutputFormat::Markdown)
        } else {
            Err(anyhow!("See --help"))
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
        if s == "auto" {
            Ok(Color::Auto)
        } else if s == "never" {
            Ok(Color::Never)
        } else if s == "always" {
            Ok(Color::Always)
        } else {
            Err(anyhow!("See --help"))
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

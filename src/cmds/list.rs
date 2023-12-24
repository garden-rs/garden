use anyhow::Result;
use clap::Parser;

use crate::{display::Color, model};

/// List available gardens, groups, trees and commands
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ListOptions {
    /// Long listing
    #[arg(short)]
    long_listing: bool,
    /// List commands
    #[arg(long, short)]
    commands: bool,
}

pub fn main(app_context: &model::ApplicationContext, options: &ListOptions) -> Result<()> {
    let config = app_context.get_root_config_mut();

    if options.commands {
        println!("{}", Color::blue("commands:"));
        for cmd in config.commands.keys() {
            println!("  - {}", Color::yellow(cmd));
        }
        return Ok(());
    }

    if !config.trees.is_empty() {
        println!("{}", Color::blue("trees:"));
        if options.long_listing {
            for (name, tree) in config.trees.iter() {
                let default_display_path = tree.get_path().get_expr();
                let display_path = tree.get_path().get_value().unwrap_or(default_display_path);
                if tree.commands.is_empty() {
                    println!(
                        "  {}{} {}",
                        Color::yellow(name),
                        Color::yellow(":"),
                        Color::green(display_path)
                    );
                } else {
                    println!("  {}{}", Color::yellow(name), Color::yellow(":"));
                    println!(
                        "    {} {}",
                        Color::yellow("path:"),
                        Color::green(display_path)
                    );
                    println!("    {}", Color::yellow("commands:"));
                    for cmd in tree.commands.keys() {
                        println!("      - {}", Color::green(cmd));
                    }
                }
            }
        } else {
            for name in config.trees.keys() {
                println!("  - {}", Color::yellow(name));
            }
        }
    }

    if !config.groups.is_empty() {
        println!("{}", Color::blue("groups:"));
        if options.long_listing {
            for (name, group) in &config.groups {
                println!("  {}{}", Color::yellow(name), Color::yellow(":"));
                for member in &group.members {
                    println!("    - {}", Color::green(member));
                }
            }
        } else {
            for name in config.groups.keys() {
                println!("  - {}", Color::yellow(name));
            }
        }
    }

    if !config.gardens.is_empty() {
        println!("{}", Color::blue("gardens:"));
        for garden in config.gardens.keys() {
            println!("  - {}", Color::yellow(garden));
        }
    }

    Ok(())
}

use anyhow::Result;
use clap::Parser;

use crate::model;

/// List available gardens, groups, trees and commands
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ListOptions {
    /// List commands
    #[arg(long, short)]
    commands: bool,
}

pub fn main(app_context: &model::ApplicationContext, options: &ListOptions) -> Result<()> {
    let config = app_context.get_root_config_mut();

    if options.commands {
        println!("commands:");
        for cmd in config.commands.keys() {
            println!("- {cmd}");
        }
        return Ok(());
    }

    if !config.gardens.is_empty() {
        println!("gardens:");
        print!("    ");
        for garden in config.gardens.keys() {
            print!("{garden} ");
        }
        println!();
    }

    if !config.groups.is_empty() {
        println!("groups:");
        print!("    ");
        for name in config.groups.keys() {
            print!("{name} ");
        }
        println!();
    }

    if !config.trees.is_empty() {
        println!("trees:");
        print!("    ");
        for name in config.trees.keys() {
            print!("{name} ");
        }
        println!();
    }

    Ok(())
}

use anyhow::Result;
use clap::Parser;

/// List available gardens, groups, trees and commands
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ListOptions {
    /// List commands
    #[arg(long, short)]
    commands: bool,
}

use super::super::model;

pub fn main(app: &mut model::ApplicationContext, options: &ListOptions) -> Result<()> {
    let config = app.get_root_config_mut();

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
        for garden in config.gardens.values() {
            print!("{} ", garden.get_name());
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
        for tree in &config.trees {
            print!("{} ", tree.get_name());
        }
        println!();
    }

    Ok(())
}

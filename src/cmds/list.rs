use anyhow::Result;

use super::super::model;


pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    let config = &app.config;

    if !config.gardens.is_empty() {
        println!("gardens:");
        print!("    ");
        for garden in &config.gardens {
            print!("{} ", garden.name);
        }
        println!("");
    }

    if !config.groups.is_empty() {
        println!("groups:");
        print!("    ");
        for group in &config.groups {
            print!("{} ", group.name);
        }
        println!("");
    }

    if !config.trees.is_empty() {
        println!("trees:");
        print!("    ");
        for tree in &config.trees{
            print!("{} ", tree.name);
        }
        println!("");
    }

    Ok(())
}

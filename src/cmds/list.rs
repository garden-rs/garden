use anyhow::Result;

use super::super::model;


pub fn main(app: &mut model::ApplicationContext) -> Result<()> {

    let config = app.get_config_mut();

    if !config.gardens.is_empty() {
        println!("gardens:");
        print!("    ");
        for garden in &config.gardens {
            print!("{} ", garden.get_name());
        }
        println!("");
    }

    if !config.groups.is_empty() {
        println!("groups:");
        print!("    ");
        for group in &config.groups {
            print!("{} ", group.get_name());
        }
        println!("");
    }

    if !config.trees.is_empty() {
        println!("trees:");
        print!("    ");
        for tree in &config.trees {
            print!("{} ", tree.get_name());
        }
        println!("");
    }

    Ok(())
}

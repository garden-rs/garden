use ::config;
use ::model;


pub fn main(options: &mut model::CommandOptions) {
    let cfg = config::new(&options.filename, options.verbose);

    if !cfg.gardens.is_empty() {
        println!("gardens:");
        print!("    ");
        for garden in &cfg.gardens {
            print!("{} ", garden.name);
        }
        println!("");
    }

    if !cfg.groups.is_empty() {
        println!("groups:");
        print!("    ");
        for group in &cfg.groups {
            print!("{} ", group.name);
        }
        println!("");
    }

    if !cfg.trees.is_empty() {
        println!("trees:");
        print!("    ");
        for tree in &cfg.trees{
            print!("{} ", tree.name);
        }
        println!("");
    }
}

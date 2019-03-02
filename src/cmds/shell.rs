use ::cmd;
use ::eval;
use ::config;
use ::model;
use ::query;


pub fn main(options: &mut model::CommandOptions) {
    options.args.insert(0, "garden shell".to_string());

    let mut tree = String::new();
    let mut garden = String::new();
    let mut garden_opt: Option<String> = None;

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden shell - run a shell in an evaluated tree context");

        ap.refer(&mut tree).required()
            .add_argument("tree", argparse::Store, "tree to evaluate");

        ap.refer(&mut garden)
            .add_argument("garden", argparse::Store, "garden to evaluate");

        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    let verbose = options.is_debug("config::new");
    let mut cfg = config::new(&options.filename, verbose);
    if options.is_debug("config") {
        debug!("{}", cfg);
    }

    if !garden.is_empty() {
        garden_opt = Some(garden);
    }

    let mut exit_status: i32 = 0;
    match query::tree_context(&cfg, &tree, garden_opt) {
        Ok(context) => {
            let expr = cfg.shell.to_string();
            let shell = eval::tree_value(
                &mut cfg, &expr, context.tree, context.garden);

            let commands = vec!(shell);
            exit_status = cmd::exec_in_context(
                &mut cfg, &context, options.quiet, options.verbose, &commands);
        }
        Err(err) => {
            error!("{}", err);
        }
    }

    std::process::exit(exit_status);
}

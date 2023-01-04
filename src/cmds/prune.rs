use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use std::io::prelude::*;

use super::super::cmd;
use super::super::errors;
use super::super::model;
use super::super::model::Color;

/// Get the default number of prune jobs to run in parallel
fn default_num_jobs() -> usize {
    match std::thread::available_parallelism() {
        Ok(value) => std::cmp::max(value.get(), 3),
        Err(_) => 4,
    }
}

/// Remove unreferenced Git repositories
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct PruneOptions {
    /// Number of parallel jobs
    #[arg(short = 'j', long = "jobs", default_value_t = default_num_jobs())]
    num_jobs: usize,
    /// Set the maximum prune depth
    #[arg(long, short = 'd', default_value_t = -1)]
    max_depth: isize,
    /// Only prune starting at the given depth
    #[arg(long, default_value_t = -1)]
    min_depth: isize,
    /// Only prune at the exact depth. Alias for '--min-depth=# --max-depth=#'
    #[arg(long, default_value_t = -1)]
    exact_depth: isize,
    /// Prune all repositories without prompting (DANGER!)
    #[arg(long)]
    no_prompt: bool,
    /// Enable deletion [default: deletion is disabled]
    #[arg(long = "rm")]
    remove: bool,
    /// Limit pruning to the specified subdirectories
    paths: Vec<String>,
}

/// Main entry point for the "garden prune" command
pub fn main(app: &mut model::ApplicationContext, options: &mut PruneOptions) -> Result<()> {
    let config = app.get_root_config_mut();

    // At least two threads must be running in order for the TraverseFilesystem task to
    // be able to produce results. Otherwise we'll block in the PromptUser thread without
    // making progress.
    if options.num_jobs < 3 {
        options.num_jobs = 3;
    }

    // Do not allow min_depth to be greater than max_depth.
    if options.max_depth >= 0 && options.max_depth < options.min_depth {
        println!("error: --max-depth cannot be less than --min-depth");
        std::process::exit(errors::EX_USAGE);
    }

    // --exact-depth <depth> is an alias for --min-depth <depth> --max-depth <depth>.
    if options.exact_depth >= 0 {
        if options.min_depth >= 0 || options.max_depth >= 0 {
            println!("error: --exact-depth cannot be used with --min-depth and --max-depth");
            std::process::exit(errors::EX_USAGE);
        }
        options.min_depth = options.exact_depth;
        options.max_depth = options.exact_depth;
    }

    let exit_status = prune(config, options, &options.paths)?;

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}

/// PathBufMessage is sent across channels between the TraverseFilesystem,
/// PromptUser and RemovePaths tasks. The Path variant contains a PathBuf to process and
/// the Finished variant is used to signal the end of the message stream.

enum PathBufMessage {
    Path(std::path::PathBuf),
    Finished,
}

/// TraverseFilesystem walks the filesystem and sends a PathBufMessage as it
/// discovers Git repositories during its traversal.

struct TraverseFilesystem<'a> {
    min_depth: isize,
    max_depth: isize,
    send_repo_path: crossbeam::channel::Sender<PathBufMessage>,
    root_path: std::path::PathBuf,
    path_filters: &'a Vec<std::path::PathBuf>,
    configured_tree_paths: &'a std::collections::HashSet<std::path::PathBuf>,
}

impl TraverseFilesystem<'_> {
    /// Start a parallel traversal over the "paths" Vec.
    fn traverse(&self) {
        self.traverse_toplevel(&self.root_path).unwrap_or(());
        self.send_repo_path
            .send(PathBufMessage::Finished)
            .unwrap_or(());
    }

    /// Traverse all of the top-level directories specified on the command-line.
    /// This function initiates the recursive walk performed by traverse_subdir().
    /// The top-level garden root is never removed.
    fn traverse_toplevel(&self, pathbuf: &std::path::PathBuf) -> std::io::Result<()> {
        let current_depth: isize = 0;
        // Traverse over all of the child directories in parallel.
        let entries: Vec<_> = std::fs::read_dir(pathbuf)?.collect();
        entries.par_iter().for_each(|entry_result| {
            if let Ok(entry) = entry_result {
                let path = entry.path();
                if let Some(path_canon) = self.validate_entry_for_traversal(&path) {
                    self.traverse_subdir(&path_canon, current_depth)
                        .unwrap_or(());
                }
            }
        });

        Ok(())
    }

    /// Recursively traverse subdirectories
    fn traverse_subdir(
        &self,
        pathbuf: &std::path::PathBuf,
        current_depth: isize,
    ) -> std::io::Result<()> {
        // Is the current directory a git worktree? We detect this by checking for ".git".
        let mut git_dir = pathbuf.to_path_buf();
        git_dir.push(".git");

        if git_dir.exists() {
            if is_within_bounds(current_depth, self.min_depth, self.max_depth) {
                self.send_repo_path
                    .send(PathBufMessage::Path(pathbuf.to_path_buf()))
                    .unwrap_or(());
            }
            return Ok(());
        }

        // Bare repositories are named "foo.git" and have a "git" file extension.
        if let Some(extension) = pathbuf.extension() {
            if extension == "git" {
                if is_within_bounds(current_depth, self.min_depth, self.max_depth) {
                    self.send_repo_path
                        .send(PathBufMessage::Path(pathbuf.to_path_buf()))
                        .unwrap_or(());
                }
                return Ok(());
            }
        }

        // Recursively traverse the child subdirectories in parallel.
        let entries: Vec<_> = std::fs::read_dir(pathbuf)?.collect();
        entries.par_iter().for_each(|entry_result| {
            if let Ok(entry) = entry_result {
                let path = entry.path();
                if let Some(path_canon) = self.validate_entry_for_traversal(&path) {
                    if is_within_max_bounds(current_depth, self.max_depth) {
                        self.traverse_subdir(&path_canon, current_depth + 1)
                            .unwrap_or(());
                    }
                }
            }
        });

        Ok(())
    }

    /// Validate a pathbuf for traversal.
    fn validate_entry_for_traversal(&self, path: &std::path::Path) -> Option<std::path::PathBuf> {
        if path.is_dir()
            && !path.is_symlink()
            && match path.file_name() {
                // Directories named ".git" are not traversed.
                Some(basename) => basename != ".git",
                None => false,
            }
        {
            if let Ok(path_canon) = path.canonicalize() {
                if !self.configured_tree_paths.contains(&path_canon) && !self.is_filtered_path(path)
                {
                    return Some(path_canon);
                }
            }
        }
        None
    }

    /// Is the path filtered by the specified path filters?
    fn is_filtered_path(&self, path: &std::path::Path) -> bool {
        // When no path filters exist then we can exit immediately.
        if self.path_filters.is_empty() {
            return false;
        }

        // Otherwise the path must be a child directory of a path filter.
        for path_filter in self.path_filters {
            if path.starts_with(path_filter) || path_filter.starts_with(path) {
                return false;
            }
        }

        // If no path filter matches then this path should not be traversed.
        true
    }
}

/// Is the value within the min/max bounds.
/// A max_depth of zero is special-cased to mean unlimited.
fn is_within_bounds(value: isize, min_depth: isize, max_depth: isize) -> bool {
    value >= min_depth && is_within_max_bounds(value, max_depth)
}

/// Is the value within the max bounds for traversal?
/// min_depth is not checked for traversal but is checked when emitting messages.
fn is_within_max_bounds(value: isize, max_depth: isize) -> bool {
    max_depth == -1 || value <= max_depth
}

/// The RemovePaths task listens for PathBufMessage messages and removes
/// paths emitted over the recv_remove_path channel.
struct RemovePaths {
    /// Paths to remove are received on this channel from the PromptUser task.
    recv_remove_path: crossbeam::channel::Receiver<PathBufMessage>,
    /// Information about paths that have already been removed are reported by
    /// sending paths to the PromptUser task via the send_finished_path channel.
    send_finished_path: crossbeam::channel::Sender<PathBufMessage>,
    /// Dry-run mode does not actually perform deletions.
    dry_run: bool,
}

impl RemovePaths {
    /// Process the recv_remove_path channel and remove paths until no messages remain.
    fn remove_paths(&self, remove_scope: &rayon::ScopeFifo<'_>) {
        loop {
            match self.recv_remove_path.recv() {
                Ok(PathBufMessage::Path(pathbuf)) => {
                    // Remove paths from the filesystem and send a completion message.
                    if !self.dry_run {
                        let pathbuf = pathbuf.to_path_buf();
                        remove_scope.spawn_fifo(move |_| {
                            rm_rf::ensure_removed(&pathbuf).unwrap_or(());

                            // Remove empty parent directorires leading up to this path.
                            let mut parent_option = pathbuf.parent();
                            while let Some(parent_pathbuf) = parent_option {
                                if !parent_pathbuf.exists() {
                                    break;
                                }
                                if std::fs::remove_dir(parent_pathbuf).is_err() {
                                    break;
                                }
                                parent_option = parent_pathbuf.parent();
                            }
                        });
                    }
                    self.send_finished_path
                        .send(PathBufMessage::Path(pathbuf))
                        .unwrap_or(());
                }
                Ok(PathBufMessage::Finished) | Err(_) => {
                    self.send_finished_path
                        .send(PathBufMessage::Finished)
                        .unwrap_or(());
                    return;
                }
            }
        }
    }
}

/// Repsonses from the prompt_for_deletion() return this enum.
enum PromptResponse {
    All,    // Delete all subsequent entries.
    Delete, // Delete the current entry.
    Skip,   // Skip the current entry.
    Quit,   // Quit and delete nothing.
}

/// Read input from stdin for whether or not we should delete the current path.
fn prompt_for_deletion(pathbuf: &std::path::Path) -> PromptResponse {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut buffer = String::new();
    let answer;

    loop {
        let path_string = pathbuf.to_string_lossy();
        let path_basename = match pathbuf.file_name() {
            Some(stem) => stem.to_string_lossy(),
            None => continue,
        };

        println!();
        // # <path>
        println!("{} {}", Color::cyan("#"), Color::blue(path_string).bold());
        // # Delete the "xyz" repository?
        println!(
            "{}",
            Color::yellow(format!("Delete the \"{}\" repository?", path_basename)),
        );
        // # "all" deletes "..." and all subsequent repositories.
        println!(
            "{}: \"{}\" deletes \"{}\" and {} subsequent repositories!",
            Color::red("WARNING").bold(),
            Color::yellow("all"),
            path_basename,
            Color::red("ALL").bold(),
        );
        // # (yes, no, all, quit) [y,n,a,q]?
        print!(
            "Choices: {}, {}, {}, {} [{},{},{},{}]? ",
            Color::blue("yes"),
            Color::blue("no"),
            Color::yellow("all"),
            Color::green("quit"),
            Color::blue("y"),
            Color::blue("n"),
            Color::yellow("all"),
            Color::green("q"),
        );

        stdout.flush().unwrap_or(());

        buffer.clear();
        if stdin.read_line(&mut buffer).is_ok() {
            match buffer.trim().to_lowercase().as_str() {
                // "all" is dangerous so it has no shorthand aliases.
                "all" => {
                    answer = PromptResponse::All;
                    println!();
                    break;
                }
                "y" | "yes" => {
                    answer = PromptResponse::Delete;
                    break;
                }
                "n" | "no" | "s" | "skip" => {
                    answer = PromptResponse::Skip;
                    break;
                }
                "q" | "quit" => {
                    answer = PromptResponse::Quit;
                    println!();
                    break;
                }
                _ => {
                    println!();
                }
            }
        }
    }

    answer
}

struct PromptUser {
    recv_repo_path: crossbeam::channel::Receiver<PathBufMessage>,
    send_remove_path: crossbeam::channel::Sender<PathBufMessage>,
    recv_finished_path: crossbeam::channel::Receiver<PathBufMessage>,
    no_prompt: bool,
    quit: bool,
}

impl PromptUser {
    fn prompt_for_deletion(&mut self) {
        loop {
            match self.recv_repo_path.recv() {
                Ok(PathBufMessage::Path(pathbuf)) => {
                    if !self.quit {
                        self.prompt_pathbuf_for_deletion(pathbuf);
                    }
                }
                Ok(PathBufMessage::Finished) | Err(_) => {
                    self.send_remove_path
                        .send(PathBufMessage::Finished)
                        .unwrap_or(());
                    break;
                }
            }

            if !self.no_prompt {
                self.display_finished_nonblocking();
            }
        }

        self.display_finished_blocking();
    }

    fn prompt_pathbuf_for_deletion(&mut self, pathbuf: std::path::PathBuf) {
        if self.no_prompt {
            self.send_remove_path
                .send(PathBufMessage::Path(pathbuf))
                .unwrap_or(());
            return;
        }
        match prompt_for_deletion(&pathbuf) {
            PromptResponse::All => {
                self.no_prompt = true;
                self.send_remove_path
                    .send(PathBufMessage::Path(pathbuf))
                    .unwrap_or(());
            }
            PromptResponse::Delete => {
                self.send_remove_path
                    .send(PathBufMessage::Path(pathbuf))
                    .unwrap_or(());
            }
            PromptResponse::Skip => (),
            PromptResponse::Quit => {
                self.quit = true;
                self.send_remove_path
                    .send(PathBufMessage::Finished)
                    .unwrap_or(());
            }
        }
    }

    /// Display pending "Deleted" messages.
    fn display_finished_nonblocking(&self) {
        let mut printed = false;
        while let Ok(PathBufMessage::Path(pathbuf)) = self.recv_finished_path.try_recv() {
            if !printed {
                printed = true;
                println!();
            }
            print_deleted_pathbuf(&pathbuf);
        }
    }

    /// Block and display all of the remaining "Deleted" messages.
    fn display_finished_blocking(&self) {
        while let Ok(PathBufMessage::Path(pathbuf)) = self.recv_finished_path.recv() {
            print_deleted_pathbuf(&pathbuf);
        }
    }
}

/// Print a deleted path.
fn print_deleted_pathbuf(pathbuf: &std::path::Path) {
    println!(
        "{} {}: {}",
        Color::cyan("#"),
        Color::green("Deleted"),
        Color::blue(pathbuf.to_string_lossy()).bold(),
    );
}

/// Prune the garden config directory to remove trees that are no longer referenced
/// by the garden file. This can be run when branches or trees have been removed.
pub fn prune(
    config: &model::Configuration,
    options: &PruneOptions,
    paths: &[String],
) -> Result<i32> {
    let exit_status: i32 = 0;

    if !options.remove {
        let msg = "NOTE: Safe mode enabled. Repositories will not be deleted.";
        println!("{}", Color::green(msg));
        let msg = "Use '--rm' to enable deletion.";
        println!("{}", Color::green(msg));
    }

    // Initialize the global thread pool.
    rayon::ThreadPoolBuilder::new()
        .num_threads(options.num_jobs)
        .build_global()?;

    // Channels are used to exchange PathBufMessage messages.
    // These channels are used to emit repositories that are discovered through a
    // filesystem traversal, filter paths through user interaction, remove selected
    // paths from the filesytem and report removed paths to the user.
    //
    // The TraverseFilesystem task traverses the filesystem and sends paths to the
    // PromptUser task through the send_repo_path channels.
    //
    // The PromptUser task receives from the recv_repo_path channel and prompts
    // the user for each path. Paths that are marked for deletion are sent to the
    // send_remove_path channel.
    //
    // The RemovePaths task receives from te recv_remove_path and performs removals
    // from the filesystem. Paths that have finished deleting are sent to the
    // PromptUser task via the send_finished_path channel.
    //
    // The PromptUser task drains the recv_finished_path channel to report paths that
    // have been deleted.
    //
    // TraverseFilesystem.traverse()
    // -> TraverseFilesystem.send_repo_path
    // -> PromptUser.recv_repo_path -> prompts for deletion
    // -> PromptUser.send_remove_path
    // -> RemovePaths.recv_remove_path -> removes paths
    // -> RemovePaths.send_finished_path
    // -> PromptUser.recv_finished_path -> prints deletion mssages.
    let (send_repo_path, recv_repo_path) = crossbeam::channel::unbounded();
    let (send_remove_path, recv_remove_path) = crossbeam::channel::unbounded();
    let (send_finished_path, recv_finished_path) = crossbeam::channel::unbounded();

    // Existing trees are never removed. Create a HashSet containing all of the current
    // tree paths so that we can skip them while traversing.
    let mut configured_tree_paths = std::collections::HashSet::new();
    {
        for tree in &config.trees {
            if let Some(pathbuf) = tree.canonical_pathbuf() {
                configured_tree_paths.insert(pathbuf);
            }
        }
    }

    let root_path = config.root_path.to_path_buf();
    let path_filters: Vec<std::path::PathBuf> = paths
        .iter()
        .map(|value| config.relative_pathbuf(value))
        .collect();

    rayon::scope_fifo(|scope| {
        // Spawn tasks in reverse order. Receivers first, senders after.
        scope.spawn_fifo(|remove_scope| {
            // RemovePaths handles filesystem removals.
            let remove_paths = RemovePaths {
                recv_remove_path,
                send_finished_path,
                dry_run: !options.remove,
            };
            remove_paths.remove_paths(remove_scope);
        });
        scope.spawn_fifo(|_| {
            // PromptUser prompts for confirmation and forwards requests to RemovePaths.
            let quit = false;
            let mut prompt_user = PromptUser {
                recv_repo_path,
                send_remove_path,
                recv_finished_path,
                no_prompt: options.no_prompt,
                quit,
            };
            prompt_user.prompt_for_deletion();
        });
        scope.spawn_fifo(|_| {
            // TraverseFilesystem searches for Git repositories and sends their paths
            // into the pipeline for confirmation and removal.
            let traverse_filesystem = TraverseFilesystem {
                min_depth: options.min_depth,
                max_depth: options.max_depth,
                send_repo_path,
                root_path,
                path_filters: &path_filters,
                configured_tree_paths: &configured_tree_paths,
            };
            traverse_filesystem.traverse();
        });
    });

    Ok(exit_status)
}

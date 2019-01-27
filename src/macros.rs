/// Print a message to stderr with an "error: " prefix and terminate the process
///
/// Parameters:
/// - `args`: A `std::fmt::Arguments`
pub fn error(args: std::fmt::Arguments) {
    eprintln!("error: {}", args);
    std::process::exit(1);
}

/// Print a message to stderr with a "debug: " prefix
///
/// Parameters:
/// - `args`: A `std::fmt::Arguments`
pub fn debug(args: std::fmt::Arguments) {
    eprintln!("debug: {}", args);
}

/// Print a message to stderr with an "debug : " prefix
///
/// Parameters:
/// - `fmt`: A format string.
/// - `args*`: Format string arguments.
#[macro_export]
macro_rules! debug {
    ( $fmt:expr $(, $args:expr )* ) => (
        $crate::macros::debug(format_args!($fmt, $( $args ),*))
    );
}

/// Print a message to stderr with a "error: " prefix and terminate the process
///
/// Parameters:
/// - `fmt`: A format string.
/// - `args*`: Format string arguments.
#[macro_export]
macro_rules! error {
    ( $fmt:expr $(, $args:expr )* ) => (
        $crate::macros::error(format_args!($fmt, $( $args ),*))
    );
}

/// Unwrap an Option<T> and return the result; terminate if unwrappable.
/// This variant assumes a void function and returns.
///
/// Parameters:
/// - `expr`: An expression that results to an Option<T>.
/// - `message`: Error message format arguments.
#[macro_export]
macro_rules! unwrap_or_err {
    ($expr:expr $(, $message:expr )* ) => {
        match $expr {
            Ok(value) => value,
            Err(err) => {
                error!($( $message ),*, err);
                return;
            }
        }
    }
}

/// Unwrap an Option<T> and return the result; terminate if unwrappable.
/// This variant returns the specified value from the function on error.
///
/// Parameters:
/// - `expr`: An expression that results to an Option<T>.
/// - `retval`: The value to return from the current function on error.
/// - `message`: Error message format arguments.
#[macro_export]
macro_rules! unwrap_or_err_return {
    ($expr:expr, $retval:ident $(, $message:expr )* ) => {
        match $expr {
            Ok(value) => value,
            Err(err) => {
                error!($( $message ),*, err);
                return $retval;
            }
        }
    }
}

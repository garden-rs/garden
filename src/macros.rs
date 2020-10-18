/// Print a message to stderr with an "error: " prefix
///
/// Parameters:
/// - `args`: A `std::fmt::Arguments`
pub fn error(args: std::fmt::Arguments) {
    eprintln!("error: {}", args);
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
    ( $fmt:expr $(, $args:expr )* ) => {
        $crate::macros::error(format_args!($fmt, $( $args ),*));
        std::process::exit(1);
    }
}


/// Implement std::display::Display with a custom format
/// Parameters:
/// - `struct_name`: The struct to extend.
/// - `format`: The format string to use.
#[macro_export]
macro_rules! impl_display_fmt {
    ($struct_name:ident, $format:expr) => {
        impl std::fmt::Display for $struct_name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter)
            -> std::fmt::Result {
                return write!(formatter, $format, self);
            }
        }
    }
}


/// Implement std::display::Display with a pretty-print format
/// Parameters:
/// - `struct_name`: The struct to extend.
#[macro_export]
macro_rules! impl_display {
    ($x:ident) => {
        impl_display_fmt!($x, "{:#?}");
    }
}


/// Implement std::display::Display with a brief debug format
/// Parameters:
/// - `struct_name`: The struct to extend.
#[macro_export]
macro_rules! impl_display_brief {
    ($x:ident) => {
        impl_display_fmt!($x, "{:?}");
    }
}

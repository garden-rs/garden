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


/// Print a message to stderr with a "error: " prefix
///
/// Parameters:
/// - `fmt`: A format string.
/// - `args*`: Format string arguments.
#[macro_export]
macro_rules! errmsg {
    ( $fmt:expr $(, $args:expr )* ) => {
        $crate::macros::error(format_args!($fmt, $( $args ),*));
    }
}


/// Unwrap an Option<T> and return the result; terminate if unwrappable.
/// This variant assumes a void function and returns.
///
/// Parameters:
/// - `expr`: An expression that resolves to an Option<T>.
/// - `message`: Error message format arguments.
#[macro_export]
macro_rules! unwrap_or_err {
    ($expr:expr $(, $message:expr )* ) => {
        match $expr {
            Ok(value) => value,
            Err(err) => {
                error!($( $message ),*, err);
            }
        }
    }
}

/// Unwrap an Option<T> and return the result; terminate if unwrappable.
/// This variant returns the specified value from the function on error.
///
/// Parameters:
/// - `expr`: An expression that resolves to an Option<T>.
/// - `retval`: The value to return from the current function on error.
/// - `message`: Error message format arguments.
#[macro_export]
macro_rules! unwrap_or_err_return {
    ($expr:expr, $retval:expr $(, $message:expr )* ) => {
        match $expr {
            Ok(value) => value,
            Err(err) => {
                errmsg!($( $message ),*, err);
                return $retval;
            }
        }
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

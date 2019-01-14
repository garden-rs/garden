pub fn error(args: std::fmt::Arguments) {
    eprintln!("error: {}", args);
    std::process::exit(1);
}

pub fn debug(args: std::fmt::Arguments) {
    eprintln!("debug: {}", args);
}

#[macro_export]
macro_rules! debug {
    ( $fmt:expr $(, $args:expr )* ) => (
        $crate::macros::debug(format_args!($fmt, $( $args ),*))
    );
}

#[macro_export]
macro_rules! error {
    ( $fmt:expr $(, $args:expr )* ) => (
        $crate::macros::error(format_args!($fmt, $( $args ),*))
    );
}

#[macro_export]
macro_rules! unwrap_or_err (
    ($expr:expr $(, $err:expr )* ) => (
        match $expr {
            Ok(value) => value,
            Err(err) => {
                error!($( $err ),*, err);
                return;
            }
        }
    )
);

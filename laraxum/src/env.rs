/// Get environment variable.
///
/// # Panics
/// - Not present.
/// - Not unicode.
macro_rules! env_var {
    ($env_var:expr) => {
        match ::std::env::var($env_var) {
            ::core::result::Result::Ok(ok) => ok,
            ::core::result::Result::Err(::std::env::VarError::NotPresent) => {
                ::core::panic!(::core::concat!(
                    "environment variable \"",
                    $env_var,
                    "\" not found"
                ));
            }
            ::core::result::Result::Err(::std::env::VarError::NotUnicode(ref s)) => {
                ::core::panic!(
                    ::core::concat!(
                        "environment variable \"",
                        $env_var,
                        "\" was not valid unicode: {:?}"
                    ),
                    s
                );
            }
        }
    };
}
pub(crate) use env_var;
/// Get optional environment variable.
///
/// # Panics
/// - Not unicode.
macro_rules! env_var_opt {
    ($env_var:expr) => {
        match ::std::env::var($env_var) {
            ::core::result::Result::Ok(ok) => ::core::option::Option::Some(ok),
            ::core::result::Result::Err(::std::env::VarError::NotPresent) => {
                ::core::option::Option::None
            }
            ::core::result::Result::Err(::std::env::VarError::NotUnicode(ref s)) => {
                ::core::panic!(
                    ::core::concat!(
                        "environment variable \"",
                        $env_var,
                        "\" was not valid unicode: {:?}"
                    ),
                    s
                );
            }
        }
    };
}
pub(crate) use env_var_opt;
/// Get environment variable with default.
///
/// # Panics
/// - Not unicode.
macro_rules! env_var_default {
    ($env_var:expr, $default:expr) => {
        $crate::env::env_var_opt!($env_var)
            .map(::std::borrow::Cow::Owned)
            .unwrap_or(::std::borrow::Cow::Borrowed($default))
    };
}
pub(crate) use env_var_default;

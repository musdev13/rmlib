#[cfg(feature = "ffi")]
mod error;
#[cfg(feature = "ffi")]
pub mod exports;
#[cfg(feature = "ffi")]
mod runtime;
#[cfg(feature = "ffi")]
mod strings;

#[cfg(feature = "ffi")]
pub use error::rmlib_last_error;
#[cfg(feature = "ffi")]
pub use strings::rmlib_string_free;

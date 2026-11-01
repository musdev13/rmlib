#![cfg(feature = "ffi")]

#[path = "../common/mod.rs"]
mod common;
mod helpers;

mod auth;
mod config;
mod lib_info;
#[cfg(feature = "modrinth")]
mod modrinth;
#[cfg(feature = "profiles")]
mod profiles;
#[cfg(feature = "resman-mods")]
mod resman_mods;
#[cfg(feature = "resman-resourcepacks")]
mod resman_resourcepacks;
mod types;
mod version;

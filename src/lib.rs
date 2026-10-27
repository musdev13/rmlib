// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 3 of the License, or (at your option) any later version.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, see <https://www.gnu.org/licenses/>.

#[cfg(feature = "core")]
pub mod core;
#[cfg(feature = "ely")]
pub mod extra;
#[cfg(feature = "ffi")]
mod ffi;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const CLIENT_TOKEN: &str = "93301754-cd11-4a00-bd50-f69dffaf4b0b";
pub const CLIENT_ID: &str = "00000000402b5328";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleInfo {
    pub name: &'static str,
}

#[allow(clippy::vec_init_then_push)]
pub fn available_modules() -> Vec<ModuleInfo> {
    let mut out = Vec::new();

    #[cfg(feature = "core")]
    out.push(ModuleInfo { name: "core" });

    #[cfg(feature = "ely")]
    out.push(ModuleInfo { name: "ely" });

    out
}

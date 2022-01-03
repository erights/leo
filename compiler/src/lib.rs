// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

//! The compiler for Leo programs.
//!
//! The [`Compiler`] type compiles Leo programs into R1CS circuits.

#![allow(clippy::module_inception)]
#![allow(clippy::upper_case_acronyms)]
#![doc = include_str!("../README.md")]

pub mod accesses;
pub use accesses::*;

pub mod compiler;

pub mod console;
pub use console::*;

pub mod global;
pub use global::*;

pub mod expression;
pub use expression::*;

pub mod function;
pub use function::*;

pub mod output;
pub use output::*;

pub(crate) mod program;
pub(crate) use program::*;

pub mod statement;
pub use statement::*;

pub mod phase;
pub use phase::*;

pub mod phases;
pub use phases::*;

pub mod option;
pub use option::*;

#[cfg(test)]
mod test;

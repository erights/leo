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

use crate::{Expression, Node, StaticString};
use leo_span::Span;

use serde::{Deserialize, Serialize};
use std::fmt;

/// The arguments `args` passed to `console.log(args)` or `console.error(args)`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ConsoleArgs {
    /// The formatting string with `parameters` interpolated into it.
    pub string: StaticString,
    /// Parameters to interpolate in `string`.
    pub parameters: Vec<Expression>,
    /// The span from `(` to `)`.
    pub span: Span,
}

impl fmt::Display for ConsoleArgs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\"{}\", {}",
            self.string,
            self.parameters
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

crate::simple_node_impl!(ConsoleArgs);

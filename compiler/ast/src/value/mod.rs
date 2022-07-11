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

use crate::{GroupLiteral, Identifier, Literal, Type};

use leo_errors::{type_name, FlattenError, LeoError, Result, TypeCheckerError};
use leo_span::{Span, Symbol};

use indexmap::IndexMap;
use std::{
    fmt::Display,
    ops::{BitAnd, BitOr, BitXor, Not},
};

// TODO: Consider refactoring this module to use the console implementations from snarkVM.

// Macro for making implementing unary operations over appropriate types easier.
macro_rules! implement_const_unary {
    (
        @overflowing
        name: $name:ident,
        method: $method:ident,
        string: $str:expr,
        patterns: [$([$type:ident, $m_type:ty]),+]
    ) => {
        implement_const_unary!{
            name: $name,
            patterns: [$([
                t: $type,
                l: |l: $m_type, span| l.$method().ok_or_else(|| FlattenError::unary_overflow(l, $str, span))
            ]),+]
        }
    };

    (
        @non-overflowing
        name: $name:ident,
        method: $method:ident,
        patterns: [$([$type:ident, $m_type:ty]),+]
    ) => {
        implement_const_unary!{
            name: $name,
            patterns: [$([
                t: $type,
                l: |l: $m_type, _| -> Result<$m_type> { Ok(l.$method()) }
            ]),+]
        }
    };

    (
        name: $name:ident,
        patterns: [$([
            t: $type:ident,
            l: $logic:expr
        ]),+]
    ) => {
        pub(crate) fn $name(self, span: Span) -> Result<Self> {
            use Value::*;

            match self {
                $(
                    $type(v, _) => {
                        Ok($type($logic(v.into(), span)?, span))
                    },
                )+
                // Unreachable because type checking should have already caught this and errored out.
                s => unreachable!("Const operation not supported {}.{}()", type_name(&s), stringify!($name))
            }
        }
    };
}

// Macro for making implementing binary operations over appropriate types easier.
macro_rules! implement_const_binary {
    // for overflowing operations that can overflow
    (
        @overflowing
        name: $name:ident,
        method: $method:ident,
        string: $str:expr,
        patterns: [$(
            // lhs, rhs, out, method left, method right
            [$lhs:ident, [$($rhs:ident),+], $out:ident, $m_lhs:ty, $m_rhs:ty]
        ),+]
    ) => {
        implement_const_binary!{
            name: $name,
            patterns: [$([
                types: $lhs, [$($rhs),+], $out,
                logic: |l: $m_lhs, r: $m_rhs, t, span| l.$method(r).ok_or_else(|| FlattenError::binary_overflow(l, $str, r, t, span))
            ]),+]
        }
    };

    // for wrapping math operations
    (
        @non-overflowing
        name: $name:ident,
        method: $method:ident,
        patterns: [$(
            // lhs, rhs, out, method left, method right, method output
            [$lhs:ident, [$($rhs:ident),+], $out:ident, $m_lhs:ty, $m_rhs:ty]
        ),+]
    ) => {
        implement_const_binary!{
            name: $name,
            patterns: [$([
                types: $lhs, [$($rhs),+], $out,
                logic: |l: $m_lhs, r: $m_rhs, _, _| -> Result<$m_lhs> {Ok(l.$method(r))}
            ]),+]
        }
    };

    // for cmp operations
    (
        @cmp
        name: $name:ident,
        method: $method:ident,
        string: $str:expr,
        patterns: [$(
            // lhs, rhs, out, method left, method right, method output
            [$lhs:ident, [$($rhs:ident),+], $out:ident, $m_lhs:ty, $m_rhs:ty]
        ),+]
    ) => {
        implement_const_binary!{
            name: $name,
            patterns: [$([
                types: $lhs, [$($rhs),+], $out,
                logic: |l: $m_lhs, r: $m_rhs, _, _| -> Result<bool> {Ok(l.$method(&r))}
            ]),+]
        }
    };

    (
        name: $name:ident,
        patterns: [$([
            types: $lhs:ident, [$($rhs:ident),+], $out:ident,
            logic: $logic:expr
        ]),+]
    ) => {
        pub(crate) fn $name(self, other: Self, span: Span) -> Result<Self> {
            use Value::*;

            match (self, other) {
                $(
                    $(
                        ($lhs(types, _), $rhs(rhs, _)) => {
                            let rhs_type = type_name(&rhs);
                            let out = $logic(types, rhs.into(), rhs_type, span)?;
                            Ok($out(out, span))
                        },
                    )+
                )+
                // Unreachable because type checking should have already caught this and errored out.
                (s, o) => unreachable!("Const operation not supported {}.{}({})", type_name(&s), stringify!($name), type_name(&o))
            }
        }
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Input(Type, Identifier),
    Address(String, Span),
    Boolean(bool, Span),
    Circuit(Identifier, IndexMap<Symbol, Value>),
    Field(String, Span),
    Group(Box<GroupLiteral>),
    I8(i8, Span),
    I16(i16, Span),
    I32(i32, Span),
    I64(i64, Span),
    I128(i128, Span),
    U8(u8, Span),
    U16(u16, Span),
    U32(u32, Span),
    U64(u64, Span),
    U128(u128, Span),
    Scalar(String, Span),
    String(String, Span),
}

impl Value {
    pub(crate) fn is_supported_const_fold_type(&self) -> bool {
        use Value::*;
        matches!(
            self,
            Boolean(_, _)
                | I8(_, _)
                | I16(_, _)
                | I32(_, _)
                | I64(_, _)
                | I128(_, _)
                | U8(_, _)
                | U16(_, _)
                | U32(_, _)
                | U64(_, _)
                | U128(_, _)
        )
    }

    pub(crate) fn from_u128(type_: Type, value: u128, span: Span) -> Self {
        // Should never error since we converted from a start..stop and then back.
        match type_ {
            Type::U8 => Value::U8(value.try_into().unwrap(), span),
            Type::U16 => Value::U16(value.try_into().unwrap(), span),
            Type::U32 => Value::U32(value.try_into().unwrap(), span),
            Type::U64 => Value::U64(value.try_into().unwrap(), span),
            Type::U128 => Value::U128(value, span),
            Type::I8 => Value::I8(value.try_into().unwrap(), span),
            Type::I16 => Value::I16(value.try_into().unwrap(), span),
            Type::I32 => Value::I32(value.try_into().unwrap(), span),
            Type::I64 => Value::I64(value.try_into().unwrap(), span),
            Type::I128 => Value::I128(value.try_into().unwrap(), span),
            _ => unreachable!(),
        }
    }

    implement_const_unary!(
        @overflowing
        name: abs,
        method: checked_abs,
        string: "abs",
        patterns: [
            [I8, i8],
            [I16, i16],
            [I32, i32],
            [I64, i64],
            [I128, i128]
        ]
    );

    implement_const_unary!(
        @non-overflowing
        name: abs_wrapped,
        method: wrapping_abs,
        patterns: [
            [I8, i8],
            [I16, i16],
            [I32, i32],
            [I64, i64],
            [I128, i128]
        ]
    );

    implement_const_unary!(
        @overflowing
        name: neg,
        method: checked_neg,
        string: "neg",
        patterns: [
            // [Field, Field],
            // [Group, Group],
            [I8, i8],
            [I16, i16],
            [I32, i32],
            [I64, i64],
            [I128, i128]
        ]
    );

    implement_const_unary!(
        @non-overflowing
        name: not,
        method: not,
        patterns: [
            [Boolean, bool],
            [I8, i8],
            [I16, i16],
            [I32, i32],
            [I64, i64],
            [I128, i128],
            [U8, u8],
            [U16, u16],
            [U32, u32],
            [U64, u64],
            [U128, u128]
        ]
    );

    implement_const_binary!(
        @overflowing
        name: add,
        method: checked_add,
        string: "+",
        patterns: [
            // [Field, [Field], Field, _, _],
            // [Group, [Group], Group, _, _],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
            //[Scalar, [Scalar], Scalar, _, _],

        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: add_wrapped,
        method: wrapping_add,
        patterns: [
            // [Field, [Field], Field, _, _],
            // [Group, [Group], Group, _, _],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
            //[Scalar, [Scalar], Scalar, _, _],

        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: bitand,
        method: bitand,
        patterns: [
            [Boolean, [Boolean], Boolean, bool, bool],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
        ]
    );

    implement_const_binary!(
        @overflowing
        name: div,
        method: checked_div,
        string: "/",
        patterns: [
            // [Field, [Field], Field, _, _],
            // [Group, [Group], Group, _, _],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
            //[Scalar, [Scalar], Scalar, _, _],
        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: div_wrapped,
        method: wrapping_div,
        patterns: [
            // [Field, [Field], Field, _, _],
            // [Group, [Group], Group, _, _],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
            //[Scalar, [Scalar], Scalar, _, _],
        ]
    );

    implement_const_binary!(
        @cmp
        name: eq,
        method: eq,
        string: "==",
        patterns: [
            [Boolean, [Boolean], Boolean, bool, bool],
            [I8, [I8], Boolean, i8, i8],
            [I16, [I16], Boolean, i16, i16],
            [I32, [I32], Boolean, i32, i32],
            [I64, [I64], Boolean, i64, i64],
            [I128, [I128], Boolean, i128, i128],
            [U8, [U8], Boolean, u8, u8],
            [U16, [U16], Boolean, u16, u16],
            [U32, [U32], Boolean, u32, u32],
            [U64, [U64], Boolean, u64, u64],
            [U128, [U128], Boolean, u128, u128]
        ]
    );

    implement_const_binary!(
        @cmp
        name: ge,
        method: ge,
        string: ">=",
        patterns: [
            [I8, [I8], Boolean, i8, i8],
            [I16, [I16], Boolean, i16, i16],
            [I32, [I32], Boolean, i32, i32],
            [I64, [I64], Boolean, i64, i64],
            [I128, [I128], Boolean, i128, i128],
            [U8, [U8], Boolean, u8, u8],
            [U16, [U16], Boolean, u16, u16],
            [U32, [U32], Boolean, u32, u32],
            [U64, [U64], Boolean, u64, u64],
            [U128, [U128], Boolean, u128, u128]
        ]
    );

    implement_const_binary!(
        @cmp
        name: gt,
        method: gt,
        string: ">",
        patterns: [
            [I8, [I8], Boolean, i8, i8],
            [I16, [I16], Boolean, i16, i16],
            [I32, [I32], Boolean, i32, i32],
            [I64, [I64], Boolean, i64, i64],
            [I128, [I128], Boolean, i128, i128],
            [U8, [U8], Boolean, u8, u8],
            [U16, [U16], Boolean, u16, u16],
            [U32, [U32], Boolean, u32, u32],
            [U64, [U64], Boolean, u64, u64],
            [U128, [U128], Boolean, u128, u128]
        ]
    );

    implement_const_binary!(
        @cmp
        name: le,
        method: le,
        string: "<=",
        patterns: [
            [I8, [I8], Boolean, i8, i8],
            [I16, [I16], Boolean, i16, i16],
            [I32, [I32], Boolean, i32, i32],
            [I64, [I64], Boolean, i64, i64],
            [I128, [I128], Boolean, i128, i128],
            [U8, [U8], Boolean, u8, u8],
            [U16, [U16], Boolean, u16, u16],
            [U32, [U32], Boolean, u32, u32],
            [U64, [U64], Boolean, u64, u64],
            [U128, [U128], Boolean, u128, u128]
        ]
    );

    implement_const_binary!(
        @cmp
        name: lt,
        method: lt,
        string: "<",
        patterns: [
            [I8, [I8], Boolean, i8, i8],
            [I16, [I16], Boolean, i16, i16],
            [I32, [I32], Boolean, i32, i32],
            [I64, [I64], Boolean, i64, i64],
            [I128, [I128], Boolean, i128, i128],
            [U8, [U8], Boolean, u8, u8],
            [U16, [U16], Boolean, u16, u16],
            [U32, [U32], Boolean, u32, u32],
            [U64, [U64], Boolean, u64, u64],
            [U128, [U128], Boolean, u128, u128]
        ]
    );

    implement_const_binary!(
        @overflowing
        name: mul,
        method: checked_mul,
        string: "*",
        patterns: [
            // [Field, [Field], Field, _, _],
            // [Group, [Group], Group, _, _],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
            //[Scalar, [Scalar], Scalar, _, _],
        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: mul_wrapped,
        method: wrapping_mul,
        patterns: [
            // [Field, [Field], Field, _, _],
            // [Group, [Group], Group, _, _],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
            //[Scalar, [Scalar], Scalar, _, _],
        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: bitor,
        method: bitor,
        patterns: [
            [Boolean, [Boolean], Boolean, bool, bool],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
        ]
    );

    implement_const_binary!(
        @overflowing
        name: pow,
        method: checked_pow,
        string: "**",
        patterns: [
            [I8, [U8, U16, U32], I8, i8, u32],
            [I16, [U8, U16, U32], I16, i16, u32],
            [I32, [U8, U16, U32], I32, i32, u32],
            [I64, [U8, U16, U32], I64, i64, u32],
            [I128, [U8, U16, U32], I128, i128, u32],
            [U8, [U8, U16, U32], U8, u8, u32],
            [U16, [U8, U16, U32], U16, u16, u32],
            [U32, [U8, U16, U32], U32, u32, u32],
            [U64, [U8, U16, U32], U64, u64, u32],
            [U128, [U8, U16, U32], U128, u128, u32]
        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: pow_wrapped,
        method: wrapping_pow,
        patterns: [
            [I8, [U8, U16, U32], I8, i8, u32],
            [I16, [U8, U16, U32], I16, i16, u32],
            [I32, [U8, U16, U32], I32, i32, u32],
            [I64, [U8, U16, U32], I64, i64, u32],
            [I128, [U8, U16, U32], I128, i128, u32],
            [U8, [U8, U16, U32], U8, u8, u32],
            [U16, [U8, U16, U32], U16, u16, u32],
            [U32, [U8, U16, U32], U32, u32, u32],
            [U64, [U8, U16, U32], U64, u64, u32],
            [U128, [U8, U16, U32], U128, u128, u32]
        ]
    );

    implement_const_binary!(
        @overflowing
        name: shl,
        method: checked_shl,
        string: "<<",
        patterns: [
            [I8, [U8, U16, U32], I8, i8, u32],
            [I16, [U8, U16, U32], I16, i16, u32],
            [I32, [U8, U16, U32], I32, i32, u32],
            [I64, [U8, U16, U32], I64, i64, u32],
            [I128, [U8, U16, U32], I128, i128, u32],
            [U8, [U8, U16, U32], U8, u8, u32],
            [U16, [U8, U16, U32], U16, u16, u32],
            [U32, [U8, U16, U32], U32, u32, u32],
            [U64, [U8, U16, U32], U64, u64, u32],
            [U128, [U8, U16, U32], U128, u128, u32]
        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: shl_wrapped,
        method: wrapping_shl,
        patterns: [
            [I8, [U8, U16, U32], I8, i8, u32],
            [I16, [U8, U16, U32], I16, i16, u32],
            [I32, [U8, U16, U32], I32, i32, u32],
            [I64, [U8, U16, U32], I64, i64, u32],
            [I128, [U8, U16, U32], I128, i128, u32],
            [U8, [U8, U16, U32], U8, u8, u32],
            [U16, [U8, U16, U32], U16, u16, u32],
            [U32, [U8, U16, U32], U32, u32, u32],
            [U64, [U8, U16, U32], U64, u64, u32],
            [U128, [U8, U16, U32], U128, u128, u32]
        ]
    );

    implement_const_binary!(
        @overflowing
        name: shr,
        method: checked_shr,
        string: ">>",
        patterns: [
            [I8, [U8, U16, U32], I8, i8, u32],
            [I16, [U8, U16, U32], I16, i16, u32],
            [I32, [U8, U16, U32], I32, i32, u32],
            [I64, [U8, U16, U32], I64, i64, u32],
            [I128, [U8, U16, U32], I128, i128, u32],
            [U8, [U8, U16, U32], U8, u8, u32],
            [U16, [U8, U16, U32], U16, u16, u32],
            [U32, [U8, U16, U32], U32, u32, u32],
            [U64, [U8, U16, U32], U64, u64, u32],
            [U128, [U8, U16, U32], U128, u128, u32]
        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: shr_wrapped,
        method: wrapping_shr,
        patterns: [
            [I8, [U8, U16, U32], I8, i8, u32],
            [I16, [U8, U16, U32], I16, i16, u32],
            [I32, [U8, U16, U32], I32, i32, u32],
            [I64, [U8, U16, U32], I64, i64, u32],
            [I128, [U8, U16, U32], I128, i128, u32],
            [U8, [U8, U16, U32], U8, u8, u32],
            [U16, [U8, U16, U32], U16, u16, u32],
            [U32, [U8, U16, U32], U32, u32, u32],
            [U64, [U8, U16, U32], U64, u64, u32],
            [U128, [U8, U16, U32], U128, u128, u32]
        ]
    );

    implement_const_binary!(
        @overflowing
        name: sub,
        method: checked_sub,
        string: "-",
        patterns: [
            // [Field, [Field], Field, _, _],
            // [Group, [Group], Group, _, _],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
            //[Scalar, [Scalar], Scalar, _, _],
        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: sub_wrapped,
        method: wrapping_sub,
        patterns: [
            // [Field, [Field], Field, _, _],
            // [Group, [Group], Group, _, _],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
            //[Scalar, [Scalar], Scalar, _, _],
        ]
    );

    implement_const_binary!(
        @non-overflowing
        name: xor,
        method: bitxor,
        patterns: [
            [Boolean, [Boolean], Boolean, bool, bool],
            [I8, [I8], I8, i8, i8],
            [I16, [I16], I16, i16, i16],
            [I32, [I32], I32, i32, i32],
            [I64, [I64], I64, i64, i64],
            [I128, [I128], I128, i128, i128],
            [U8, [U8], U8, u8, u8],
            [U16, [U16], U16, u16, u16],
            [U32, [U32], U32, u32, u32],
            [U64, [U64], U64, u64, u64],
            [U128, [U128], U128, u128, u128]
        ]
    );
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        match self {
            Input(type_, ident) => write!(f, "input var {}: {type_}", ident.name),
            Address(val, _) => write!(f, "{val}"),
            Circuit(val, _) => write!(f, "{}", val.name),
            Boolean(val, _) => write!(f, "{val}"),
            Field(val, _) => write!(f, "{val}"),
            Group(val) => write!(f, "{val}"),
            I8(val, _) => write!(f, "{val}"),
            I16(val, _) => write!(f, "{val}"),
            I32(val, _) => write!(f, "{val}"),
            I64(val, _) => write!(f, "{val}"),
            I128(val, _) => write!(f, "{val}"),
            U8(val, _) => write!(f, "{val}"),
            U16(val, _) => write!(f, "{val}"),
            U32(val, _) => write!(f, "{val}"),
            U64(val, _) => write!(f, "{val}"),
            U128(val, _) => write!(f, "{val}"),
            Scalar(val, _) => write!(f, "{val}"),
            String(val, _) => write!(f, "{val}"),
        }
    }
}

impl TryFrom<Value> for u128 {
    type Error = LeoError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_ref().try_into()
    }
}

impl TryFrom<&Value> for u128 {
    type Error = LeoError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        use Value::*;
        match value {
            I8(val, span) => {
                u128::try_from(*val).map_err(|_| FlattenError::loop_has_neg_value(Type::from(value), *span).into())
            }
            I16(val, span) => {
                u128::try_from(*val).map_err(|_| FlattenError::loop_has_neg_value(Type::from(value), *span).into())
            }
            I32(val, span) => {
                u128::try_from(*val).map_err(|_| FlattenError::loop_has_neg_value(Type::from(value), *span).into())
            }
            I64(val, span) => {
                u128::try_from(*val).map_err(|_| FlattenError::loop_has_neg_value(Type::from(value), *span).into())
            }
            I128(val, span) => {
                u128::try_from(*val).map_err(|_| FlattenError::loop_has_neg_value(Type::from(value), *span).into())
            }
            U8(val, _) => Ok(*val as u128),
            U16(val, _) => Ok(*val as u128),
            U32(val, _) => Ok(*val as u128),
            U64(val, _) => Ok(*val as u128),
            U128(val, _) => Ok(*val),
            _ => unreachable!(),
        }
    }
}

impl AsRef<Value> for Value {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl From<Value> for Type {
    fn from(v: Value) -> Self {
        v.as_ref().into()
    }
}

impl From<&Value> for Type {
    fn from(v: &Value) -> Self {
        use Value::*;
        match v {
            Input(type_, _) => *type_,
            Address(_, _) => Type::Address,
            Boolean(_, _) => Type::Boolean,
            Circuit(ident, _) => Type::Identifier(*ident),
            Field(_, _) => Type::Field,
            Group(_) => Type::Group,
            I8(_, _) => Type::I8,
            I16(_, _) => Type::I16,
            I32(_, _) => Type::I32,
            I64(_, _) => Type::I64,
            I128(_, _) => Type::I128,
            U8(_, _) => Type::U8,
            U16(_, _) => Type::U16,
            U32(_, _) => Type::U32,
            U64(_, _) => Type::U64,
            U128(_, _) => Type::U128,
            Scalar(_, _) => Type::Scalar,
            String(_, _) => Type::String,
        }
    }
}

// TODO: Consider making this `Option<Value>` instead of `Value`.
impl From<&Literal> for Value {
    /// Converts a literal to a value.
    /// This should only be invoked on literals that are known to be valid.
    fn from(literal: &Literal) -> Self {
        match literal {
            Literal::Address(string, span) => Self::Address(string.clone(), span.clone()),
            Literal::Boolean(bool, span) => Self::Boolean(bool.clone(), span.clone()),
            Literal::Field(string, span) => Self::Field(string.clone(), span.clone()),
            Literal::Group(group_literal) => Self::Group(group_literal.clone()),
            Literal::Scalar(string, span) => Self::Scalar(string.clone(), span.clone()),
            Literal::String(string, span) => Self::String(string.clone(), span.clone()),
            Literal::I8(string, span) => Self::I8(string.parse::<i8>().unwrap(), span.clone()),
            Literal::I16(string, span) => Self::I16(string.parse::<i16>().unwrap(), span.clone()),
            Literal::I32(string, span) => Self::I32(string.parse::<i32>().unwrap(), span.clone()),
            Literal::I64(string, span) => Self::I64(string.parse::<i64>().unwrap(), span.clone()),
            Literal::I128(string, span) => Self::I128(string.parse::<i128>().unwrap(), span.clone()),
            Literal::U8(string, span) => Self::U8(string.parse::<u8>().unwrap(), span.clone()),
            Literal::U16(string, span) => Self::U16(string.parse::<u16>().unwrap(), span.clone()),
            Literal::U32(string, span) => Self::U32(string.parse::<u32>().unwrap(), span.clone()),
            Literal::U64(string, span) => Self::U64(string.parse::<u64>().unwrap(), span.clone()),
            Literal::U128(string, span) => Self::U128(string.parse::<u128>().unwrap(), span.clone()),
        }
    }
}

// impl From<Value> for Literal {
//     fn from(v: Value) -> Self {
//         use Value::*;
//         match v {
//             Input(_, _) => panic!("We need to test if this is hittable"),
//             Address(v, span) => Literal::Address(v, span),
//             Boolean(v, span) => Literal::Boolean(v, span),
//             Circuit(ident, values) => {
//                 Literal::Circuit(ident, values.into_iter().map(|(n, v)| (n, v.into())).collect())
//             }
//             Field(v, span) => Literal::Field(v, span),
//             Group(v) => Literal::Group(v),
//             I8(v, span) => Literal::I8(v.to_string(), span),
//             I16(v, span) => Literal::I16(v.to_string(), span),
//             I32(v, span) => Literal::I32(v.to_string(), span),
//             I64(v, span) => Literal::I64(v.to_string(), span),
//             I128(v, span) => Literal::I128(v.to_string(), span),
//             U8(v, span) => Literal::U8(v.to_string(), span),
//             U16(v, span) => Literal::U16(v.to_string(), span),
//             U32(v, span) => Literal::U32(v.to_string(), span),
//             U64(v, span) => Literal::U64(v.to_string(), span),
//             U128(v, span) => Literal::U128(v.to_string(), span),
//             Scalar(v, span) => Literal::Scalar(v, span),
//             String(v, span) => Literal::String(v, span),
//         }
//     }
// }
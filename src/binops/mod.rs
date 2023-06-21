// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

// Implements binary operators "&T op U", "T op &U", "&T op &U"
// based on "T op U" where T and U are f256
macro_rules! forward_ref_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a> $imp<f256> for &'a f256 {
            type Output = <f256 as $imp<f256>>::Output;

            #[inline(always)]
            fn $method(self, rhs: f256) -> Self::Output {
                $imp::$method(*self, rhs)
            }
        }
        impl $imp<&f256> for f256 {
            type Output = <f256 as $imp<f256>>::Output;

            #[inline(always)]
            fn $method(self, rhs: &f256) -> Self::Output {
                $imp::$method(self, *rhs)
            }
        }
        impl $imp<&f256> for &f256 {
            type Output = <f256 as $imp<f256>>::Output;

            #[inline(always)]
            fn $method(self, rhs: &f256) -> Self::Output {
                $imp::$method(*self, *rhs)
            }
        }
    };
}

macro_rules! forward_op_assign {
    (impl $imp:ident, $method:ident, $base_imp:ident, $base_method:ident) => {
        impl<T> $imp<T> for f256
        where
            f256: $base_imp<T, Output = Self>,
        {
            #[inline(always)]
            fn $method(&mut self, rhs: T) {
                *self = $base_imp::$base_method(*self, rhs);
            }
        }
    };
}

mod add;
mod cmp;
mod div;
pub(crate) mod mul;
mod rem;

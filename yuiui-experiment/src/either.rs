#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn as_ref(&self) -> Either<&L, &R> {
        match self {
            Either::Left(value) => Either::Left(value),
            Either::Right(value) => Either::Right(value),
        }
    }

    pub fn as_mut(&mut self) -> Either<&mut L, &mut R> {
        match self {
            Either::Left(value) => Either::Left(value),
            Either::Right(value) => Either::Right(value),
        }
    }
}

#[macro_export]
macro_rules! either {
    (if $($rest:tt)*) => { either_internal!(@if [] $($rest)*) };
    (match $($rest:tt)*) => { either_internal!(@match [] $($rest)*) };
}

#[macro_export]
macro_rules! either_internal {
    (@if [$($predicate:tt)*] { $($then:tt)* } else { $($else:tt)* }) => {
        if $($predicate)* {
            crate::either::Either::Left($($then)*)
        } else {
            crate::either::Either::Right($($else)*)
        }
    };
    (@if [$($predicate:tt)*] { $($then:tt)* } else if $($rest:tt)*) => {
        if $($predicate)* {
            crate::either::Either::Left($($then)*)
        } else {
            crate::either::Either::Right(either_internal!(@if [] $($rest)*))
        }
    };
    (@if [$($predicate:tt)*] $head:tt $($rest:tt)*) => {
        either_internal!(@if [$($predicate)* $head] $($rest)*)
    };
    (@match [$($scrutinee:tt)*] { $($rest:tt)* }) => {
        either_internal!(@match_normalize [$($scrutinee)*] [] $($rest)*)
    };
    (@match [$($scrutinee:tt)*] $head:tt $($rest:tt)*) => {
        either_internal!(@match [$($scrutinee)* $head] $($rest)*)
    };
    (@match_first [$($scrutinee:tt)*] [] [$($arms:tt)*] $match:pat => $body:block) => {
        either_internal!(
            @match_next
            [$($scrutinee)*]
            []
            [$($arms)* $match => $body]
        )
    };
    (@match_first [$($scrutinee:tt)*] [] [$($arms:tt)*] $match:pat => $body:block $($rest:tt)*) => {
        either_internal!(
            @match_next
            [$($scrutinee)*]
            [crate::either::Either::Right]
            [$($arms)* $match => crate::either::Either::Left($body)]
            $($rest)*
        )
    };
    (@match_call [$($scrutinee:tt)*] [$($context:tt)*] [$($arms:tt)*] [] [$match:pat => $body:expr] $($rest:tt)*) => {
        either_internal!(
            @match_next
            [$($scrutinee)*]
            [$($context)*]
            [$($arms)* $match => $body]
            $($rest)*
        )
    };
    (@match_call [$($scrutinee:tt)*] [$($context:tt)*] [$($arms:tt)*] [$f:path $(, $fs:path)* $(,)?] [$match:pat => $body:expr] $($rest:tt)*) => {
        either_internal!(
            @match_call
            [$($scrutinee)*]
            [$($context)*]
            [$($arms)*]
            [$($fs,)*]
            [$match => $f($body)]
            $($rest)*
        )
    };
    (@match_next [$($scrutinee:tt)*] [$($context:tt)*] [$($match:pat => $body:expr)*]) => {
        match $($scrutinee)* {
            $($match => $body,)*
        }
    };
    (@match_next [$($scrutinee:tt)*] [] [$($arms:tt)*] $match:pat => $body:block) => {
        // last
        either_internal!(
            @match_build
            [$($scrutinee)*]
            []
            [$($arms)* $match => crate::either::Either::Left($body)]
        )
    };
    (@match_next [$($scrutinee:tt)*] [$($context:tt)*] [$($arms:tt)*] $match:pat => $body:block) => {
        // last
        either_internal!(
            @match_call
            [$($scrutinee)*]
            [crate::either::Either::Right, $($context)*]
            [$($arms)*]
            [$($context)*]
            [$match => $body]
        )
    };
    (@match_next [$($scrutinee:tt)*] [$($context:tt)*] [$($arms:tt)*] $match:pat => $body:block $($rest:tt)*) => {
        either_internal!(
            @match_call
            [$($scrutinee)*]
            [crate::either::Either::Right, $($context)*]
            [$($arms)*]
            [$($context)*]
            [$match => crate::either::Either::Left($body)]
            $($rest)*
        )
    };
    (@match_next [$($scrutinee:tt)*] [] [$($arms:tt)*] $match:pat => $body:block $($rest:tt)*) => {
        either_internal!(
            @match_next
            [$($scrutinee)*]
            []
            [$($arms)* $match => crate::either::Either::Right($body)]
            $($rest)*
        )
    };
    (@match_normalize [$($scrutinee:tt)*] [$($arms:tt)*]) => {
        either_internal!(
            @match_first
            [$($scrutinee)*]
            []
            []
            $($arms)*
        )
    };
    (@match_normalize [$($scrutinee:tt)*] [$($arms:tt)*] $match:pat => $body:block, $($rest:tt)*) => {
        either_internal!(
            @match_normalize
            [$($scrutinee)*]
            [$($arms)* $match => $body]
            $($rest)*
        )
    };
    (@match_normalize [$($scrutinee:tt)*] [$($arms:tt)*] $match:pat => $body:block $($rest:tt)*) => {
        either_internal!(
            @match_normalize
            [$($scrutinee)*]
            [$($arms)* $match => $body]
            $($rest)*
        )
    };
    (@match_normalize [$($scrutinee:tt)*] [$($arms:tt)*] $match:pat => $body:expr $(,)?) => {
        either_internal!(
            @match_normalize
            [$($scrutinee)*]
            [$($arms)* $match => { $body }]
        )
    };
    (@match_normalize [$($scrutinee:tt)*] [$($arms:tt)*] $match:pat => $body:expr, $($rest:tt)*) => {
        either_internal!(
            @match_normalize
            [$($scrutinee)*]
            [$($arms)* $match => { $body }]
            $($rest)*
        )
    };
}

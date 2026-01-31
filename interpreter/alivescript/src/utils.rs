use std::sync::Arc;

pub trait Invert<T, E> {
    type Output;
    fn invert(self) -> Self::Output;
}

impl<T, E> Invert<T, E> for Option<Result<T, E>> {
    type Output = Result<Option<T>, E>;

    fn invert(self: Self) -> Self::Output {
        self.map_or(Ok(None), |v| v.map(Some))
    }
}

impl<T, E> Invert<T, E> for Result<Option<T>, E> {
    type Output = Option<Result<T, E>>;

    fn invert(self: Self) -> Self::Output {
        self.map_or_else(|e| Some(Err(e)), |v| v.map(Ok))
    }
}

pub trait WrapWhere<T> {
    fn wrap_where<F>(self, predicate: F) -> Option<T>
    where
        F: FnOnce(&T) -> bool;
}

impl<T> WrapWhere<T> for T {
    fn wrap_where<F>(self, predicate: F) -> Option<T>
    where
        F: FnOnce(&T) -> bool,
    {
        if predicate(&self) { Some(self) } else { None }
    }
}

pub trait MapIf<T> {
    fn map_if<F>(self, maybe_map: F) -> T
    where
        F: FnOnce(&T) -> Option<T>;
}

impl<T> MapIf<T> for T {
    fn map_if<F>(self, maybe_map: F) -> T
    where
        F: FnOnce(&T) -> Option<T>,
    {
        maybe_map(&self).unwrap_or(self)
    }
}

pub trait Apply<T> {
    fn apply<F>(self, func: F) -> T
    where
        F: FnOnce(T) -> T;
}

impl<T> Apply<T> for T {
    fn apply<F>(self, func: F) -> T
    where
        F: FnOnce(T) -> T,
    {
        func(self)
    }
}

pub type ROString = Arc<str>;

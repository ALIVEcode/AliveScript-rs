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

use core::ops::Deref;

pub enum DerefEither<T, Left, Right>
where
    Left: Deref<Target = T>,
    Right: Deref<Target = T>,
{
    Left(Left),
    Right(Right),
}

impl<T, Left, Right> Deref for DerefEither<T, Left, Right>
where
    Left: Deref<Target = T>,
    Right: Deref<Target = T>,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Left(left) => left,
            Self::Right(right) => right,
        }
    }
}

impl<T, Left, Right> quote::ToTokens for DerefEither<T, Left, Right>
where
    T: quote::ToTokens,
    Left: Deref<Target = T>,
    Right: Deref<Target = T>,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.deref().to_tokens(tokens);
    }
    fn into_token_stream(self) -> proc_macro2::TokenStream
    where
        Self: Sized,
    {
        self.deref().into_token_stream()
    }
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        self.deref().to_token_stream()
    }
}

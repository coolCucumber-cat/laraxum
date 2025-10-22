use core::ops::Deref;

/// Like [Cow<T>][std::borrow::Cow<T>] but only for reading from, not writing into.
/// Uses [Box] to own instead of bare type.
pub enum CowBoxDeref<'a, T> {
    Borrowed(&'a T),
    Owned(Box<T>),
}

impl<'a, T> Deref for CowBoxDeref<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Borrowed(borrowed) => borrowed,
            Self::Owned(ref owned) => &**owned,
        }
    }
}

impl<'a, T> quote::ToTokens for CowBoxDeref<'a, T>
where
    T: quote::ToTokens,
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

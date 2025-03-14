use syn::parse::{ParseBuffer, ParseStream};

pub fn parse_curly_brackets(input: ParseStream) -> syn::Result<ParseBuffer> {
    Ok(syn::__private::parse_braces(input)?.content)
}
// pub fn parse_square_brackets(input: ParseStream) -> syn::Result<ParseBuffer> {
//     Ok(syn::__private::parse_brackets(input)?.content)
// }
// pub fn parse_round_brackets(input: ParseStream) -> syn::Result<ParseBuffer> {
//     Ok(syn::__private::parse_parens(input)?.content)
// }

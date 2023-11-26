use proc_macro::TokenStream;

mod distributor;
mod syntax_macro;

#[proc_macro_attribute]
pub fn create_syntax(attr: TokenStream, input: TokenStream) -> TokenStream {
    syntax_macro::arg_producing(attr, input)
}

#[proc_macro]
pub fn create_distributor(input: TokenStream) -> TokenStream {
    distributor::create_distributor(input)
}

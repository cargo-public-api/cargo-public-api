extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn simple_proc_macro(_item: TokenStream) -> TokenStream {
    "struct FromSimpleProcMacro".parse().unwrap()
}

#[proc_macro_derive(SimpleDeriveMacro, attributes(first_attr, second_attr))]
pub fn derive_error(_input: TokenStream) -> TokenStream {
    "struct FromSimpleDeriveMacro".parse().unwrap()
}

#[proc_macro_attribute]
pub fn simple_proc_macro_attribute(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    "struct FromSimpleProcMacroAttribute".parse().unwrap()
}

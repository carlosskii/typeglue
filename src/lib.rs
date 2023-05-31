extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use syn::{DeriveInput, Data, DataStruct, Fields};

mod glue;


#[proc_macro_error]
#[proc_macro_derive(Glue)]
pub fn derive_glue(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse_macro_input!(input);

    match ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(_), .. }) => {
            glue::struct_named_impl(ast)
        },
        Data::Struct(DataStruct { fields: Fields::Unnamed(_), .. }) => {
            glue::struct_unnamed_impl(ast)
        },
        Data::Struct(DataStruct { fields: Fields::Unit, .. }) => {
            abort!(ast, "Glue cannot be derived for unit structs")
        },
        Data::Enum(_) => {
            glue::enum_impl(ast)
        },
        Data::Union(_) => {
            abort!(ast, "Glue can only be derived for structs")
        }
    }
}
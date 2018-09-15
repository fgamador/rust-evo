extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

// TODO upgrade syn: https://crates.io/crates/syn

use proc_macro::TokenStream;

#[proc_macro_derive(HasLocalEnvironment)]
pub fn has_local_environment_derive(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_has_local_environment(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_has_local_environment(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;

    // TODO check for field of type LocalEnvironment
    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) =>
            fields.iter().filter_map(|f|
                match f.ty {
                    syn::Ty::Path(_, syn::Path { global: _, ref segments }) =>
                        match segments.last() {
//                          Some("LocalEnvironment".to_str()) => Some(&f.ident),
                            Some(syn::PathSegment { ident: _, parameters: _ }) => Some(&f.ident),
                            _ => None
                        },
                    _ => None
                }
            ).next(),
        _ => None // panic!("HasLocalEnvironment applied to non-struct")
    };

    quote! {
        impl HasLocalEnvironment for #name {
            fn environment(&self) -> &LocalEnvironment {
                &self.environment
            }

            fn environment_mut(&mut self) -> &mut LocalEnvironment {
                &mut self.environment
            }
        }
    }
}

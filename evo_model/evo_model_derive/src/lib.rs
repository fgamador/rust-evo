extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

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
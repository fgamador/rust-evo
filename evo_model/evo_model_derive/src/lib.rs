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
    //let type_name = syn::Ident::from("LocalEnvironment");
    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) =>
            fields.iter().filter_map(|f|
                match f.ty {
                    syn::Ty::Path(_, syn::Path { global: _, ref segments }) =>
                        match segments.last() {
//                          Some("LocalEnvironment".to_str()) => Some(&f.ident),
                            Some(syn::PathSegment { ident: type_name, parameters: _ }) => {
                                println!("{}", type_name);
                                Some(&f.ident)
                            }
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

fn field_is_struct_of_type(field: &syn::Field, type_name: &str) -> bool {
    match field.ty {
        syn::Ty::Path(_, syn::Path { global: _, ref segments }) =>
            match segments.last() {
                Some(syn::PathSegment { ref ident, parameters: _ }) => ident == type_name,
                _ => false
            },
        _ => false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_field_of_struct_type() {
        let field = field_of_struct_type("StructType");
        assert!(field_is_struct_of_type(&field, "StructType"));
//        let fields = get_fields_of_struct_type(body, "TypeName");
    }

    fn field_of_struct_type(type_name: &str) -> syn::Field {
        syn::Field {
            ident: None,
            vis: syn::Visibility::Public,
            attrs: Vec::new(),
            ty: syn::Ty::Path(None, syn::Path::from(type_name)),
        }
    }
}

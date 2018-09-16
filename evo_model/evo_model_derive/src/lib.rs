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

    let fields = get_fields_of_struct_type(&ast.body, "LocalEnvironment");
    if fields.len() != 1 {
        panic!("HasLocalEnvironment must be applied to a struct with exactly one field of type LocalEnvironment");
    }
    //let field_name = fields[0].ident.unwrap();

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

fn get_fields_of_struct_type<'a>(body: &'a syn::Body, type_name: &str) -> Vec<&'a syn::Field> {
    match body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) =>
            fields.iter().filter(|f| field_is_struct_of_type(f, type_name)).collect(),
        _ => vec![]
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
    fn gets_fields_of_struct_type() {
        let body = syn::Body::Struct(syn::VariantData::Struct(
            vec![field_of_struct_type("StructType")]));
        let fields = get_fields_of_struct_type(&body, "StructType");
        assert_eq!(1, fields.len());
    }

    #[test]
    fn identifies_field_of_struct_type() {
        let field = field_of_struct_type("StructType");
        assert!(field_is_struct_of_type(&field, "StructType"));
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

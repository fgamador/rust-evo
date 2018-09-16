extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

// TODO upgrade syn: https://crates.io/crates/syn

use proc_macro::TokenStream;

#[proc_macro_derive(GraphEdge)]
pub fn graph_edge_derive(input: TokenStream) -> TokenStream {
    trait_derive(input, |ast| {
        let name = &ast.ident;
        let field_name = get_field_name_of_struct_type(&ast.body, "GraphEdgeData");

        quote! {
            impl GraphEdge for #name {
                fn edge_handle(&self) -> EdgeHandle {
                    self.#field_name.edge_handle()
                }

                fn node1_handle(&self) -> NodeHandle {
                    self.#field_name.node1_handle()
                }

                fn node2_handle(&self) -> NodeHandle {
                    self.#field_name.node2_handle()
                }

                fn graph_edge_data(&self) -> &GraphEdgeData {
                    &self.#field_name
                }

                fn graph_edge_data_mut(&mut self) -> &mut GraphEdgeData {
                    &mut self.#field_name
                }
            }
        }
    })
}

#[proc_macro_derive(GraphNode)]
pub fn graph_node_derive(input: TokenStream) -> TokenStream {
    trait_derive(input, |ast| {
        let name = &ast.ident;
        let field_name = get_field_name_of_struct_type(&ast.body, "GraphNodeData");

        quote! {
            impl GraphNode for #name {
                fn node_handle(&self) -> NodeHandle {
                    self.#field_name.handle()
                }

                fn graph_node_data(&self) -> &GraphNodeData {
                    &self.#field_name
                }

                fn graph_node_data_mut(&mut self) -> &mut GraphNodeData {
                    &mut self.#field_name
                }
            }
        }
    })
}

#[proc_macro_derive(HasLocalEnvironment)]
pub fn has_local_environment_derive(input: TokenStream) -> TokenStream {
    trait_derive(input, |ast| {
        let name = &ast.ident;
        let field_name = get_field_name_of_struct_type(&ast.body, "LocalEnvironment");

        quote! {
            impl HasLocalEnvironment for #name {
                fn environment(&self) -> &LocalEnvironment {
                    &self.#field_name
                }

                fn environment_mut(&mut self) -> &mut LocalEnvironment {
                    &mut self.#field_name
                }
            }
        }
    })
}

#[proc_macro_derive(NewtonianBody)]
pub fn newtonian_body_derive(input: TokenStream) -> TokenStream {
    trait_derive(input, |ast| {
        let name = &ast.ident;
        let field_name = get_field_name_of_struct_type(&ast.body, "NewtonianState");

        quote! {
            impl NewtonianBody for #name {
                fn position(&self) -> Position {
                    self.#field_name.position()
                }

                fn velocity(&self) -> Velocity {
                    self.#field_name.velocity()
                }

                fn move_for(&mut self, duration: Duration) {
                    self.#field_name.move_for(duration);
                }

                fn kick(&mut self, impulse: Impulse) {
                    self.#field_name.kick(impulse);
                }

                fn forces(&self) -> &Forces {
                    self.#field_name.forces()
                }

                fn forces_mut(&mut self) -> &mut Forces {
                    self.#field_name.forces_mut()
                }

                fn exert_forces(&mut self, duration: Duration) {
                    self.#field_name.exert_forces(duration);
                }
            }
        }
    })
}

fn trait_derive<F>(input: TokenStream, impl_trait: F) -> TokenStream
    where F: Fn(&syn::DeriveInput) -> quote::Tokens
{
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_trait(&ast);
    gen.parse().unwrap()
}

fn get_field_name_of_struct_type<'a>(body: &'a syn::Body, type_name: &str) -> &'a syn::Ident {
    let fields = get_fields_of_struct_type(body, type_name);
    if fields.len() != 1 {
        panic!("Macro must be applied to a struct with exactly one field of type {}", type_name);
    }
    fields[0].ident.as_ref().unwrap()
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

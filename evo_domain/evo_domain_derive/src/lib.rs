use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(GraphEdge)]
pub fn graph_edge_derive(input: TokenStream) -> TokenStream {
    trait_derive(input, |ast| {
        let name = &ast.ident;
        let field_name = get_name_of_field_that_is_struct_of_type(&ast.data, "GraphEdgeData");

        quote! {
            impl GraphEdge for #name {
                fn edge_handle(&self) -> EdgeHandle {
                    self.#field_name.handle()
                }

                fn node1_handle(&self) -> NodeHandle {
                    self.#field_name.node1_handle()
                }

                fn node2_handle(&self) -> NodeHandle {
                    self.#field_name.node2_handle()
                }

                fn other_node_handle(&self, node_handle: NodeHandle) -> NodeHandle {
                    self.#field_name.other_node_handle(node_handle)
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

#[proc_macro_derive(GraphMetaEdge)]
pub fn graph_meta_edge_derive(input: TokenStream) -> TokenStream {
    trait_derive(input, |ast| {
        let name = &ast.ident;
        let field_name = get_name_of_field_that_is_struct_of_type(&ast.data, "GraphMetaEdgeData");

        quote! {
            impl GraphMetaEdge for #name {
                fn edge1_handle(&self) -> EdgeHandle {
                    self.#field_name.edge1_handle()
                }

                fn edge2_handle(&self) -> EdgeHandle {
                    self.#field_name.edge2_handle()
                }

                fn graph_meta_edge_data(&self) -> &GraphMetaEdgeData {
                    &self.#field_name
                }

                fn graph_meta_edge_data_mut(&mut self) -> &mut GraphMetaEdgeData {
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
        let field_name = get_name_of_field_that_is_struct_of_type(&ast.data, "GraphNodeData");

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

                fn has_edge(&self, node_edge_index: usize) -> bool {
                    self.#field_name.has_edge_handle(node_edge_index)
                }

                fn edge_handle(&self, node_edge_index: usize) -> EdgeHandle {
                    self.#field_name.edge_handle(node_edge_index)
                }

                fn edge_handles(&self) -> &[Option<EdgeHandle>] {
                    self.#field_name.edge_handles()
                }
            }
        }
    })
}

#[proc_macro_derive(HasLocalEnvironment)]
pub fn has_local_environment_derive(input: TokenStream) -> TokenStream {
    trait_derive(input, |ast| {
        let name = &ast.ident;
        let field_name = get_name_of_field_that_is_struct_of_type(&ast.data, "LocalEnvironment");

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
        let field_name = get_name_of_field_that_is_struct_of_type(&ast.data, "NewtonianState");

        quote! {
            impl NewtonianBody for #name {
                fn mass(&self) -> Mass {
                    self.#field_name.mass()
                }

                fn position(&self) -> Position {
                    self.#field_name.position()
                }

                fn velocity(&self) -> Velocity {
                    self.#field_name.velocity()
                }

                fn move_for_one_tick(&mut self) {
                    self.#field_name.move_for_one_tick();
                }

                fn kick(&mut self, impulse: Impulse) {
                    self.#field_name.kick(impulse);
                }

                fn net_force(&self) -> &NetForce {
                    self.#field_name.net_force()
                }

                fn net_force_mut(&mut self) -> &mut NetForce {
                    self.#field_name.net_force_mut()
                }

                fn exert_net_force_for_one_tick(&mut self) {
                    self.#field_name.exert_net_force_for_one_tick();
                }
            }
        }
    })
}

fn trait_derive<F>(input: TokenStream, impl_trait: F) -> TokenStream
where
    F: Fn(&syn::DeriveInput) -> proc_macro2::TokenStream,
{
    let ast = parse_macro_input!(input as DeriveInput);
    let expanded = impl_trait(&ast);
    TokenStream::from(expanded)
}

fn get_name_of_field_that_is_struct_of_type<'a>(
    data: &'a syn::Data,
    type_name: &str,
) -> &'a syn::Ident {
    let fields = get_fields_that_are_structs_of_type(data, type_name);
    if fields.len() != 1 {
        panic!(
            "Macro must be applied to a struct with exactly one field of type {}",
            type_name
        );
    }
    fields[0].ident.as_ref().unwrap()
}

fn get_fields_that_are_structs_of_type<'a>(
    data: &'a syn::Data,
    type_name: &str,
) -> Vec<&'a syn::Field> {
    match data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .filter(|f| field_is_struct_of_type(f, type_name))
                .collect(),
            _ => vec![],
        },
        _ => vec![],
    }
}

fn field_is_struct_of_type(field: &syn::Field, type_name: &str) -> bool {
    match &field.ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => match segments.last() {
            Some(syn::PathSegment { ident, .. }) => ident == type_name,
            _ => false,
        },
        _ => false,
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn gets_fields_of_struct_type() {
//        let data = syn::Data::Struct(syn::DataStruct {
//            struct_token: syn::Token![struct],
//            fields: syn::Fields::Named(syn::FieldsNamed {
//                brace_token: syn::token::Brace(),
//                named: Default::default(),
//            }),
//            semi_token: None,
//        });
//        //vec![field_of_struct_type("StructType")]));
//        let fields = get_fields_of_struct_type(&data, "StructType");
//        assert_eq!(1, fields.len());
//    }
//
//    #[test]
//    fn identifies_field_of_struct_type() {
//        let field = field_of_struct_type("StructType");
//        assert!(field_is_struct_of_type(&field, "StructType"));
//    }
//
//    fn field_of_struct_type(type_name: &str) -> syn::Field {
//        syn::Field {
//            attrs: Vec::new(),
//            vis: syn::Visibility::Public(syn::Token![pub]),
//            ident: None,
//            colon_token: None,
//            ty: syn::Type::Path(syn::TypePath { qself: None, path: syn::Path { leading_colon: None, segments: {} } }),
//        }
//    }
//}

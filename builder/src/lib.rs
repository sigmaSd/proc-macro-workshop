use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::{format_ident, quote};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let command_struct_name = input.ident;
    let commandbuilder_struct_name = format_ident!("{}Builder",command_struct_name);


    let is_optional = |ftype: &syn::Type| {
        if let syn::Type::Path(ref p) = ftype {
            &p.path.segments[0].ident.to_string() == "Option"
        } else {
            false
        }
    };

    let get_inner_type = |ftype: &syn::Type| {
        if let syn::Type::Path(ref p) = ftype {
            if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
                if inner_ty.args.len() != 1 {
                    return None;
                }

                let inner_ty = inner_ty.args.first().unwrap();
                if let syn::GenericArgument::Type(ref t) = inner_ty {
                    return Some(t.clone());
                }
                unreachable!()
            } else {
                return None
            }

        } else {
            None
        }
    };


    let fields =  if let syn::Data::Struct(ds)  = input.data {
        ds.fields
    } else {
        unreachable!()
    };

    let fields_to_o = fields.iter().map(|f|{
        let name = &f.ident;
        let ftype = &f.ty;
        if is_optional(ftype) {
            quote!(#name: self.#name.clone())
        } else {
            quote!(#name: self.#name.clone().ok_or("field is required")?) 
        }
    });

    let fields_dec = fields.iter().map(|f|{
        let name = &f.ident;
        let ftype = &f.ty;
        if is_optional(ftype) {
            quote!(#name: #ftype) 
        } else {
            quote!(#name: Option<#ftype>) 
        }
    });

    let fields_inv = fields.iter().map(|f|{
        let name = &f.ident;
        quote!(#name: None) 
    });

    let fields_fn = fields.iter().map(|f|{
        let name = &f.ident;
        let ftype = &f.ty;
        if is_optional(ftype) {
            let inner_type = get_inner_type(ftype);
            quote!(fn #name(&mut self, #name: #inner_type) -> &mut Self {
                self.#name = Some(#name);
                self
            })
        } else {
            quote!(fn #name(&mut self, #name: #ftype) -> &mut Self {
                self.#name = Some(#name);
                self
            })    
        }
        
    });


    let expanded = quote!(
        pub struct #commandbuilder_struct_name {
                    #(#fields_dec,)*
                }
            
        impl #commandbuilder_struct_name {
            #(#fields_fn)*

            pub fn build(&mut self) -> Result<#command_struct_name, Box<dyn std::error::Error>> {
                Ok(
                    #command_struct_name {
                        #(#fields_to_o,)*
                    }
                )
                /*
                    let a struct {
                        none
                        none
                    }.to_or()
                */
                
            }
        }

/*fn executable(&mut self, executable: String) -> &mut Self {
//             self.executable = Some(executable);
//             self
//         } */


        impl #command_struct_name {
            pub fn builder() -> #commandbuilder_struct_name {
                #commandbuilder_struct_name {
                    #(#fields_inv,)*
                }
                
            }
        }
    );
    
    TokenStream::from(expanded)
}



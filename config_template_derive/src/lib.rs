use proc_macro2::Ident;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn;
use syn::parse_macro_input;
use syn::DeriveInput;
use syn::ExprLit;

#[proc_macro_derive(ConfigTemplate, attributes(default_value))]
pub fn derive_config_template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut template = String::with_capacity(8192);

    if let syn::Data::Struct(s) = input.data {
        if let syn::Fields::Named(fields_named) = s.fields {
            for f in fields_named.named {
                let mut default_value = Option::<String>::None;

                for a in f.attrs {
                    let attr_id = a.path().get_ident()
                        .map_or_else(|| "".to_owned(), |id| id.to_string());

                    if attr_id == "doc" {
                        if let syn::Meta::NameValue(name_value) = &a.meta {
                            if let syn::Expr::Lit(literal) = &name_value.value {
                                if let syn::Lit::Str(str) = &literal.lit {
                                    template.push_str("#");
                                    template.push_str(&str.value());
                                    template.push_str("\n");
                                }
                            }
                        }
                    }

                    if attr_id == "default_value" {
                        if let Ok(expr) = &a.parse_args::<ExprLit>() {
                            if let syn::Lit::Str(lit) = &expr.lit {
                                default_value = Some(lit.value());
                            }
                        }
                    }
                }

                let field_ident = f.ident.unwrap().to_string();

                match default_value {
                    Some(dv) => {
                        template.push_str(&field_ident);
                        template.push_str(": ");
                        template.push_str(&dv);
                        template.push_str("\n\n");
                    },
                    None => {
                        let err_msg = format!("#[default_value(...)] is missing at the field '{}'", field_ident);

                        return TokenStream::from(quote::quote! {
                            compile_error!(#err_msg);
                        });
                    },
                }
            }
        }
    }

    // for a in input.attrs {
    //     println!("  {}", a.path().get_ident().unwrap().to_string());
    // }
    
    let struct_ident = format!("{}{}", input.ident, "Template");
    let config_template_ident = Ident::new(&struct_ident, Span::call_site());

    let appending = quote::quote! {
        pub const #config_template_ident: &str = #template;
    };

    TokenStream::from(appending)
}

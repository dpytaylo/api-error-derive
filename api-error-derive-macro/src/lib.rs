use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Error, LitStr, Meta};

const INTERNAL_SERVER_ERROR_STR: &str = "InternalServerError";

fn api_error_validate(input: &DeriveInput) -> syn::Result<()> {
    if let syn::Data::Enum(val) = &input.data {
        for variant in &val.variants {
            let mut was_pass = false;
            let mut status_code = None;
            let mut custom = None;

            let variant_span = variant.span();
            for attr in &variant.attrs {
                match &attr.meta {
                    Meta::Path(path) => {
                        if path.is_ident("pass") {
                            was_pass = true;
                        }
                    }

                    Meta::List(list) => {
                        if list.path.is_ident("status_code") {
                            status_code = Some(
                                syn::parse2::<Ident>(list.tokens.clone()).expect("StatusCode"),
                            );
                        } else if list.path.is_ident("custom") {
                            custom = Some(
                                syn::parse2::<LitStr>(list.tokens.clone())
                                    .expect("A string literal")
                                    .value(),
                            );
                        }
                    }

                    _ => (),
                }

                if (status_code.is_some() || custom.is_some()) && was_pass {
                    return Err(Error::new(
                        variant_span,
                        "the `#[pass]` attribute is unnecessary here",
                    ));
                }
            }
        }
    } else {
        return Err(Error::new(input.span(), "excepted an enum"));
    }

    Ok(())
}

#[cfg(feature = "axum")]
fn axum_into_response(name: Ident) -> TokenStream {
    quote! {
        impl axum::response::IntoResponse for #name {
            fn into_response(self) -> axum::response::Response {
                let mut response = axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();

                let data: api_error_derive::ApiErrorData = self.into();
                response.extensions_mut().insert(data);

                response
            }
        }
    }
}

#[cfg(not(feature = "axum"))]
fn axum_into_response(_name: Ident) -> TokenStream {
    TokenStream::default()
}

fn api_error_inner(input: DeriveInput) -> syn::Result<TokenStream> {
    api_error_validate(&input)?;

    let name = input.ident;

    let branches = match input.data {
        syn::Data::Enum(val) => {
            let ident = val.variants.into_iter().map(|variant| {
                let mut was_pass = false;
                let mut status_code = None;
                let mut custom = None;

                for attr in variant.attrs {
                    match attr.meta {
                        Meta::Path(path) => {
                            if path.is_ident("pass") {
                                was_pass = true;
                            }
                        }

                        Meta::List(list) => {
                            if list.path.is_ident("status_code") {
                                status_code = Some(syn::parse2::<Ident>(list.tokens).unwrap());
                                was_pass = true;
                            } else if list.path.is_ident("custom") {
                                custom = Some(syn::parse2::<LitStr>(list.tokens).unwrap().value());
                                was_pass = true;
                            }
                        }

                        _ => (),
                    }
                }

                let ident = variant.ident;

                if !was_pass {
                    quote! {
                        #name::#ident { .. } => api_error_derive::ApiErrorData::new(
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            self_to_string,
                            #INTERNAL_SERVER_ERROR_STR.to_owned(),
                        ),
                    }
                } else {
                    let status_code = status_code
                        .unwrap_or_else(|| syn::parse_str("INTERNAL_SERVER_ERROR").unwrap());
                    let custom = custom.unwrap_or_else(|| ident.to_string());

                    quote! {
                        #name::#ident { .. } => api_error_derive::ApiErrorData::new(
                            axum::http::StatusCode::#status_code,
                            self_to_string,
                            #custom.to_owned(),
                        ),
                    }
                }
            });

            quote! {
                #(#ident)*
            }
        }
        _ => unreachable!(),
    };

    let axum = axum_into_response(name.clone());

    Ok(quote! {
        #axum

        impl std::convert::Into<api_error_derive::ApiErrorData> for #name {
            fn into(self) -> api_error_derive::ApiErrorData {
                let self_to_string = self.to_string();

                match self {
                    #branches
                }
            }
        }
    })
}

#[proc_macro_derive(ApiError, attributes(pass, status_code, custom))]
pub fn api_error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    api_error_inner(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

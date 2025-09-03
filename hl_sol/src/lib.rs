extern crate proc_macro;
use heck::ToShoutySnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

#[proc_macro]
pub fn sol(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    // extract struct name and check for multisig attribute
    let struct_name = &input.ident;
    let is_multisig = input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("multisig"));

    // extract fields from struct
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("hl_sol: only named fields are supported"),
        },
        _ => panic!("hl_sol: only structs are supported"),
    };

    // build field information for type string and sol struct
    let mut sol_fields = Vec::new();
    let mut type_string_parts = Vec::new();
    let mut hyperliquid_chain_index = None;

    for (index, field) in fields.iter().enumerate() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = get_type_string(&field.ty);

        // track position of hyperliquidChain field for multisig insertion
        if field_name == "hyperliquidChain" {
            hyperliquid_chain_index = Some(index);
        }

        // build sol field and type string part
        let field_type_ident = Ident::new(&field_type, Span::call_site());
        sol_fields.push(quote! { #field_type_ident #field_name; });
        type_string_parts.push(format!("{} {}", field_type, field_name));
    }

    // generate main type constant and struct
    let type_name = Ident::new(
        &format!("{}_TYPE", struct_name.to_string().to_shouty_snake_case()),
        Span::call_site(),
    );

    let type_string = format!(
        "HyperliquidTransaction:{}({})",
        struct_name,
        type_string_parts.join(",")
    );

    let mut output = quote! {
        pub const #type_name: &str = #type_string;

        ::alloy::sol! {
            struct #struct_name {
                #(#sol_fields)*
            }
        }
    };

    // generate multisig variant if requested
    if is_multisig {
        // hyperliquidChain field is required for multisig
        if hyperliquid_chain_index.is_none() {
            panic!("hl_sol: multisig structs must have a 'hyperliquidChain' field");
        }

        let multisig_struct_name =
            Ident::new(&format!("MultiSig{}", struct_name), Span::call_site());

        let multisig_type_name = Ident::new(
            &format!(
                "{}_MULTISIG_TYPE",
                struct_name.to_string().to_shouty_snake_case()
            ),
            Span::call_site(),
        );

        // define multisig fields to insert
        let payload_multi_sig_user = Ident::new("payloadMultiSigUser", Span::call_site());
        let outer_signer = Ident::new("outerSigner", Span::call_site());
        let address_type = Ident::new("address", Span::call_site());

        // insert multisig fields after hyperliquidChain
        let insert_position = hyperliquid_chain_index.unwrap() + 1;

        sol_fields.insert(
            insert_position,
            quote! { #address_type #payload_multi_sig_user; },
        );
        sol_fields.insert(insert_position + 1, quote! { #address_type #outer_signer; });

        type_string_parts.insert(
            insert_position,
            format!("address {}", payload_multi_sig_user),
        );
        type_string_parts.insert(insert_position + 1, format!("address {}", outer_signer));

        let multisig_type_string = format!(
            "HyperliquidTransaction:{}({})",
            struct_name,
            type_string_parts.join(",")
        );

        output.extend(quote! {
            pub const #multisig_type_name: &str = #multisig_type_string;

            ::alloy::sol! {
                struct #multisig_struct_name {
                    #(#sol_fields)*
                }
            }
        });
    }

    proc_macro::TokenStream::from(output)
}

// extract type string from syn::Type
// simply returns the identifier as-is, letting sol! macro handle validation
fn get_type_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .get_ident()
            .map(|ident| ident.to_string())
            .unwrap_or_else(|| panic!("hl_sol: complex types not supported")),
        _ => panic!("hl_sol: only simple type names supported"),
    }
}

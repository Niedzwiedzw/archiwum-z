extern crate proc_macro2;

use darling::{
    FromDeriveInput,
    FromMeta,
};

use proc_macro::{
    Ident,
    TokenStream,
};
use quote::quote;
use syn::{
    parse_macro_input,
    spanned::Spanned,
    TypePath,
};

#[derive(FromDeriveInput, Debug, Default)]
#[darling(default, attributes(iced_form), forward_attrs(allow, doc, cfg))]
struct IcedFormOptions {
    derive_into: Option<syn::Path>,
}

#[proc_macro_derive(IcedForm, attributes(iced_form))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);
    let options = IcedFormOptions::from_derive_input(&ast).expect("wrong options");

    let name = &ast.ident;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!();
    };
    let fields_with_types = fields
        .iter()
        .map(|f| {
            let ty = f.ty.clone();
            let ident = f.ident.as_ref().cloned().unwrap();
            (ident, ty)
        })
        .collect::<Vec<_>>();
    // let to_view_definition = |ident, ty| {
    //     quote! {
    //         impl<'a, , Message> IcedForm<'a, T, Message> for T
    //     }
    // };
    // let struct_fields = fields_with_types
    //     .iter()
    //     .map(|(ty, ident)| quote! {#ty: #ident});

    let map_ident_name = |name: &syn::Ident, mapping: fn(&str) -> String| {
        let mapped = mapping(&format!("{name}"));
        syn::Ident::new(&mapped, name.span())
    };
    let enum_ident = map_ident_name(name, |name| format!("{name}FieldUpdated"));
    // let enum_variants = fields_with_types.iter().map(|(ident, ty)|
    // let variant_name = map_ident_name(ident, |name| format!("{}Updated"));
    // quote!{
    //     #variant_name(#ty::)
    // });

    let output = quote! {
    pub enum #enum_ident {

    }
    impl<'a, #name, Message> IcedForm<'a, #name, Message> for T
    where
        T: iced_forms::IcedFormValue,
        Message: Clone + 'a,
    {
        fn view(
            &self,
            name: &'static str,
            value: &T,
            on_change: impl Fn(T) -> Message + 'a,
            on_error: impl Fn(IcedFormValueError) -> Message + 'a,
        ) -> iced::pure::widget::Container<'a, Message> {
            container::<'a>(text_input(
                name,
                &value.serialize(),
                move |val: String| match T::deserialize(&val) {
                    Ok(v) => on_change(v),
                    Err(e) => on_error(e),
                },
            ))
        }
    }
        };
    let output = TokenStream::from(output);
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

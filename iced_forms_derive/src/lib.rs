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
struct IcedFormDeriveOptions {
    derive_into: Option<syn::Path>,
}

fn pretty_print_tokenstream(code: &'_ TokenStream) {
    fn try_format(input: &'_ str) -> Option<String> {
        Some({
            let mut child = ::std::process::Command::new("rustfmt")
                .args(&["--edition", "2021"])
                .stdin(::std::process::Stdio::piped())
                .stdout(::std::process::Stdio::piped())
                .stderr(::std::process::Stdio::piped())
                .spawn()
                .ok()?;
            let stdin = &mut child.stdin.take().unwrap();
            {
                ::std::io::Write::write_all(stdin, input.as_bytes()).ok()?;
            }
            let mut stdout = String::new();
            ::std::io::Read::read_to_string(&mut child.stdout.take().unwrap(), &mut stdout).ok()?;
            if !child.wait().ok()?.success() {
                return None;
            }
            stdout
        })
    }
    let mut code = code.to_string();
    // Try to format the code, but don't sweat it if it fails.
    if let Some(formatted) = try_format(&code) {
        code = formatted;
    }
    // Now let's try to also colorize it:

    // Fallback to non-colorized output.
    eprintln!("{}", code);
}

#[proc_macro_derive(IcedFormDerive, attributes(iced_form))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);
    let options = IcedFormDeriveOptions::from_derive_input(&ast).expect("wrong options");

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

    let map_ident_name = |name: &syn::Ident, mapping: fn(&str) -> String| {
        let mapped = mapping(&format!("{name}"));
        syn::Ident::new(&mapped, name.span())
    };
    let enum_ident = map_ident_name(name, |name| format!("{name}FieldUpdated"));

    let to_enum_variant_name_updated =
        |ident: &syn::Ident| map_ident_name(ident, |name| format!("{name}Updated"));
    let to_enum_variant_name_error =
        |ident: &syn::Ident| map_ident_name(ident, |name| format!("{name}Error"));

    let to_enum_variant = |(ident, ty): &(syn::Ident, syn::Type)| {
        let variant_name_updated = to_enum_variant_name_updated(ident);
        let variant_name_error = to_enum_variant_name_error(ident);
        quote! {
            #variant_name_updated(<#ty as iced_forms::IcedForm<'a, #ty>>::Message),
            #variant_name_error(iced_forms::IcedFormValueError)
        }
    };
    let enum_variants = fields_with_types
        .iter()
        .map(to_enum_variant)
        .collect::<Vec<_>>();

    let subform_pushes = fields_with_types.iter().map(|(ident, ty)| {
        let field_header_text = stringify!(ident);
        let variant_name_update = to_enum_variant_name_updated(ident);
        let variant_name_error = to_enum_variant_name_error(ident);

        let type_name = stringify!(ty);
        // let enum_variant = to_enum_variant((ident, ty));
        quote! {
            .push(
                row()
                .push(text(#field_header_text))
                .push(
                    self.#ident.view(
                        #type_name,
                        move |changed_field| match changed_field {
                            Ok(value) => {
                                let mut form = self.clone();
                                form.#ident = value;
                                on_change(Ok(form))
                            },
                            Err(e) => {
                                on_change(Err(e))
                            }
                        },
                        )
                    )
                )
        }
    });

    let output = quote! {

    impl<'a> iced_forms::IcedForm<'a> for #name {
        fn view(
            &self,
            name: &'static str,
            on_change: impl Fn(iced_forms::IcedFormValueResult<Self>) -> iced_forms::IcedFormValueResult<Self> + 'a,
        ) -> iced::pure::widget::Container<'a, iced_forms::IcedFormValueResult<Self>> {
            use iced::pure::{container, column, row, text};
            iced::pure::container::<'a>(column() #(#subform_pushes)*)
        }
    }
        };

    let output = TokenStream::from(output);
    eprintln!("{output}");
    // pretty_print_tokenstream(&output);
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

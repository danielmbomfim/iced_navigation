use proc_macro::TokenStream;
use proc_quote::quote;
use syn::{
    parse_macro_input, punctuated, token, Data, DeriveInput, Field, Fields, FieldsUnnamed,
    GenericArgument, Ident, ItemEnum, Lit, LitStr, Meta, NestedMeta, Path, Type, Variant,
    Visibility,
};

#[proc_macro_derive(NavigationConvertible)]
pub fn derive_navigation_mapper(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let enum_name = &input.ident;

    let Data::Enum(enum_data) = input.data else {
        return syn::Error::new_spanned(
            enum_name,
            "NavigationConvertible can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    let mut page_type: Option<Type> = None;

    for variant in enum_data.variants.iter() {
        if let Variant {
            ident: ref variant_ident,
            fields: Fields::Unnamed(ref fields),
            ..
        } = variant
        {
            if variant_ident == "NavigationAction" {
                if let Some(field) = fields.unnamed.first() {
                    if let Type::Path(ref path) = field.ty {
                        if path.path.segments.len() == 1
                            && path.path.segments[0].ident == "NavigationAction"
                        {
                            if let syn::PathArguments::AngleBracketed(ref args) =
                                path.path.segments[0].arguments
                            {
                                if let Some(GenericArgument::Type(ty)) = args.args.first() {
                                    page_type = Some(ty.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let page_type = match page_type {
      Some(value) => value,
      None => return syn::Error::new_spanned(
          enum_name,
          "NavigationConvertible can only be derived for enums with the variant NavigationAction(NavigationAction<T>)",
      )
      .to_compile_error()
      .into()
    };

    let trait_name = Ident::new("NavigationConvertible", enum_name.span());

    let expanded = quote! {
        impl #trait_name for #enum_name {
            type PageMapper = #page_type;

            fn from_action(action: NavigationAction<Self::PageMapper>) -> Self {
                #enum_name::NavigationAction(action)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn navigator_message(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemEnum);
    let enum_name = &input.ident;

    let attr_str = attr.to_string();
    let page_mapper = Ident::new(&attr_str, enum_name.span());

    let navigation_action_variant = Variant {
        attrs: vec![],
        ident: Ident::new("NavigationAction", enum_name.span()),
        fields: Fields::Unnamed(FieldsUnnamed {
            paren_token: token::Paren::default(),
            unnamed: punctuated::Punctuated::from_iter(vec![Field {
                vis: Visibility::Inherited,
                ident: None,
                colon_token: None,
                attrs: vec![],
                ty: Type::Verbatim(quote! { NavigationAction<#page_mapper> }),
            }]),
        }),
        discriminant: None,
    };

    input.variants.push(navigation_action_variant);

    let expanded = quote! { #input };

    TokenStream::from(expanded)
}

struct PageAttributes {
    title: LitStr,
    component: Path,
    settings: Option<Path>,
}

impl PageAttributes {
    fn parse(enum_name: &Ident, value: &Variant) -> Result<Self, syn::Error> {
        let Some(attr) = value.attrs.iter().find(|attr| attr.path.is_ident("page")) else {
            return Err(syn::Error::new_spanned(
                enum_name,
                "page attribute must be defined in each variant of the enum",
            ));
        };

        let Ok(Meta::List(meta_list)) = attr.parse_meta() else {
            return Err(syn::Error::new_spanned(
                enum_name,
                "page attribute must be defined in each variant of the enum",
            ));
        };

        let mut title = None;
        let mut component: Option<Path> = None;
        let mut settings: Option<Path> = None;

        for nested_meta in meta_list.nested.iter() {
            if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
                if name_value.path.is_ident("title") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        title = Some(lit_str);
                    }
                } else if name_value.path.is_ident("component") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        component = Some(lit_str.parse().expect("Expected a valid function path"));
                    }
                } else if name_value.path.is_ident("settings") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        settings = Some(lit_str.parse().expect("Expected a valid function path"));
                    }
                }
            }
        }

        Ok(Self {
            title: title.expect("a title is expected for each page").to_owned(),
            component: component
                .expect("a component is expected for each page")
                .to_owned(),
            settings,
        })
    }
}

#[proc_macro_derive(StackNavigatorMapper, attributes(message, page))]
pub fn derive_stack_navigator_mapper(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let enum_name = &input.ident;

    let Data::Enum(enum_data) = input.data else {
        return syn::Error::new_spanned(
            enum_name,
            "StackNavigatorMapper can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    let mut message_type: Option<Path> = None;

    for attr in &input.attrs {
        if attr.path.is_ident("message") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                if let Some(NestedMeta::Meta(Meta::Path(path))) = meta_list.nested.first() {
                    message_type = Some(path.clone());
                }
            }
        }
    }

    let message_type =
        match message_type {
            Some(name) => name,
            None => return syn::Error::new_spanned(
                enum_name,
                "StackNavigatorMapper must specify #[message(Message)] attribute, where \"Message\" is your message enum.",
            )
            .into_compile_error()
            .into(),
        };

    let trait_name = Ident::new("StackNavigatorMapper", enum_name.span());
    let mut variant_attrs: Vec<PageAttributes> = Vec::with_capacity(enum_data.variants.len());

    for variant in enum_data.variants.iter() {
        match PageAttributes::parse(enum_name, variant) {
            Ok(value) => variant_attrs.push(value),
            Err(err) => return err.to_compile_error().into(),
        };
    }

    let title_match =
        enum_data
            .variants
            .iter()
            .zip(variant_attrs.iter())
            .map(|(variant, result)| {
                let variant_name = &variant.ident;
                let title = &result.title;

                match &variant.fields {
                    Fields::Unit => {
                        quote! { Self::#variant_name => #title }
                    }
                    Fields::Unnamed(_) => {
                        quote! { Self::#variant_name(..) => #title }
                    }
                    Fields::Named(_) => {
                        quote! { Self::#variant_name { .. } => #title }
                    }
                }
            });

    let component_match =
        enum_data
            .variants
            .iter()
            .zip(variant_attrs.iter())
            .map(|(variant, result)| {
                let variant_name = &variant.ident;
                let component = &result.component;

                match &variant.fields {
                    Fields::Unit => {
                        quote! { Self::#variant_name => Box::new(#component()) }
                    }
                    Fields::Unnamed(fields) => {
                        let params: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| Ident::new(&format!("arg{}", i), variant_name.span()))
                            .collect();

                        quote! { Self::#variant_name(#(#params),*) => Box::new(#component(#(#params),*)) }
                    }
                    Fields::Named(fields) => {
                        let params: Vec<_> = fields
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().unwrap())
                            .collect();

                        quote! { Self::#variant_name { #(#params),* } => Box::new(#component(#(#params),*)) }
                    }
                }
            });

    let settings_match = enum_data.variants.iter().zip(variant_attrs.iter()).map(
        |(variant, result)| {
            let variant_name = &variant.ident;
            let settings = &result.settings;

            match &variant.fields {
                Fields::Unit => match settings {
                    Some(value) => quote! { Self::#variant_name => #value() },
                    None => quote! { Self::#variant_name => None },
                },
                Fields::Unnamed(fields) => {
                    let params: Vec<_> = (0..fields.unnamed.len())
                        .map(|i| Ident::new(&format!("arg{}", i), variant_name.span()))
                        .collect();

                    match settings {
                        Some(value) => {
                            quote! { Self::#variant_name(#(#params),*) => #value(#(#params),*) }
                        }
                        None => quote! { Self::#variant_name(..) => None },
                    }
                }
                Fields::Named(fields) => {
                    let params: Vec<_> = fields
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();

                    match settings {
                        Some(value) => {
                            quote! { Self::#variant_name{ #(#params),* } => #value(#(#params),*) }
                        }
                        None => quote! { Self::#variant_name{ .. } => None },
                    }
                }
            }
        },
    );

    let expanded = quote! {
        impl #trait_name for #enum_name {
          type Message = #message_type;

          fn title(&self) -> String {
            match self {
              #(#title_match),*
            }.to_owned()
          }

          fn into_component(&self) -> Box<dyn PageComponent<Self::Message>> {
            match self {
              #(#component_match),*
            }
          }

          fn settings(&self) -> Option<HeaderSettings> {
            match self {
              #(#settings_match),*
            }
          }
        }
    };

    TokenStream::from(expanded)
}

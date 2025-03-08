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
                        } else if path.path.segments.len() == 2
                            && path.path.segments[1].ident == "NavigationAction"
                        {
                            if let syn::PathArguments::AngleBracketed(ref args) =
                                path.path.segments[1].arguments
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

            fn from_action(action: iced_navigation::NavigationAction<Self::PageMapper>) -> Self {
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
                ty: Type::Verbatim(quote! { iced_navigation::NavigationAction<#page_mapper> }),
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
    title_component: Option<Path>,
    back_button: Option<Path>,
    right_button: Option<Path>,
}

impl PageAttributes {
    fn parse(value: &Variant) -> Result<Self, syn::Error> {
        let Some(attr) = value.attrs.iter().find(|attr| attr.path.is_ident("page")) else {
            return Err(syn::Error::new_spanned(
                value,
                "Each variant of the enum must have #[page(...)] declared.",
            ));
        };

        let Ok(Meta::List(meta_list)) = attr.parse_meta() else {
            return Err(syn::Error::new_spanned(
                value,
                "Each variant of the enum must have #[page(...)] declared.",
            ));
        };

        let mut title = None;
        let mut component: Option<Path> = None;
        let mut settings: Option<Path> = None;
        let mut title_component: Option<Path> = None;
        let mut back_button: Option<Path> = None;
        let mut right_button: Option<Path> = None;

        for nested_meta in meta_list.nested.iter() {
            if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
                if name_value.path.is_ident("title") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        title = Some(lit_str);
                    }
                } else if name_value.path.is_ident("component") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        component = match lit_str.parse() {
                            Ok(value) => Some(value),
                            Err(_) => {
                                return Err(syn::Error::new_spanned(
                                    name_value,
                                    "component must be a function, for example, #[page(component = my_function)]",
                                ))
                            }
                        };
                    }
                } else if name_value.path.is_ident("settings") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        settings = match lit_str.parse() {
                            Ok(value) => Some(value),
                            Err(_) => {
                                return Err(
                                  syn::Error::new_spanned(
                                    name_value,
                                    "settings must be a function, for example, #[page(settings = my_function)]",
                                  )
                                );
                            }
                        };
                    }
                } else if name_value.path.is_ident("title_component") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        title_component = match lit_str.parse() {
                            Ok(value) => Some(value),
                            Err(_) => {
                                return Err(
                                  syn::Error::new_spanned(
                                    name_value,
                                    "title_component must be a function, for example, #[page(title_component = my_function)]"
                                  )
                                );
                            }
                        };
                    }
                } else if name_value.path.is_ident("back_button") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        back_button = match lit_str.parse() {
                            Ok(value) => Some(value),
                            Err(_) => {
                                return Err(
                                  syn::Error::new_spanned(
                                    name_value,
                                    "back_button must be a function, for example, #[page(back_button = my_function)]",
                                  )
                                );
                            }
                        };
                    }
                } else if name_value.path.is_ident("right_button") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        right_button = match lit_str.parse() {
                            Ok(value) => Some(value),
                            Err(_) => {
                                return Err(
                                  syn::Error::new_spanned(
                                    name_value,
                                    "right_button must be a function, for example, #[page(right_button = my_function)]",
                                  )
                                );
                            }
                        };
                    }
                }
            }
        }

        Ok(Self {
            title: match title {
                Some(value) => value.to_owned(),
                None => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        r#"Expected #[page(title = "your page title")]"#,
                    ))
                }
            },
            component: match component {
                Some(value) => value.to_owned(),
                None => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        r#"Expected #[page(component = "your page component")]"#,
                    ))
                }
            },
            settings,
            title_component,
            back_button,
            right_button,
        })
    }
}

fn func_to_match(variant: &Variant, function_path: &Option<Path>) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;

    match &variant.fields {
        Fields::Unit => match function_path {
            Some(value) => quote! { Self::#variant_name => Some(#value()) },
            None => quote! { Self::#variant_name => None },
        },
        Fields::Unnamed(fields) => {
            let params: Vec<_> = (0..fields.unnamed.len())
                .map(|i| Ident::new(&format!("arg{}", i), variant_name.span()))
                .collect();

            match function_path {
                Some(value) => {
                    quote! { Self::#variant_name(#(#params),*) => Some(#value(#(#params),*)) }
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

            match function_path {
                Some(value) => {
                    quote! { Self::#variant_name{ #(#params),* } => Some(#value(#(#params),*)) }
                }
                None => quote! { Self::#variant_name{ .. } => None },
            }
        }
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
        match PageAttributes::parse(variant) {
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

    let settings_match =
        enum_data
            .variants
            .iter()
            .zip(variant_attrs.iter())
            .map(|(variant, result)| {
                let settings = &result.settings;

                func_to_match(&variant, settings)
            });

    let title_component_match =
        enum_data
            .variants
            .iter()
            .zip(variant_attrs.iter())
            .map(|(variant, result)| {
                let title_component = &result.title_component;

                func_to_match(&variant, title_component)
            });

    let back_button_match =
        enum_data
            .variants
            .iter()
            .zip(variant_attrs.iter())
            .map(|(variant, result)| {
                let back_button = &result.back_button;

                func_to_match(&variant, back_button)
            });

    let right_button_match =
        enum_data
            .variants
            .iter()
            .zip(variant_attrs.iter())
            .map(|(variant, result)| {
                let right_button = &result.right_button;

                func_to_match(&variant, right_button)
            });

    let expanded = quote! {
        impl #trait_name for #enum_name {
          type Message = #message_type;

          fn title(&self) -> String {
            match self {
              #(#title_match),*
            }.to_owned()
          }

          fn into_component(&self) -> Box<dyn iced_navigation::PageComponent<Self::Message>> {
            match self {
              #(#component_match),*
            }
          }

          fn settings(&self) -> Option<iced_navigation::components::header::HeaderSettings> {
            match self {
              #(#settings_match),*
            }
          }

          fn back_button(&self) -> Option<Box<dyn iced_navigation::components::header::HeaderButtonElement<Self::Message>>> {
            match self {
              #(#back_button_match),*
            }
          }

          fn right_button(&self) -> Option<Box<dyn iced_navigation::components::header::HeaderButtonElement<Self::Message>>> {
            match self {
              #(#right_button_match),*
            }
          }

          fn title_widget(&self) -> Option<Box<dyn iced_navigation::components::header::HeaderTitleElement<Self::Message>>> {
            match self {
              #(#title_component_match),*
            }
          }
        }
    };

    TokenStream::from(expanded)
}

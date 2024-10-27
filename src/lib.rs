use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

extern crate proc_macro;
extern crate proc_macro2;

#[proc_macro_attribute]
pub fn interface(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_impl = syn::parse_macro_input!(item as syn::ItemImpl);
    let item_impl_stream = quote! {#item_impl};
    let methods = get_tagged_methods(item_impl).unwrap();
    let additional_item_impl_stream = create_additional_item_impl_stream(&methods);
    let variant_callable_item_impl_stream = create_variant_callable_item_impl_stream(&methods);
    let out_stream = quote! {
        #item_impl_stream
        #additional_item_impl_stream
        #variant_callable_item_impl_stream
    };
    proc_macro::TokenStream::from(out_stream)
}

#[proc_macro_attribute]
pub fn method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn create_additional_item_impl_stream(methods: &Vec<TaggedMethod>) -> proc_macro2::TokenStream {
    let method_idents = methods.iter().map(|m| quote::format_ident!("{}", m.ident));
    let additional_method_idents = methods
        .iter()
        .map(|m| quote::format_ident!("{}_variant", m.ident));
    let input_types = methods.iter().map(|m| m.input_types.clone());
    let idxs: Vec<Vec<syn::Index>> = methods
        .iter()
        .map(|m| -> Vec<syn::Index> { (0..m.input_types.len()).map(syn::Index::from).collect() })
        .collect();
    quote! {
        impl Greeter {
            #(
                pub fn #additional_method_idents(&mut self, args: glib::Variant) -> glib::Variant {
                    let args: (#(#input_types),*,) = FromVariant::from_variant(&args).unwrap();
                    let res = self.#method_idents(#(args.#idxs),*);
                    let res = ToVariant::to_variant(&res);
                    res
                }
            )*
        }
    }
}

fn create_variant_callable_item_impl_stream(
    methods: &Vec<TaggedMethod>,
) -> proc_macro2::TokenStream {
    quote! {}
}

struct TaggedMethod {
    pub ident: String,
    pub input_types: Vec<syn::Type>,
    pub output_type: Option<syn::Type>,
}

impl From<syn::Signature> for TaggedMethod {
    fn from(value: syn::Signature) -> Self {
        let ident = value.ident.to_string();
        let input_types = get_input_types(value.inputs);
        let output_type = get_output_type(value.output);
        TaggedMethod {
            ident,
            input_types,
            output_type,
        }
    }
}

fn get_input_types(arguments: Punctuated<syn::FnArg, syn::token::Comma>) -> Vec<syn::Type> {
    let mut input_types = vec![];
    for arg in arguments {
        match arg {
            syn::FnArg::Typed(t) => input_types.push(*t.ty),
            syn::FnArg::Receiver(_) => (),
        }
    }
    input_types
}

fn get_output_type(return_type: syn::ReturnType) -> Option<syn::Type> {
    match return_type {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, t) => Some(*t),
    }
}

fn get_tagged_methods(impl_item: syn::ItemImpl) -> Result<Vec<TaggedMethod>, ()> {
    let methods = get_methods(impl_item)?;
    Ok(filter_tagged_methods(methods)?)
}

fn get_methods(impl_item: syn::ItemImpl) -> Result<Vec<syn::ImplItemFn>, ()> {
    let mut methods = vec![];
    for item in impl_item.items {
        match item {
            syn::ImplItem::Fn(fun) => methods.push(fun),
            _ => (),
        };
    }
    Ok(methods)
}

fn filter_tagged_methods(methods: Vec<syn::ImplItemFn>) -> Result<Vec<TaggedMethod>, ()> {
    let mut tagged_methods = vec![];
    for method in methods {
        for attr in method.attrs {
            match (attr.style, attr.meta) {
                (syn::AttrStyle::Outer, syn::Meta::Path(path)) if path.is_ident("method") => {
                    tagged_methods.push(TaggedMethod::from(method.sig.clone()))
                }
                _ => (),
            }
        }
    }
    Ok(tagged_methods)
}

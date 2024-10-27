use proc_macro::TokenStream;
use syn::punctuated::Punctuated;

extern crate proc_macro;

#[proc_macro_attribute]
pub fn interface(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let i = item.clone();
    let impl_item = syn::parse_macro_input!(item as syn::ItemImpl);
    let methods = get_tagged_methods(impl_item);
    // TODO: Generate <method_name>_variant & impl VariantCallable block.
    i
}

#[proc_macro_attribute]
pub fn method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

struct TaggedMethod {
    ident: String,
    input_types: Vec<syn::Type>,
    output_type: Option<syn::Type>,
}

impl From<syn::Signature> for TaggedMethod {
    fn from(value: syn::Signature) -> Self {
        let ident = value.ident.to_string();
        let input_types = get_input_types(value.inputs);
        let output_type = get_output_type(value.output);
        TaggedMethod {ident, input_types, output_type}
    }
}

fn get_input_types(arguments: Punctuated<syn::FnArg,syn::token::Comma>) -> Vec<syn::Type> {
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

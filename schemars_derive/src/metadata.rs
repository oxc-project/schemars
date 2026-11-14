use proc_macro2::TokenStream;

#[derive(Debug, Clone)]
pub struct SchemaMetadata<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub deprecated: bool,
    pub read_only: bool,
    pub write_only: bool,
    pub examples: &'a [syn::Path],
    pub default: Option<TokenStream>,
}

impl<'a> SchemaMetadata<'a> {
    pub fn apply_to_schema(&self, schema_expr: &mut TokenStream) {
        if !self.examples.is_empty() {
            let examples = self.examples.iter().map(|eg| {
                quote! {
                    schemars::_serde_json::value::to_value(#eg())
                }
            });

            *schema_expr = quote! {
                schemars::_private::metadata::add_examples(#schema_expr,  [#(#examples),*].into_iter().flatten())
            };
        }

        if self.has_static_metadata() || self.default.is_some() {
            let metadata = self.static_metadata();
            let default = self.default_value_expr();
            *schema_expr = quote! {
                schemars::_private::metadata::apply(#schema_expr, &#metadata, #default)
            };
        }
    }

    /// Whether any metadata other than `examples` and `default` is set.
    pub fn has_static_metadata(&self) -> bool {
        self.title.is_some()
            || self.description.is_some()
            || self.deprecated
            || self.read_only
            || self.write_only
    }

    /// A constant `schemars::_private::metadata::Metadata` expression, which the compiler
    /// promotes to a `&'static` when borrowed.
    pub fn static_metadata(&self) -> TokenStream {
        fn opt_str(s: Option<&str>) -> TokenStream {
            match s {
                Some(s) => quote!(Some(#s)),
                None => quote!(None),
            }
        }
        let title = opt_str(self.title);
        let description = opt_str(self.description);
        let deprecated = self.deprecated;
        let read_only = self.read_only;
        let write_only = self.write_only;
        quote! {
            schemars::_private::metadata::Metadata {
                title: #title,
                description: #description,
                deprecated: #deprecated,
                read_only: #read_only,
                write_only: #write_only,
            }
        }
    }

    /// An `Option<serde_json::Value>` expression for the `default` keyword.
    pub fn default_value_expr(&self) -> TokenStream {
        match &self.default {
            Some(default) => quote!(schemars::_schemars_maybe_to_value!(#default)),
            None => quote!(None),
        }
    }
}

use quote::{quote, quote_spanned, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Token,
};

mod attrs;

pub use attrs::{Compute, Count, Offset};
use attrs::{FieldAttrs, ItemAttrs, VariantAttrs};

pub struct Items {
    pub docs: Vec<syn::Attribute>,
    pub use_stmts: Vec<SimpleUse>,
    pub items: Vec<Item>,
    pub helpers: Vec<syn::ItemFn>,
}

pub enum Item {
    Single(SingleItem),
    Group(ItemGroup),
    RawEnum(RawEnum),
    Flags(BitFlags),
}

/// A single concrete object, such as a particular table version or record format.
pub struct SingleItem {
    pub docs: Vec<syn::Attribute>,
    pub lifetime: Option<syn::Lifetime>,
    pub offset_host: Option<syn::Path>,
    pub manual_compile_type: Option<syn::Path>,
    pub init: Vec<(syn::Ident, syn::Type)>,
    pub name: syn::Ident,
    pub fields: Vec<Field>,
}

/// A group of items that can exist in the same location, such as tables
/// with multiple versions.
pub struct ItemGroup {
    pub docs: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub lifetime: Option<syn::Lifetime>,
    pub format_typ: syn::Ident,
    // the version attribute, if present. the actual version is stored in `format_typ`
    pub generate_getters: Option<syn::Path>,
    pub offset_host: Option<syn::Path>,
    pub variants: Vec<Variant>,
}

/// A raw c-style enum
pub struct RawEnum {
    pub docs: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub repr: syn::Ident,
    pub variants: Vec<RawVariant>,
}

/// A raw scalar variant
pub struct RawVariant {
    pub docs: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub value: syn::LitInt,
}

/// A set of bit-flags
pub struct BitFlags {
    pub docs: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub type_: syn::Ident,
    pub variants: Vec<RawVariant>,
}

pub struct Variant {
    pub docs: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub version: attrs::Version,
    pub typ: syn::Ident,
    pub typ_lifetime: Option<syn::Lifetime>,
}

pub enum Field {
    Single(SingleField),
    Array(ArrayField),
}

pub struct SingleField {
    pub docs: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub typ: FieldType,
    pub hidden: Option<syn::Path>,
    pub offset: Option<Offset>,
    pub compute: Option<Compute>,
}

pub enum FieldType {
    Offset {
        offset_type: syn::Ident,
        target_type: Option<syn::Ident>,
    },
    Scalar {
        typ: syn::Ident,
    },
    Other {
        typ: syn::Path,
    },
}

pub struct ArrayField {
    pub docs: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub inner_typ: FieldType,
    pub inner_lifetime: Option<syn::Lifetime>,
    pub count: Count,
    pub variable_size: Option<syn::Path>,
}

/// A simple 'use' statement consisting of a single path.
pub struct SimpleUse(syn::Path);

impl Parse for SimpleUse {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let _use_token = input.parse::<Token![use]>()?;
        let path: syn::Path = input.parse().map_err(|_| {
            syn::Error::new(
                _use_token.span(),
                "only simple/single use statements of form 'use path::to::Item' are supported",
            )
        })?;
        input.parse::<Token![;]>()?;
        Ok(SimpleUse(path))
    }
}

impl quote::ToTokens for SimpleUse {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let path = &self.0;
        tokens.extend(quote!(use #path;))
    }
}

impl Parse for Items {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let docs = get_optional_module_docs(input)?;
        let use_stmts = get_use_statements(input)?;
        let mut items = Vec::new();
        let mut helpers = Vec::new();
        while !input.is_empty() {
            if input.peek(Token![fn]) {
                helpers.push(input.parse()?);
            } else {
                items.push(input.parse()?);
            }
        }
        Ok(Self {
            use_stmts,
            docs,
            items,
            helpers,
        })
    }
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let attrs = get_optional_attributes(input)?;
        let attrs = ItemAttrs::parse(&attrs)?;
        let enum_token = input
            .peek(Token![enum])
            .then(|| input.parse::<Token![enum]>())
            .transpose()?;
        let name: syn::Ident = input.parse()?;
        let lifetime = validate_lifetime(input)?;
        let content;
        let _ = braced!(content in input);
        if enum_token.is_some() && attrs.repr.is_some() {
            let variants = Punctuated::<RawVariant, Token![,]>::parse_terminated(&content)?
                .into_iter()
                .collect();
            RawEnum::new(name, variants, attrs).map(Self::RawEnum)
        } else if enum_token.is_some() {
            let variants = Punctuated::<Variant, Token![,]>::parse_terminated(&content)?;
            let variants = variants.into_iter().collect();
            ItemGroup::new(name, lifetime, variants, attrs).map(Self::Group)
        } else if attrs.flags.is_some() {
            let variants = Punctuated::<RawVariant, Token![,]>::parse_terminated(&content)?
                .into_iter()
                .collect();
            BitFlags::new(name, variants, attrs).map(Self::Flags)
        } else {
            let fields = Punctuated::<Field, Token![,]>::parse_terminated(&content)?;
            let fields = fields.into_iter().collect();
            let item = SingleItem {
                docs: attrs.docs,
                offset_host: attrs.offset_host,
                manual_compile_type: attrs.manual_compile,
                init: attrs.init,
                lifetime,
                name,
                fields,
            };
            item.validate()?;
            Ok(Self::Single(item))
        }
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let attrs = get_optional_attributes(input)?;
        let attrs = VariantAttrs::parse(&attrs)?;
        let name = input.parse::<syn::Ident>()?;
        let content;
        parenthesized!(content in input);
        let typ = content.parse::<syn::Ident>()?;
        let typ_lifetime = validate_lifetime(&content)?;
        if let Some(version) = attrs.version {
            Ok(Self {
                docs: attrs.docs,
                name,
                version,
                typ,
                typ_lifetime,
            })
        } else {
            Err(syn::Error::new(
                name.span(),
                "all variants require #[version(..)] attribute",
            ))
        }
    }
}

impl Parse for RawVariant {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let attrs = get_optional_attributes(input)?;
        let attrs = VariantAttrs::parse(&attrs)?;
        let name = input.parse::<syn::Ident>()?;
        let _ = input.parse::<Token![=]>()?;
        let value: syn::LitInt = input.parse()?;
        Ok(Self {
            docs: attrs.docs,
            name,
            value,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let attrs = get_optional_attributes(input)?;
        let attrs = FieldAttrs::parse(&attrs)?;
        let name = input.parse::<syn::Ident>()?;
        let _ = input.parse::<Token![:]>()?;

        if input.lookahead1().peek(token::Bracket) {
            let content;
            bracketed!(content in input);
            let typ = content.parse::<syn::Path>()?;
            let lifetime = ensure_single_lifetime(&typ)?;
            let typ = parse_field_type(&typ)?;
            attrs.into_array(name, typ, lifetime).map(Field::Array)
        } else {
            let typ = parse_field_type(&input.parse()?)?;
            attrs.into_single(name, typ).map(Field::Single)
        }
    }
}

impl Field {
    pub fn name(&self) -> &syn::Ident {
        match self {
            Field::Array(v) => &v.name,
            Field::Single(v) => &v.name,
        }
    }

    pub fn as_single(&self) -> Option<&SingleField> {
        match self {
            Field::Array(_) => None,
            Field::Single(v) => Some(v),
        }
    }

    pub fn as_array(&self) -> Option<&ArrayField> {
        match self {
            Field::Array(v) => Some(v),
            Field::Single(_) => None,
        }
    }

    fn is_array(&self) -> bool {
        matches!(self, Field::Array(_))
    }

    /// true if this is an offset or an array of offsets
    fn is_offset(&self) -> bool {
        match self {
            Field::Single(fld) => matches!(fld.typ, FieldType::Offset { .. }),
            Field::Array(fld) => matches!(fld.inner_typ, FieldType::Offset { .. }),
        }
    }

    fn requires_lifetime(&self) -> bool {
        self.is_array()
    }

    pub fn docs(&self) -> &[syn::Attribute] {
        match self {
            Field::Array(v) => &v.docs,
            Field::Single(v) => &v.docs,
        }
    }

    pub fn visible(&self) -> bool {
        match self {
            Field::Single(s) if s.hidden.is_some() => false,
            Field::Array(a) if a.variable_size.is_some() => false,
            _ => true,
        }
    }

    pub fn compile_type(&self) -> proc_macro2::TokenStream {
        match self {
            Field::Single(fld) => fld.typ.compile_type(),
            Field::Array(fld) => {
                let inner = fld.inner_typ.compile_type();
                quote!(Vec<#inner>)
            }
        }
    }

    pub fn is_computed(&self) -> bool {
        self.as_single()
            .map(|fld| fld.compute.is_some())
            .unwrap_or(false)
    }

    pub fn view_init_expr(&self) -> proc_macro2::TokenStream {
        let name = self.name();
        let span = name.span();
        let init_fn = match self {
            Field::Single(value) => {
                let typ = value.typ.view_field_tokens();
                quote_spanned!(span=> zerocopy::LayoutVerified::<_, #typ>::new_unaligned_from_prefix(bytes)?)
            }
            Field::Array(array) if array.variable_size.is_none() => {
                let typ = array.inner_typ.view_field_tokens();
                let count = match &array.count {
                    Count::Field(name) => {
                        let span = name.span();
                        let resolved_value = super::make_resolved_ident(name);
                        Some(quote_spanned!(span=> #resolved_value as usize))
                    }
                    Count::Literal(lit) => {
                        let span = lit.span();
                        Some(quote_spanned!(span=> #lit))
                    }
                    Count::Function { fn_, args } => {
                        let span = fn_.span();
                        let args = args.iter().map(super::make_resolved_ident);
                        Some(quote_spanned!(span=> #fn_( #( #args ),* )))
                    }
                    Count::All(_) => None,
                };
                if let Some(count) = count {
                    quote_spanned!(span=> zerocopy::LayoutVerified::<_, [#typ]>::new_slice_unaligned_from_prefix(bytes, #count)?)
                } else {
                    quote_spanned!(span => (zerocopy::LayoutVerified::<_, [#typ]>::new_slice_unaligned(bytes)?, 0))
                }
            }
            _ => quote_spanned!(span=> compile_errror!("we don't init this type yet")),
        };
        quote_spanned!(span=> let (#name, bytes) = #init_fn;)
    }

    pub fn view_getter_fn(&self) -> Option<proc_macro2::TokenStream> {
        if !self.visible() {
            return None;
        }
        let docs = self.docs();
        let name = self.name();
        let span = name.span();
        let body = self.getter_body(true);
        let return_type = self.getter_return_type();
        Some(quote_spanned! {span=>
            #( #docs )*
            pub fn #name(&self) -> #return_type {
                #body
            }
        })
    }

    pub fn getter_return_type(&self) -> proc_macro2::TokenStream {
        match self {
            Field::Single(field) => field.cooked_type_tokens(),
            Field::Array(array) => {
                let typ = array.inner_typ.view_field_tokens();
                let span = array.name.span();
                quote_spanned!(span=> &[#typ])
            }
        }
    }

    /// used in view init methods, for resolving fields that are used as arguments
    pub fn resolve_expr(&self) -> proc_macro2::TokenStream {
        self.getter_body(false)
    }

    fn getter_body(&self, with_self: bool) -> proc_macro2::TokenStream {
        let span = self.name().span();
        let name = self.name();
        let self_token = with_self.then(|| quote!(self.));
        match self {
            Field::Single(field) if field.is_be_wrapper() => {
                quote_spanned!(span=> #self_token #name.get())
            }
            _ => quote_spanned!(span=> &#self_token #name),
        }
    }

    /// The type that represents this field in a view struct.
    pub fn view_field_decl(&self) -> proc_macro2::TokenStream {
        let name = self.name();
        match self {
            Field::Single(item) => {
                let typ = item.typ.view_field_tokens();
                let span = typ.span();
                let allow_dead = item.hidden.as_ref().map(|hidden| {
                    let span = hidden.span();
                    quote_spanned!(span=> #[allow(dead_code)])
                });
                quote_spanned!(span=> #allow_dead #name: zerocopy::LayoutVerified<&'a [u8], #typ>)
            }
            Field::Array(array) if array.variable_size.is_none() => {
                let typ = array.inner_typ.view_field_tokens();
                let span = typ.span();
                quote_spanned!(span=> #name: zerocopy::LayoutVerified<&'a [u8], [#typ]>)
            }
            _ => panic!("variable arrays are not handled yet, you shouldn't be calling this"),
        }
    }

    pub fn to_owned_expr(&self) -> Result<proc_macro2::TokenStream, syn::Error> {
        match self {
            Field::Single(field) => field.to_owned_expr(),
            Field::Array(field) => field.to_owned_expr(),
        }
    }

    pub fn font_write_expr(&self) -> proc_macro2::TokenStream {
        match self {
            Field::Single(field) => field.font_write_expr(),
            Field::Array(field) => field.font_write_expr(),
        }
    }
}

impl FieldType {
    pub fn is_offset(&self) -> bool {
        matches!(self, FieldType::Offset { .. })
    }

    pub fn view_field_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Offset {
                offset_type,
                target_type,
            } => match target_type {
                //Some(target) => quote!(BigEndian<#offset_type<#target<'a>>>),
                Some(_) => quote!(BigEndian<#offset_type>),
                None => quote!(BigEndian<#offset_type>),
            },
            Self::Other { typ } => typ.to_token_stream(),
            Self::Scalar { typ } => quote!(BigEndian<#typ>),
        }
    }

    fn compile_type(&self) -> proc_macro2::TokenStream {
        match self {
            FieldType::Scalar { typ } => typ.into_token_stream(),
            FieldType::Other { typ } => typ.into_token_stream(),
            FieldType::Offset {
                offset_type,
                target_type,
            } => {
                let target = target_type
                    .as_ref()
                    .map(|t| t.into_token_stream())
                    .unwrap_or_else(|| quote!(Box<dyn FontWrite>));
                let offset = if offset_type == "Offset16" {
                    syn::Ident::new("OffsetMarker16", offset_type.span())
                } else if offset_type == "Offset24" {
                    syn::Ident::new("OffsetMarker24", offset_type.span())
                } else if offset_type == "Offset32" {
                    syn::Ident::new("OffsetMarker32", offset_type.span())
                } else {
                    panic!("this should already be validated?");
                };
                quote!(#offset<#target>)
            }
        }
    }
}

impl SingleField {
    pub fn is_be_wrapper(&self) -> bool {
        !matches!(&self.typ, FieldType::Other { .. })
    }

    /// The return type of a getter of this type.
    ///
    /// this is about returning T for BigEndian<T>, but returning &T for some
    /// non-scalar T.
    fn cooked_type_tokens(&self) -> proc_macro2::TokenStream {
        match &self.typ {
            //FieldType::Offset {
            //offset_type,
            //target_type: Some(targ),
            //} => quote!(#offset_type<#targ>),
            FieldType::Offset { offset_type, .. } => quote!(#offset_type),
            FieldType::Scalar { typ, .. } => quote!(#typ),
            FieldType::Other { typ } => quote!(&#typ),
        }
    }

    fn to_owned_expr(&self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let name = &self.name;
        match &self.typ {
            FieldType::Scalar { .. } => Ok(quote!(self.#name())),
            FieldType::Other { .. } => Ok(quote!(self.#name().to_owned_obj(offset_data)?)),
            FieldType::Offset {
                offset_type,
                target_type,
            } => match &target_type {
                Some(target_type) => Ok(
                    //TODO: this is where we want a 'from' type.
                    quote!(OffsetMarker::new_maybe_null(self.#name().read::<super::#target_type>(offset_data).and_then(|obj| obj.to_owned_obj(offset_data)))),
                ),
                None => Err(syn::Error::new(
                    offset_type.span(),
                    "offsets with unknown types require custom FromObjRef impls",
                )),
            },
        }
    }

    pub fn font_write_expr(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        match &self.typ {
            FieldType::Scalar { typ } => match &self.compute {
                None => quote!(self.#name.write_into(writer);),
                Some(Compute::Len(fld)) => {
                    quote!(#typ::try_from(self.#fld.len()).unwrap().write_into(writer); )
                }
                Some(Compute::Literal(lit)) => quote!( (#lit as #typ).write_into(writer); ),
                Some(Compute::With(path)) => quote! {
                    let #name: #typ = #path(self);
                    #name.write_into(writer);
                },
            },
            _ => quote!(self.#name.write_into(writer);),
        }
    }
}

impl ArrayField {
    pub fn to_owned_expr(&self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let name = &self.name;
        let map_impl = match &self.inner_typ {
            FieldType::Scalar { .. } => quote!(Some(item.get())),
            FieldType::Other { .. } => quote!(item.to_owned_obj(offset_data)),
            //TODO: also a from type here
            FieldType::Offset {
                target_type: Some(target_type),
                ..
            } => {
                quote!(Some(OffsetMarker::new_maybe_null(item.get().read::<super::#target_type>(offset_data).and_then(|obj| obj.to_owned_obj(offset_data)))))
            }
            FieldType::Offset { offset_type, .. } => {
                return Err(syn::Error::new(
                    offset_type.span(),
                    "offsets with unknown types require custom FromObjRef impls",
                ))
            }
        };

        Ok(quote! {
            self.#name().iter().map(|item| #map_impl).collect::<Option<Vec<_>>>()?
        })
    }

    pub fn font_write_expr(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        quote!(self.#name.write_into(writer);)
    }
}
impl SingleItem {
    /// `true` if this contains offsets or fields with lifetimes.
    pub fn has_references(&self) -> bool {
        self.offset_host.is_some() || self.has_field_with_lifetime()
    }

    pub fn has_field_with_lifetime(&self) -> bool {
        self.fields.iter().any(|x| x.requires_lifetime())
    }

    pub fn contains_offsets(&self) -> bool {
        self.fields.iter().any(Field::is_offset)
    }

    fn validate(&self) -> Result<(), syn::Error> {
        // check for lifetime
        let needs_lifetime = self.has_references();
        if needs_lifetime && self.lifetime.is_none() {
            let msg = format!(
                "object containing array or offset requires lifetime param ({}<'a>)",
                self.name
            );
            return Err(syn::Error::new(self.name.span(), &msg));
        } else if !needs_lifetime && self.lifetime.is_some() {
            return Err(syn::Error::new(
                self.lifetime.as_ref().unwrap().span(),
                "only objects containing arrays or offsets require lifetime",
            ));
        }

        // check that fields referenced in #count annotations are sane
        for (field_idx, ident) in self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(i, fld)| fld.as_array().map(|arr| (i, arr)))
            .flat_map(|(i, x)| x.count.iter_input_fields().map(move |id| (i, id)))
        {
            match self.fields.iter().position(|fld| fld.name() == ident) {
                Some(x) if x < field_idx => (),
                Some(_) => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "field must occur before it can be referenced",
                    ))
                }
                None if self.init.iter().any(|field| &field.0 == ident) => (),
                None => return Err(syn::Error::new(ident.span(), "unknown field")),
            }
        }

        // ensure #[count_all] is last, if it exists
        for (i, field) in self.fields.iter().enumerate() {
            if let Some(array) = field.as_array() {
                if let Count::All(all) = &array.count {
                    if i != self.fields.len() - 1 {
                        return Err(syn::Error::new(
                            all.span(),
                            "#[count_all] only valid on last item",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

impl ItemGroup {
    fn new(
        name: syn::Ident,
        lifetime: Option<syn::Lifetime>,
        variants: Vec<Variant>,
        attrs: ItemAttrs,
    ) -> Result<Self, syn::Error> {
        if let Some(format_typ) = attrs.format {
            Ok(Self {
                docs: attrs.docs,
                name,
                lifetime,
                variants,
                format_typ,
                generate_getters: attrs.generate_getters,
                offset_host: attrs.offset_host,
            })
        } else {
            Err(syn::Error::new(
                name.span(),
                "all enum groups require #[format(..)] attribute",
            ))
        }
    }
}

impl RawEnum {
    fn new(
        name: syn::Ident,
        variants: Vec<RawVariant>,
        attrs: ItemAttrs,
    ) -> Result<Self, syn::Error> {
        let repr = attrs.repr.ok_or_else(|| {
            syn::Error::new(
                name.span(),
                "raw enumerations require repr annotation (like: #[repr(u16)])",
            )
        })?;
        Ok(RawEnum {
            docs: attrs.docs,
            repr,
            variants,
            name,
        })
    }
}

impl BitFlags {
    fn new(
        name: syn::Ident,
        variants: Vec<RawVariant>,
        attrs: ItemAttrs,
    ) -> Result<Self, syn::Error> {
        let type_ = attrs.flags.ok_or_else(|| {
            syn::Error::new(name.span(), "flags require annotation like #[flags(u16)]")
        })?;
        Ok(BitFlags {
            docs: attrs.docs,
            type_,
            variants,
            name,
        })
    }
}

impl SimpleUse {
    pub fn compile_use_stmt(&self) -> syn::Path {
        let len = self.0.segments.len();
        let mut path = self.0.clone();
        path.segments.insert(
            len - 1,
            syn::PathSegment::from(syn::Ident::new("compile", path.span())),
        );
        path
    }
}

fn get_optional_attributes(input: ParseStream) -> Result<Vec<syn::Attribute>, syn::Error> {
    let mut result = Vec::new();
    while input.lookahead1().peek(Token![#]) {
        result.extend(Attribute::parse_outer(input)?);
    }
    Ok(result)
}

fn get_use_statements(input: ParseStream) -> Result<Vec<SimpleUse>, syn::Error> {
    let mut result = Vec::new();
    while input.peek(Token![use]) {
        let item = SimpleUse::parse(input)?;
        result.push(item);
    }
    Ok(result)
}

fn get_optional_module_docs(input: ParseStream) -> Result<Vec<syn::Attribute>, syn::Error> {
    let mut result = Vec::new();
    while input.peek(Token![#]) && input.peek2(Token![!]) {
        let item = Attribute::parse_inner(input).map_err(|e| {
            syn::Error::new(e.span(), format!("error parsing inner attribute: '{}'", e))
        })?;
        for attr in &item {
            if !attr.path.is_ident("doc") {
                return Err(syn::Error::new_spanned(
                    attr,
                    "only doc attributes are supported",
                ));
            }
        }
        result.extend(item);
    }

    Ok(result)
}

fn parse_field_type(input: &syn::Path) -> Result<FieldType, syn::Error> {
    let last = input.segments.last().expect("do zero-length paths exist?");
    if last.ident != "BigEndian" {
        return Ok(FieldType::Other { typ: input.clone() });
    }
    let inner = get_single_generic_type_arg(&last.arguments).ok_or_else(|| {
        syn::Error::new(last.ident.span(), "expected single generic type argument")
    })?;
    let last = inner.segments.last().unwrap();
    if ["Offset16", "Offset24", "Offset32"].contains(&last.ident.to_string().as_str()) {
        let target_type = get_single_generic_type_arg(&last.arguments)
            .map(|p| p.segments.last().unwrap().ident.clone());
        return Ok(FieldType::Offset {
            target_type,
            offset_type: last.ident.clone(),
        });
    }
    if last.arguments.is_empty() {
        Ok(FieldType::Scalar {
            typ: last.ident.clone(),
        })
    } else {
        Err(syn::Error::new(last.span(), "unexpected arguments"))
    }
}

fn get_single_generic_type_arg(input: &syn::PathArguments) -> Option<syn::Path> {
    match input {
        syn::PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
            let arg = args.args.last().unwrap();
            if let syn::GenericArgument::Type(syn::Type::Path(path)) = arg {
                if path.qself.is_none() && path.path.segments.len() == 1 {
                    return Some(path.path.clone());
                }
            }
            None
        }
        _ => None,
    }
}

fn ensure_single_lifetime(input: &syn::Path) -> Result<Option<syn::Lifetime>, syn::Error> {
    match input.segments.last().map(|seg| &seg.arguments) {
        Some(syn::PathArguments::AngleBracketed(args)) => {
            let mut iter = args.args.iter().filter_map(|arg| match arg {
                syn::GenericArgument::Lifetime(arg) => Some(arg),
                _ => None,
            });
            let result = iter.next();
            if let Some(extra_lifetime) = iter.next() {
                Err(syn::Error::new(
                    extra_lifetime.span(),
                    "at most a single lifetime \"'a\" is supported",
                ))
            } else {
                Ok(result.cloned())
            }
        }
        Some(syn::PathArguments::Parenthesized(args)) => Err(syn::Error::new(
            args.span(),
            "whatever this is trying to do, we definitely do not support it",
        )),
        None | Some(syn::PathArguments::None) => Ok(None),
    }
}

/// Ensure types have at most a single lifetime param, "'a".
fn validate_lifetime(input: ParseStream) -> Result<Option<syn::Lifetime>, syn::Error> {
    let generics = input.parse::<syn::Generics>()?;
    if generics.const_params().count() + generics.type_params().count() > 1 {
        return Err(syn::Error::new(
            generics.span(),
            "font types should contain at most a single lifetime",
        ));
    }
    if let Some(lifetime) = generics.lifetimes().nth(1) {
        return Err(syn::Error::new(
            lifetime.span(),
            "tables can contain at most a single lifetime",
        ));
    }

    let result = generics.lifetimes().next().map(|lt| &lt.lifetime).cloned();
    Ok(result)
}

//impl ToTokens for FieldType {
//fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
//match self {
//FieldType::Offset {
//offset_type,
//target_type: Some(targ),
//} => tokens.extend(quote!(BigEndian<#offset_type<#targ>>)),
//FieldType::Offset { offset_type, .. } => tokens.extend(quote!(BigEndian<#offset_type>)),
//FieldType::Scalar { typ } => tokens.extend(quote!(BigEndian<#typ>)),
//FieldType::Other { typ } => typ.to_tokens(tokens),
//}
//}
//}

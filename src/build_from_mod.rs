use crate::{build, render, types::*};
use syn::*;

pub fn build_contexts_with_module(
    tokens: proc_macro::TokenStream,
    accounts: YamlAccounts,
) -> Result<proc_macro::TokenStream> {
    let mut item_mod: ItemMod = parse2(tokens.into()).expect("Failed to parse main program module");

    let (_, items) = item_mod.content.as_mut().expect("module is empty");

    let mut extra_items = vec![];

    let q: Attribute = parse_quote! { #[context] };
    for item in items {
        if let Item::Struct(item_struct) = item {
            if let Some(attr) = item_struct.attrs.iter_mut().find(|a| *a == &q) {
                *attr = parse_quote! { #[derive(Accounts)] };
            } else {
                continue;
            }
            extra_items.extend(complete_struct(
                &accounts,
                ContextBuilder::new(item_struct),
            )?);
        }
    }

    extra_items.extend(
        render::compile_wrapper_types()
            .into_iter()
            .map(|t| syn::parse_quote! { #t }),
    );
    item_mod.content.as_mut().map(|c| c.1.extend(extra_items));

    Ok(quote::quote! { #item_mod }.into())
}

fn complete_struct(account_defs: &YamlAccounts, item: &mut ContextBuilder) -> Result<Vec<Item>> {

    let spec = read_struct(item)?;

    let BuiltContext { mut accounts, .. } = build::build_context(&spec, account_defs)?;
    let accounts_clone = accounts.clone();

    //eprintln!("{}", item.ident);

    /*
     * The problem is that account fields without attributes are not updated
     */

    /*
     * Update attributes on existing accounts
     */
    for id in item.iter_fields() {
        //eprintln!("update attributes on: {}", id);
        let mut account = accounts.remove(&id).unwrap();
        if let Some(tokens) = render::compile_metas(&mut account) {
            //eprintln!("UPDATE attrs: {:?}", tokens);
            item.set_attr(id, syn::parse_quote! { #tokens });
        }
    }

    /*
     * Insert remaining accounts
     */
    for (id, account) in accounts.iter_mut() {
        if let Fields::Named(fields_named) = &mut item.fields {
            //eprintln!("create field: {}", id);
            let field = create_field(id.clone(), account);
            fields_named.named.push(field);
        }
    }

    let items = render::compile_context_wrapper((item.ident.clone(), accounts_clone));

    Ok(items
        .into_iter()
        .map(|i| syn::parse_quote! { #i })
        .collect())
}

fn create_field(id: Ident, account: &mut BuiltAccount) -> Field {
    let ty = render::compile_type(account);
    let metas = render::compile_metas(account);
    //eprintln!("METAS: {:?}", metas);
    Field {
        attrs: Vec::from_iter(metas.map(|m| syn::parse_quote! { #m })),
        vis: syn::parse_quote!(pub),
        ident: Some(id),
        colon_token: None,
        ty: syn::parse_quote! { #ty },
    }
}

fn read_struct(item: &ContextBuilder) -> Result<Vec<ContextProp>> {
    let mut out = Vec::new();

    let a: Path = parse_quote! { account };
    for field in &item.fields {
        let args = match field.attrs.iter().find(|attr| attr.path == a) {
            Some(attr) => parse::Parser::parse2(
                |input: parse::ParseStream| {
                    let content;
                    parenthesized!(content in input);
                    let p = content.parse_terminated::<AccountArg, Token![,]>(|p| p.parse())?;
                    Ok(p.into_iter().collect())
                },
                attr.tokens.clone(),
            )?,
            None => vec![],
        };
        let id = field.ident.clone().unwrap();
        out.push(ContextProp::Account { name: id, args });
    }

    Ok(out)
}

struct ContextBuilder(ItemStruct);
impl std::ops::Deref for ContextBuilder {
    type Target = ItemStruct;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for ContextBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl ContextBuilder {
    pub fn new(item: &mut ItemStruct) -> &mut ContextBuilder {
        unsafe { std::mem::transmute(item) }
    }
    pub fn iter_fields(&mut self) -> Vec<Ident> {
        self.0
            .fields
            .iter()
            .filter_map(|f| f.ident.clone())
            .collect()
    }
    pub fn set_attr(&mut self, id: Ident, attrs: Attribute) {
        let a: Path = parse_quote! { account };
        let field = self
            .0
            .fields
            .iter_mut()
            .find(|field| Some(id.clone()) == field.ident);
        if let Some(field) = field {
            if let Some(attr) = field.attrs.iter_mut().find(|attr| attr.path == a) {
                *attr = attrs;
            } else {
                field.attrs.push(attrs)
            }
        }
    }
}

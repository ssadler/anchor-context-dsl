use crate::types::parse_error;
use syn::*;

pub fn parse_at_level_fn<L: IndentLevel>(
    input: parse::ParseStream,
    mut f: impl FnMut() -> Result<()>,
) -> Result<()> {
    input.parse::<L>()?;
    f()?;
    while input.fork().parse::<L>().is_ok() {
        input.parse::<L>()?;
        f()?;
    }
    Ok(())
}

pub fn parse_at_level<I: IndentLevel, T: ParseIndented<I>>(
    input: parse::ParseStream,
) -> Result<Vec<T>> {
    let mut out = vec![];
    parse_at_level_fn::<I>(input, || Ok(out.push(parse_indented(input)?)))?;
    Ok(out)
}

struct YamlListItem<T>(T);
impl<L: IndentLevel, T: ParseIndented<L>> ParseIndented<L> for YamlListItem<T> {
    fn parse_indented(input: parse::ParseStream) -> Result<Self> {
        input.parse::<Token![-]>()?;
        Ok(YamlListItem(parse_indented(input)?))
    }
}
pub fn parse_yaml_array<I: IndentLevel, T: ParseIndented<I::Next>>(
    input: parse::ParseStream,
) -> Result<Vec<T>> {
    if input.peek(syn::token::Bracket) {
        let content;
        syn::bracketed!(content in input);
        let items = content.parse_terminated::<T, Token![,]>(parse_indented)?;
        Ok(items.into_iter().collect())
    } else {
        let out: Vec<YamlListItem<T>> = parse_at_level::<I::Next, _>(input)?;
        Ok(out.into_iter().map(|YamlListItem(t)| t).collect())
    }
}

/*
 */
pub struct IndentedToken<T: parse::Parse>(pub T);
impl<L: IndentLevel, T: parse::Parse> ParseIndented<L> for IndentedToken<T> {
    fn parse_indented(input: parse::ParseStream) -> Result<Self> {
        Ok(IndentedToken(input.parse()?))
    }
}
pub fn parse_indented_token<L: IndentLevel, O: parse::Parse>(
    input: parse::ParseStream,
) -> Result<O> {
    input.parse() // O is syn::Type
}
pub fn parse_yaml_token_array<I: IndentLevel, T: parse::Parse>(
    input: parse::ParseStream,
) -> Result<Vec<T>> {
    let out = parse_yaml_array::<I, IndentedToken<T>>(input)?;
    Ok(out.into_iter().map(|IndentedToken(t)| t).collect())
}

pub trait ParseIndented<I: IndentLevel>: Sized {
    fn parse_indented(input: parse::ParseStream) -> Result<Self>;
}

pub fn parse_indented<L: IndentLevel, O: ParseIndented<L>>(input: parse::ParseStream) -> Result<O> {
    O::parse_indented(input)
}

pub fn parse_indented_next_level<L: IndentLevel, O: ParseIndented<L::Next>>(
    input: parse::ParseStream,
) -> Result<O> {
    O::parse_indented(input)
}

pub struct Indent<const LEVEL: usize>;

impl<const LEVEL: usize> parse::Parse for Indent<LEVEL> {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        if LEVEL > 16 {
            Err(syn::Error::new(input.span(), "Indent level max is 16"))?;
        }

        let id = input.parse::<syn::Ident>()?;
        let valid = id.to_string().starts_with("__indent_");
        if !valid {
            Err(parse_error!(input.span(), "expected indent"))?
        }

        let l: usize = id.to_string()[9..].parse().unwrap();
        if l != LEVEL {
            let msg = format!("Wrong indent level (expected {}, got {})", LEVEL, l);
            Err(syn::Error::new(input.span(), msg))?
        }

        Ok(Indent)
    }
}

pub trait IndentLevel: parse::Parse {
    type Next: IndentLevel;
    //fn get_level() -> usize;
}

macro_rules! define_indent_levels {
    ($a:literal $b:literal $($rest:literal)*) => {
        impl IndentLevel for Indent<$a> {
            type Next = Indent<$b>;
            //fn get_level() -> usize { $a }
        }
        define_indent_levels!($b $($rest)*);
    };
    ($a:literal) => {
        impl IndentLevel for Indent<$a> {
            type Next = Indent<0>;
            //fn get_level() -> usize { 1000 }
        }
    }
}
define_indent_levels!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17);

pub fn create_indented_tokenstream(
    tokens: proc_macro::TokenStream,
) -> Option<proc_macro::TokenStream> {
    // Expect a single Group token
    let mut iter = tokens.into_iter();
    let group = match iter.next() {
        Some(proc_macro::TokenTree::Group(g)) => g,
        _ => panic!("Expected a single Group token"),
    };

    let source = match group.span().source_text() {
        Some(s) => s,
        None => return None,
    };
    let inner_tokens = group.stream();

    let mut result = Vec::new();
    let mut current_pos = 0;

    for token in inner_tokens {
        let text = match &token {
            // required because spans sometimes encompass multiple tokens which confuses
            // things e.g. for lifetimes
            proc_macro::TokenTree::Punct(p) if p.as_char() == '\'' => Some("'".to_string()),
            proc_macro::TokenTree::Ident(i) => Some(i.to_string()),
            _ => token.span().source_text(),
        };
        if let Some(text) = text {
            if let Some(pos) = source[current_pos..].find(&text) {
                let abs_pos = current_pos + pos;
                let prefix = &source[current_pos..abs_pos];

                if let Some(id) = parse_indent(prefix, &token) {
                    result.push(id.into());
                }

                result.push(token);
                current_pos = abs_pos + text.len();
            }
        }
    }

    Some(result.into_iter().collect())
}

fn parse_indent(s: &str, token: &proc_macro::TokenTree) -> Option<proc_macro::Ident> {
    //eprintln!("{:?}\n{:?}\n\n", s, token);

    let span = token.span();
    let orig = s;

    let mut s = s;
    if s.chars().next() == Some('{') {
        s = &s[1..]; // .to_string();
    }

    let mut seen = false;
    loop {
        match s.find(|c| c == '\n') {
            Some(pos) => {
                seen = true;
                s = &s[pos + 1..]
            }
            None => break,
        }
    }
    //eprintln!("seen newline: {}", seen);
    if !seen {
        return None;
    }

    let spaces = s.chars().take_while(|&c| c == ' ').count();
    if spaces % 2 == 1 {
        panic!("indent spaces should be multiple of 2: {:?}", orig);
    }
    let s = format!("__indent_{}", spaces >> 1);
    let ss = s.as_str();
    return Some(proc_macro::Ident::new(ss, span));
}

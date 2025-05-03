
use syn::*;
use crate::types::parse_error;

pub fn parse_many_indented_iter<I: IndentLevel, T: ParseIndented<I>>(
    input: parse::ParseStream
) -> impl Iterator<Item = Result<T>> + '_ {
    std::iter::from_fn(move || {
        match input.fork().parse::<I>() {
            Ok(_) => {
                // Advance the stream only if the fork was successful
                if let Err(e) = input.parse::<I>() {
                    return Some(Err(e));
                }
                Some(T::parse_indented(input))
            },
            Err(_) => None,
        }
    })
}

pub fn parse_many_indented_fn<L: IndentLevel>(
    input: parse::ParseStream,
    mut f: impl FnMut() -> Result<()>
) -> Result<()> {
    input.parse::<L>()?;
    f()?;
    loop {
        match input.fork().parse::<L>() {
            Ok(_) => {
                input.parse::<L>()?;
                f()?;
            },
            _ => break
        }
    }
    Ok(())
}

pub fn parse_many_indented<I: IndentLevel, T: ParseIndented<I>>(
    input: parse::ParseStream
) -> Result<Vec<T>> {
    input.parse::<I>()?;
    let mut out = vec![T::parse_indented(input)?];
    loop {
        match input.fork().parse::<I>() {
            Ok(_) => {
                input.parse::<I>()?;
                out.push(T::parse_indented(input)?);
            },
            _ => break
        }
    }
    Ok(out)
}

struct YamlListItem<T>(T);
impl<L: IndentLevel, T: ParseIndented<L>> ParseIndented<L> for YamlListItem<T> {
    fn parse_indented(input: parse::ParseStream) -> Result<Self> {
        input.parse::<Token![-]>()?;
        Ok(YamlListItem(T::parse_indented(input)?))
    }
}
pub fn parse_yaml_array<I: IndentLevel, T: ParseIndented<I>>(input: parse::ParseStream) -> Result<Vec<T>> {
    let out: Vec<YamlListItem<T>> = parse_many_indented(input)?;
    Ok(out.into_iter().map(|YamlListItem(t)| t).collect())
}

/*
 */
pub struct IndentedToken<T: parse::Parse>(pub T);
impl<L: IndentLevel, T: parse::Parse> ParseIndented<L> for IndentedToken<T> {
    fn parse_indented(input: parse::ParseStream) -> Result<Self> {
        Ok(IndentedToken(input.parse()?))
    }
}
pub fn parse_indented_token<L: IndentLevel, O: parse::Parse>(input: parse::ParseStream) -> Result<O> {
    input.parse()
}
pub fn parse_yaml_token_array<I: IndentLevel, T: parse::Parse>(input: parse::ParseStream) -> Result<Vec<T>> {
    let out = parse_yaml_array::<I, IndentedToken<T>>(input)?;
    Ok(out.into_iter().map(|IndentedToken(t)| t).collect())
}


pub trait ParseIndented<L: IndentLevel>: Sized {
    fn parse_indented(input: parse::ParseStream) -> Result<Self>;
}

pub fn parse_indented<L: IndentLevel, O: ParseIndented<L>>(input: parse::ParseStream) -> Result<O> {
    O::parse_indented(input)
}

pub struct Indent<const LEVEL: usize>;

impl<const LEVEL: usize> parse::Parse for Indent<LEVEL> {
    fn parse(input: parse::ParseStream) -> Result<Self> {

        if LEVEL > 16 { Err(syn::Error::new(input.span(), "Indent level max is 16"))?; }

        let id = input.parse::<syn::Ident>()?;
        let valid = id.to_string().starts_with("__indent_");
        if !valid { Err(parse_error!(input.span(), "expected indent"))? }

        let l: usize = id.to_string()[9..].parse().unwrap();
        if l != LEVEL { Err(syn::Error::new(input.span(), "wrong level"))? }

        Ok(Indent)
    }
}

pub trait IndentLevel: parse::Parse {
    type Next: IndentLevel;
}

macro_rules! define_indent_levels {
    ($a:literal $b:literal $($rest:literal)*) => {
        impl IndentLevel for Indent<$a> { type Next = Indent<$b>; }
        define_indent_levels!($b $($rest)*);
    };
    ($a:literal) => { impl IndentLevel for Indent<$a> { type Next = Indent<0>; } }
}
define_indent_levels!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17);




pub fn create_indented_tokenstream(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Expect a single Group token
    let mut iter = tokens.into_iter();
    let group = match iter.next() {
        Some(proc_macro::TokenTree::Group(g)) => g,
        _ => panic!("Expected a single Group token"),
    };

    let source = group.span().source_text().expect("source_text unavailable");
    let inner_tokens = group.stream();

    fn parse_indent(s: &str, span: proc_macro::Span) -> Option<proc_macro::Ident> {

        let orig = s;

        let mut s = s; // .to_string();
        if s.chars().next() == Some('{') {
            s = &s[1..]; // .to_string();
        }

        let mut seen = false;
        loop {
            match s.find(|c| c == '\n') {
                Some(pos) => { seen = true; s = &s[pos+1..] },
                None => break
            }
        }
        if !seen {
            return None
        }

        let spaces = s.chars().take_while(|&c| c == ' ').count();
        if spaces % 2 == 1 {
            panic!("indent spaces should be multiple of 2: {:?}", orig);
        }
        let s = format!("__indent_{}", spaces>>1);
        let ss = s.as_str();
        return Some(proc_macro::Ident::new(ss, span))
    }

    let mut result = Vec::new();
    let mut current_pos = 0;

    for token in inner_tokens {
        if let Some(text) = token.span().source_text() {
            if let Some(pos) = source[current_pos..].find(&text) {
                let abs_pos = current_pos + pos;
                let prefix = &source[current_pos..abs_pos];

                if let Some(id) = parse_indent(prefix, token.span()) {
                    result.push(id.into());
                }

                result.push(token);
                current_pos = abs_pos + text.len();
            }
        }
    }
    
    result.into_iter().collect()
}



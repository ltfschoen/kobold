use proc_macro2::TokenStream as QuoteTokens;

mod parser;

pub use parser::Parser;

#[derive(Debug)]
pub struct Field {
    pub typ: QuoteTokens,
    pub name: QuoteTokens,
    pub expr: QuoteTokens,
}

#[derive(Debug)]
pub enum Node {
    Element(Element),
    Text(String),
    Expression, // variable arg, node ref arg
    Fragment(Vec<Node>),
}

#[derive(Debug)]
pub struct Element {
    pub tag: String,
    pub props: Vec<(String, QuoteTokens)>,
    pub children: Vec<Node>,
}

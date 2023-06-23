mod parser;
mod sequence;

pub struct Expression<'a, T> {
    preamble: &'a str,
    /// An IntoIterator<Item = Result<AsRef<str>, _>>
    generator: T,
    postscript: &'a str,
}

/// {a,b,c}
struct List(Vec<Expression>);

use chumsky::prelude::*;

#[derive(Clone, Debug)]
pub enum Sexp<'a> {
    Invalid,
    Symbol(&'a str),
    StringLiteral(&'a str),
    IntLiteral(&'a str),
    HexIntLiteral(&'a str),
    FloatLiteral(&'a str),
    List(Vec<Self>)
}

fn parse_escape<'src>() -> impl Parser<'src, &'src str, char, extra::Err<Simple<'src, char>>> + Copy {
    just('\\')
        .ignore_then(choice((
            just('\\'),
            just('"'),
            just('n').to('\n'),
            just('t').to('\t')
        )))
}

fn parse_string<'src>() -> impl Parser<'src, &'src str, &'src str, extra::Err<Simple<'src, char>>> + Copy {
    none_of("\\\"")
        .or(parse_escape())
        .repeated()
        .to_slice()
        .delimited_by(just('\"'), just('\"'))
        .then_ignore(one_of(" \n\t)").repeated().at_least(1).rewind())
        .padded()
}

fn parse_int<'src>() -> impl Parser<'src, &'src str, &'src str, extra::Err<Simple<'src, char>>> + Copy {
    just('-').or_not()
        .then(text::digits(10))
        .to_slice()
        .then_ignore(one_of(" \n\t)").repeated().at_least(1).rewind())
        .padded()
}

fn parse_hexint64<'src>() -> impl Parser<'src, &'src str, &'src str, extra::Err<Simple<'src, char>>> + Copy {
    just("0x").or_not()
        .then(text::digits(16).exactly(8))
        .then(just('_'))
        .then(text::digits(16).exactly(8))
        .then(just('_'))
        .then(text::digits(16).exactly(8))
        .then(just('_'))
        .then(text::digits(16).exactly(8))
        .to_slice()
        .then_ignore(one_of(" \n\t)").repeated().at_least(1).rewind())
        .padded()
}

fn parse_float<'src>() -> impl Parser<'src, &'src str, &'src str, extra::Err<Simple<'src, char>>> + Copy {
    just('-').or_not()
        .then(text::digits(10))
        .then(just('.'))
        .then(text::digits(10))
        .to_slice()
        .then_ignore(one_of(" \n\t)").repeated().at_least(1).rewind())
        .padded()
}

fn parse_symbol<'src>() -> impl Parser<'src, &'src str, &'src str, extra::Err<Simple<'src, char>>> + Copy {
    none_of(" \"()\n\t")
        .repeated()
        .at_least(1)
        .to_slice()
        .then_ignore(one_of(" \n\t)").repeated().at_least(1).rewind())
        .padded()
}

pub fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Sexp<'a>>, extra::Err<Simple<'a, char>>> {
    use Sexp::*;
    recursive(|bf| {
        choice((
            parse_string()
                .map(StringLiteral),
            parse_int()
                .map(IntLiteral),
            parse_hexint64()
                .map(HexIntLiteral),
            parse_float()
                .map(FloatLiteral),
            parse_symbol()
                .map(Symbol),
        ))
        .or(bf.delimited_by(just('('), just(')')).padded().map(List))
        .recover_with(via_parser(nested_delimiters('(', ')', [], |_| Invalid)))
        .repeated()
        .collect()
    })
}

pub fn pretty_print(sexps: &Vec<Sexp>) {
    for sexp in sexps {
        match sexp {
            Sexp::Invalid => print!("Inv "),
            Sexp::Symbol(symbol) => print!("ʆ{} ", symbol),
            Sexp::StringLiteral(str_literal) => print!("\"{}\" ", str_literal),
            Sexp::IntLiteral(num_literal) => print!("ŋ{} ", num_literal),
            Sexp::HexIntLiteral(num_literal) => print!("ŋ{} ", num_literal),
            Sexp::FloatLiteral(num_literal) => print!("ŋ{} ", num_literal),
            Sexp::List(sexps) => {
                print!("(");
                pretty_print(sexps);
                print!(") ");
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape() {
        let parser = parse_escape();

        assert_eq!(parser.parse("\\\\").unwrap(), '\\');
        assert_eq!(parser.parse("\\\"").unwrap(), '"');
        assert_eq!(parser.parse("\\n").unwrap(), '\n');
        assert_eq!(parser.parse("\\t").unwrap(), '\t');
    }

    #[test]
    fn string() {
        let parser = parse_string();

        assert_eq!(parser.parse("\"\\\\\" ").unwrap(), "\\\\");
        assert_eq!(parser.parse("\"\\\"\" ").unwrap(), "\\\"");
        assert_eq!(parser.parse("\"\\n\" ").unwrap(), "\\n");
        assert_eq!(parser.parse("\"\\t\" ").unwrap(), "\\t");
        assert_eq!(parser.parse("\"this is a normal string\" ").unwrap(), "this is a normal string");
    }

    #[test]
    fn int() {
        let parser = parse_int();

        assert_eq!(parser.parse("12345 ").unwrap(), "12345");
    }

    #[test]
    fn hexint64() {
        let parser = parse_hexint64();

        assert_eq!(parser.parse("0xdeadbeef_beefdead_44552255_12345678 ").unwrap(), "0xdeadbeef_beefdead_44552255_12345678");
    }

    #[test]
    fn float() {
        let parser = parse_float();

        assert_eq!(parser.parse("-123.123456 ").unwrap(), "-123.123456");
        assert_eq!(parser.parse("321.6543210 ").unwrap(), "321.6543210");
    }

    #[test]
    fn symbol() {
        let parser = parse_symbol();

        assert_eq!(parser.parse("97-something-bla-9_the \n").unwrap(), "97-something-bla-9_the");
    }

    #[test]
    fn kicad_sexp() {
        let parser = parser();
        let empty_sch_file = include_str!("../../reference-files/empty/empty.kicad_sch");
        let empty_pcb_file = include_str!("../../reference-files/empty/empty.kicad_pcb");

        let result = parser.parse(empty_sch_file);
        assert_eq!(result.has_errors(), false);
        assert_eq!(result.has_output(), true);

        let result = parser.parse(empty_pcb_file);
        assert_eq!(result.has_errors(), false);
        assert_eq!(result.has_output(), true);
    }
}

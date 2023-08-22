use environment::Environment;
use ir::Context;
use neoglot_lib::{regex::*, lexer::*};
use validator::verify;

mod parser;
mod validator;
mod environment;
mod ir;

#[derive(Debug, Hash, PartialOrd, PartialEq, Eq, Copy, Clone)]
pub enum TokenType{
    Ident,

    Int, Float, Hex, Bool,

    LParen, RParen,
    LBracket, RBracket,

    Comma, Colon, SemiColon,

    If, Else, While,
    Travel, Subcanvas,
    Def, Return,

    Plus, Minus, Mul,
    Div, Mod,

    Eq,

    And, Or,
    DoubleEq, GT, LT,
    Not, NotEq, GTEq,
    LTEq,

    SingleComment
}

impl Symbol for TokenType{}
impl TokenKind for TokenType{}

fn main() {
    test_parse(include_str!("test.pprs").to_string(), "C:/Users/Utilisateur/papyrus/src/test.pprs")
}

fn init_lexer(lexer:&mut Lexer<TokenType>){
    let ident_regex = Regex::new()
        .then(RegexElement::AnyOf(vec![
            RegexElement::Set('a', 'z', Quantifier::Exactly(1)),
            RegexElement::Set('A', 'Z', Quantifier::Exactly(1))
        ]))
        .then(RegexElement::Group(vec![
            RegexElement::AnyOf(vec![
                RegexElement::Item('_', Quantifier::Exactly(1)),
                RegexElement::Set('a', 'z', Quantifier::Exactly(1)),
                RegexElement::Set('A', 'Z', Quantifier::Exactly(1)),
                RegexElement::Set('0', '9', Quantifier::Exactly(1))
            ])
        ], Quantifier::ZeroOrMany));

    let float_regex = Regex::new()
        .then(RegexElement::Item('-', Quantifier::ZeroOrOne))
        .then(RegexElement::Set('0', '9', Quantifier::OneOrMany))
        .then(RegexElement::Item('.', Quantifier::Exactly(1)))
        .then(RegexElement::Set('0', '9', Quantifier::OneOrMany));

    let int_regex = Regex::new()
        .then(RegexElement::Item('-', Quantifier::ZeroOrOne))
        .then(RegexElement::Set('0', '9', Quantifier::OneOrMany));

    let hex_regex = Regex::new()
        .then(RegexElement::Item('#', Quantifier::Exactly(1)))
        .then(RegexElement::Group(vec![
            RegexElement::AnyOf(vec![
                RegexElement::Set('0', '9', Quantifier::Exactly(1)),
                RegexElement::Set('a', 'f', Quantifier::Exactly(1)),
                RegexElement::Set('A', 'F', Quantifier::Exactly(1))
            ])
        ], Quantifier::Exactly(6)));

    let if_regex = Regex::new()
        .then(RegexElement::Item('i', Quantifier::Exactly(1)))
        .then(RegexElement::Item('f', Quantifier::Exactly(1)));

    let else_regex = Regex::new()
        .then(RegexElement::Item('e', Quantifier::Exactly(1)))
        .then(RegexElement::Item('l', Quantifier::Exactly(1)))
        .then(RegexElement::Item('s', Quantifier::Exactly(1)))
        .then(RegexElement::Item('e', Quantifier::Exactly(1)));

    let while_regex = Regex::new()
        .then(RegexElement::Item('w', Quantifier::Exactly(1)))
        .then(RegexElement::Item('h', Quantifier::Exactly(1)))
        .then(RegexElement::Item('i', Quantifier::Exactly(1)))
        .then(RegexElement::Item('l', Quantifier::Exactly(1)))
        .then(RegexElement::Item('e', Quantifier::Exactly(1)));

    let travel_regex = Regex::new()
        .then(RegexElement::Item('t', Quantifier::Exactly(1)))
        .then(RegexElement::Item('r', Quantifier::Exactly(1)))
        .then(RegexElement::Item('a', Quantifier::Exactly(1)))
        .then(RegexElement::Item('v', Quantifier::Exactly(1)))
        .then(RegexElement::Item('e', Quantifier::Exactly(1)))
        .then(RegexElement::Item('l', Quantifier::Exactly(1)));

    let subcanvas_regex = Regex::new()
        .then(RegexElement::Item('s', Quantifier::Exactly(1)))
        .then(RegexElement::Item('u', Quantifier::Exactly(1)))
        .then(RegexElement::Item('b', Quantifier::Exactly(1)))
        .then(RegexElement::Item('c', Quantifier::Exactly(1)))
        .then(RegexElement::Item('a', Quantifier::Exactly(1)))
        .then(RegexElement::Item('n', Quantifier::Exactly(1)))
        .then(RegexElement::Item('v', Quantifier::Exactly(1)))
        .then(RegexElement::Item('a', Quantifier::Exactly(1)))
        .then(RegexElement::Item('s', Quantifier::Exactly(1)));

    let def_regex = Regex::new()
        .then(RegexElement::Item('d', Quantifier::Exactly(1)))
        .then(RegexElement::Item('e', Quantifier::Exactly(1)))
        .then(RegexElement::Item('f', Quantifier::Exactly(1)));

    let return_regex = Regex::new()
        .then(RegexElement::Item('r', Quantifier::Exactly(1)))
        .then(RegexElement::Item('e', Quantifier::Exactly(1)))
        .then(RegexElement::Item('t', Quantifier::Exactly(1)))
        .then(RegexElement::Item('u', Quantifier::Exactly(1)))
        .then(RegexElement::Item('r', Quantifier::Exactly(1)))
        .then(RegexElement::Item('n', Quantifier::Exactly(1)));

    let bool_regex = Regex::new()
        .then(RegexElement::AnyOf(vec![
            RegexElement::Group(vec![
                RegexElement::Item('t', Quantifier::Exactly(1)),
                RegexElement::Item('r', Quantifier::Exactly(1)),
                RegexElement::Item('u', Quantifier::Exactly(1)),
                RegexElement::Item('e', Quantifier::Exactly(1))
            ], Quantifier::Exactly(1)),

            RegexElement::Group(vec![
                RegexElement::Item('f', Quantifier::Exactly(1)),
                RegexElement::Item('a', Quantifier::Exactly(1)),
                RegexElement::Item('l', Quantifier::Exactly(1)),
                RegexElement::Item('s', Quantifier::Exactly(1)),
                RegexElement::Item('e', Quantifier::Exactly(1))
            ], Quantifier::Exactly(1))
        ]));

    let gt_eq_regex = Regex::new()
        .then(RegexElement::Item('>', Quantifier::Exactly(1)))
        .then(RegexElement::Item('=', Quantifier::Exactly(1)));

    let lt_eq_regex = Regex::new()
        .then(RegexElement::Item('<', Quantifier::Exactly(1)))
        .then(RegexElement::Item('=', Quantifier::Exactly(1)));
    
    let not_eq_regex = Regex::new()
        .then(RegexElement::Item('!', Quantifier::Exactly(1)))
        .then(RegexElement::Item('=', Quantifier::Exactly(1)));

    let sigle_comment_regex = Regex::new()
        .then(RegexElement::Item('/', Quantifier::Exactly(2)))
        .then(RegexElement::NoneOf(vec![
            RegexElement::Item('\n', Quantifier::Exactly(1))
        ], Quantifier::ZeroOrMany));


    lexer.register(LexerNode::new(if_regex, TokenType::If));
    lexer.register(LexerNode::new(else_regex, TokenType::Else));
    lexer.register(LexerNode::new(while_regex, TokenType::While));
    //lexer.register(LexerNode::new(travel_regex, TokenType::Travel));
    lexer.register(LexerNode::new(subcanvas_regex, TokenType::Subcanvas));
    lexer.register(LexerNode::new(def_regex, TokenType::Def));
    lexer.register(LexerNode::new(return_regex, TokenType::Return));

    lexer.register(LexerNode::new(bool_regex, TokenType::Bool));
    lexer.register(LexerNode::new(float_regex, TokenType::Float));
    lexer.register(LexerNode::new(int_regex, TokenType::Int));
    lexer.register(LexerNode::new(hex_regex, TokenType::Hex));

    lexer.register(LexerNode::new(ident_regex, TokenType::Ident));


    lexer.register(LexerNode::new(sigle_comment_regex, TokenType::SingleComment));


    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('(', Quantifier::Exactly(1))), TokenType::LParen));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item(')', Quantifier::Exactly(1))), TokenType::RParen));

    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('{', Quantifier::Exactly(1))), TokenType::LBracket));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('}', Quantifier::Exactly(1))), TokenType::RBracket));

    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item(',', Quantifier::Exactly(1))), TokenType::Comma));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item(':', Quantifier::Exactly(1))), TokenType::Colon));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item(';', Quantifier::Exactly(1))), TokenType::SemiColon));

    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('+', Quantifier::Exactly(1))), TokenType::Plus));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('-', Quantifier::Exactly(1))), TokenType::Minus));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('*', Quantifier::Exactly(1))), TokenType::Mul));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('/', Quantifier::Exactly(1))), TokenType::Div));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('%', Quantifier::Exactly(1))), TokenType::Mod));


    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('&', Quantifier::Exactly(2))), TokenType::And));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('|', Quantifier::Exactly(2))), TokenType::Or));

    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('=', Quantifier::Exactly(2))), TokenType::DoubleEq));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('=', Quantifier::Exactly(1))), TokenType::Eq));

    lexer.register(LexerNode::new(gt_eq_regex, TokenType::GTEq));
    lexer.register(LexerNode::new(lt_eq_regex, TokenType::LTEq));
    lexer.register(LexerNode::new(not_eq_regex, TokenType::NotEq));

    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('>', Quantifier::Exactly(1))), TokenType::GT));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('<', Quantifier::Exactly(1))), TokenType::LT));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('!', Quantifier::Exactly(1))), TokenType::Not));

    
}

fn test_tokenize(content:String, path:&str) -> LexingResult<TokenType>{
    let mut lexer = Lexer::new();
    init_lexer(&mut lexer);


    lexer.tokenize_content(content, &path)
}

fn test_parse(content:String, path: &str){
    match test_tokenize(content, path){
        LexingResult::Ok(tokens) => {
            match parser::parse(&tokens, true){
                Some(frst) => {
                    let mut env = Environment::default();
                    if verify(&frst, &mut env){
                        for instr in ir::parse(&frst, &mut Context::default()){
                            println!("{instr:?}")
                        }
                    }
                },

                None => {
                    eprintln!("Could not parse {path}");
                }
            }
        },
        LexingResult::Err(errs) =>{
            for e in errs{
                eprintln!("{e}");
            }
        }
    }
}

fn tokenize(path: &str) -> LexingResult<TokenType>{
    let mut lexer = Lexer::new();
    init_lexer(&mut lexer);

    lexer.tokenize_file(path)
}

fn parse(path: &str){
    match tokenize(path){
        LexingResult::Ok(tokens) => {
            match parser::parse(&tokens, true){
                Some(frst) => {
                    for ast in frst{
                        println!("{ast:?}");
                    }
                },

                None => {
                    eprintln!("Could not parse {path}");
                }
            }
        },
        LexingResult::Err(errs) =>{
            for e in errs{
                eprintln!("{e}");
            }
        }
    }
}
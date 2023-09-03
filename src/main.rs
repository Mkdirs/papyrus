use environment::Environment;
use ir::Runtime;
use neoglot_lib::{regex::*, lexer::*, parser::AST};
use validator::verify;
use vm::VM;
use std::{env, fmt::Display, collections::HashSet, path::Path};

mod parser;
mod validator;
mod environment;
mod ir;
mod vm;
mod output;

#[derive(Debug, Hash, PartialOrd, PartialEq, Eq, Copy, Clone)]
pub enum TokenType{
    Ident,

    Pub, Import,

    String,

    Int, Float, Hex, Bool,

    LParen, RParen,
    LBracket, RBracket,

    Comma, Colon, SemiColon, Dot,

    If, Else, While,
    Travel, Subcanvas,
    Def, Return,

    Plus, Minus, Mul,
    Div, Mod, Pow,

    Eq,

    And, Or,
    DoubleEq, GT, LT,
    Not, NotEq, GTEq,
    LTEq,

    SingleComment
}

impl Symbol for TokenType{}
impl TokenKind for TokenType{}

const IMG_OUTPUT:&str = "-img";
const VID_OUTPUT:&str = "-vid";

const IMG_FORMAT:&[&str] = &["png", "jpg"];
const VID_FORMAT:&[&str] = &["mp4"];

#[derive(Debug)]
struct Command<'a>{
    name: &'a str,
    args: Vec<String>,
    options: HashSet<String>
}

fn main() {

    let raw_args = env::args().collect::<Vec<String>>();
    let args = raw_args.get(1..).unwrap_or_default();

    if args.is_empty(){
        help();
        return;
    }
    let cmd = read_cmd(&args);

    if cmd.name == "run"{
        if cmd.args.len() == 1{
            run(&cmd.args[0], IMG_OUTPUT, IMG_FORMAT[0], cmd.options);
        
        }else if cmd.args.len() == 2{
            let default_format = if &cmd.args[1] == IMG_OUTPUT{
                IMG_FORMAT[0]
            }else if &cmd.args[1] == VID_OUTPUT{
                VID_FORMAT[0]
            }else{
                eprintln!("Unknown output type: {}", cmd.args[1]);
                help();
                return;
            };

            run(&cmd.args[0], &cmd.args[1], default_format, cmd.options)
        }else if cmd.args.len() == 3{
            if &cmd.args[1] != IMG_OUTPUT && &cmd.args[1] != VID_OUTPUT{
                eprintln!("Unknown output type: {}", cmd.args[1]);
                help();
                return;
            }
            
            if &cmd.args[1] == IMG_OUTPUT{
                if !IMG_FORMAT.contains(&cmd.args[2].as_str()){
                    eprintln!("Unknown image file format: {}", cmd.args[2]);
                    help();
                    return;
                }
            }else if &cmd.args[1] == VID_OUTPUT{
                if !VID_FORMAT.contains(&cmd.args[2].as_str()){
                    eprintln!("Unknown video file format: {}", cmd.args[2]);
                    help();
                    return;
                }
            }

            run(&cmd.args[0], &cmd.args[1], &cmd.args[2], cmd.options);
        }
        return;
    }

    help();

    
}

fn run(file:&str, output:&str, format:&str, options: HashSet<String>){
    let base = Path::new(file);

    let path = if base.is_relative(){
        env::current_dir().unwrap().join(base)
    }else{
        base.to_path_buf()
    };


    if let Some(ext) = path.extension(){
        if ext != ".pprs"{
            eprintln!("Expected a '.pprs' file extension");
            return;
        }
    }else{
        eprintln!("Expected a '.pprs' file extension");
        return;
    }

    if !path.exists(){
        eprintln!("Could not find file {}", path.display());
        return;
    }

    if let Some(f) = path.to_str(){
        let runtime = parse(f);

        if let Some(runtime) = runtime{
            let mut vm = VM::new(runtime);
            vm.run(&path, "main");

            if output == IMG_OUTPUT{
                for (i, canvas) in vm.get_saved_canvas().iter().enumerate(){
                    let data = canvas.data.iter().flat_map(|e| to_rgb(*e)).collect::<Vec<u8>>();

                    if format == IMG_FORMAT[0]{
                        output::stbi_write_png(&format!("canvas{i}.png"), canvas.width as u32, canvas.height as u32, 3, &data, 3*canvas.width as u32);
                    
                    }else if format == IMG_FORMAT[1]{
                        output::stbi_write_jpg(&format!("canvas{i}.jpg"), canvas.width as u32, canvas.height as u32, 3, &data, 85);
                    }
                }
            }
        }
    }else{
        eprintln!("Non-UTF8 chars found on the filename");
    }
    

}

fn read_cmd<'a>(args: &'a[String]) -> Command<'a>{
    let name = &args[0];
    let mut cmd = Command{name, args: vec![], options: HashSet::new()};


    for arg in &args[1..]{
        if !arg.starts_with("--"){
            cmd.args.push(arg.to_string());
        }else{
            cmd.options.insert(arg.to_string());
        }
    }


    cmd
}

fn help(){
    println!("papyrus help");
    println!("  Shows this");
    println!();
    println!("papyrus run <script>");
    println!("  Runs a script file");
    println!("  The default output type is {IMG_OUTPUT} and format is {}", IMG_FORMAT[0]);
    println!();
    println!("papyrus run <script> <{IMG_OUTPUT} | {VID_OUTPUT}>");
    println!("  Runs a script file");
    println!("  Generates an image for each canvas if {IMG_OUTPUT} is set. The default format is {}", IMG_FORMAT[0]);
    println!("  (W.I.P) Generates a single video file from all the canvas if -{VID_OUTPUT} is set. The default format is {}", VID_FORMAT[0]);
    println!();
    println!("papyrus run <script> {IMG_OUTPUT} <{}>", format_array(IMG_FORMAT, "|"));
    println!("  Runs a script file");
    println!("  Sets the output images file format");
    println!();
    println!("(W.I.P) papyrus run <script> {VID_OUTPUT} <{}> <--export-frames>", format_array(VID_FORMAT, "|"));
    println!("  Runs a script file");
    println!("  Sets the output video file format");
    println!("  Also generates the individual frames of the video if the option --export-frames is set");

}

fn format_array<>(a:&[impl Display], sep:&str) -> String{
    let mut str = String::new();
    for e in a{
        str.push_str(&format!("{e}{sep}"));
    }
    str.remove(str.len()-1);

    str
}

fn to_rgb(pixel: u32) -> [u8; 3]{
    let r = ((pixel >> 16) & 0xff) as u8;
    let g = ((pixel >> 8) & 0xff) as u8;
    let b = (pixel & 0xff) as u8;
    [r, g, b]
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

    let pub_regex = Regex::new()
            .then(RegexElement::Item('p', Quantifier::Exactly(1)))
            .then(RegexElement::Item('u', Quantifier::Exactly(1)))
            .then(RegexElement::Item('b', Quantifier::Exactly(1)));

    let import_regex = Regex::new()
            .then(RegexElement::Item('i', Quantifier::Exactly(1)))
            .then(RegexElement::Item('m', Quantifier::Exactly(1)))
            .then(RegexElement::Item('p', Quantifier::Exactly(1)))
            .then(RegexElement::Item('o', Quantifier::Exactly(1)))
            .then(RegexElement::Item('r', Quantifier::Exactly(1)))
            .then(RegexElement::Item('t', Quantifier::Exactly(1)));

    let string_regex = Regex::new()
            .then(RegexElement::Item('"', Quantifier::Exactly(1)))
            .then(RegexElement::NoneOf(vec![
                RegexElement::Item('"', Quantifier::Exactly(1))
            ], Quantifier::ZeroOrMany))
            .then(RegexElement::Item('"', Quantifier::Exactly(1)));


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

    lexer.register(LexerNode::new(pub_regex, TokenType::Pub));
    lexer.register(LexerNode::new(import_regex, TokenType::Import));
    lexer.register(LexerNode::new(string_regex, TokenType::String));

    lexer.register(LexerNode::new(ident_regex, TokenType::Ident));


    lexer.register(LexerNode::new(sigle_comment_regex, TokenType::SingleComment));


    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('(', Quantifier::Exactly(1))), TokenType::LParen));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item(')', Quantifier::Exactly(1))), TokenType::RParen));

    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('{', Quantifier::Exactly(1))), TokenType::LBracket));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('}', Quantifier::Exactly(1))), TokenType::RBracket));

    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item(',', Quantifier::Exactly(1))), TokenType::Comma));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item(':', Quantifier::Exactly(1))), TokenType::Colon));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item(';', Quantifier::Exactly(1))), TokenType::SemiColon));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('.', Quantifier::Exactly(1))), TokenType::Dot));

    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('+', Quantifier::Exactly(1))), TokenType::Plus));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('-', Quantifier::Exactly(1))), TokenType::Minus));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('*', Quantifier::Exactly(1))), TokenType::Mul));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('/', Quantifier::Exactly(1))), TokenType::Div));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('%', Quantifier::Exactly(1))), TokenType::Mod));
    lexer.register(LexerNode::new(Regex::new().then(RegexElement::Item('^', Quantifier::Exactly(1))), TokenType::Pow));


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


pub fn tokenize(path: &str) -> LexingResult<TokenType>{
    let mut lexer = Lexer::new();
    init_lexer(&mut lexer);

    lexer.tokenize_file(path)
}

pub fn prepare(path: &str) -> Vec<AST<Token<TokenType>>>{
    match tokenize(path){
        LexingResult::Ok(tokens) => {
            match parser::parse(&tokens, true){
                Some(forest) => {
                    if verify(&forest, None, &mut Environment::default()){
                        forest
                    }else {vec![]}
                },

                None => {
                    eprintln!("Could not parse {path}");
                    vec![]
                }
            }
        },

        LexingResult::Err(errs) => {
            for e in errs{
                eprintln!("{e}");
            }
            vec![]
        }
    }
}

fn parse(path: &str) -> Option<Runtime>{
    let forest = prepare(path);
    if forest.is_empty(){
        return None;
    }

    Some(ir::parse(&forest))
    
}
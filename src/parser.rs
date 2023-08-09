use neoglot_lib::{lexer::Token, parser::{Parser, AST, ParsingError, expression::{ExpressionParser, Expr}, expect}, regex::{Regex, RegexElement, Quantifier}};

use crate::TokenType;
/* 
const VAR_DECL:Regex<TokenType> = Regex::new()
    .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
    .then(RegexElement::Item(TokenType::Colon, Quantifier::Exactly(1)))
    .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
    .then(RegexElement::Item(TokenType::SemiColon, Quantifier::Exactly(1)));

const VAR_ASSIGN_INFER:Regex<TokenType> = Regex::new()
    .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
    .then(RegexElement::Item(TokenType::Eq, Quantifier::Exactly(1)))
    .then(RegexElement::NoneOf(vec![
        RegexElement::Item(TokenType::SemiColon, Quantifier::Exactly(1))
    ], Quantifier::OneOrMany))
    .then(RegexElement::Item(TokenType::SemiColon, Quantifier::Exactly(1)));
*/

pub fn parse(tokens:&[Token<TokenType>], semicolon_terminated:bool) -> Result<Vec<AST<Token<TokenType>>>, Vec<ParsingError<TokenType>>>{
    let mut forest:Vec<AST<Token<TokenType>>> = vec![];
    let mut errors:Vec<ParsingError<TokenType>> = vec![];

    let mut parser:Parser<TokenType, Token<TokenType>> = Parser::new(tokens);

    let var_assign_typed = Regex::new()
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Colon, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Eq, Quantifier::Exactly(1)))
        .then(RegexElement::NoneOf(vec![
            RegexElement::Item(TokenType::SemiColon, Quantifier::Exactly(1))
        ], Quantifier::OneOrMany));


    let var_assign_infer = Regex::new()
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Eq, Quantifier::Exactly(1)))
        .then(RegexElement::NoneOf(vec![
            RegexElement::Item(TokenType::SemiColon, Quantifier::Exactly(1))
        ], Quantifier::OneOrMany));


    let var_decl = Regex::new()
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Colon, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)));

    

    while !parser.finished(){
        if parser.on_regex(&var_assign_typed){
            match parser.slice_regex(&var_assign_typed){
                Ok(tokens) => {
                    let raw_expr = parse_expression(&tokens[4..tokens.len()]).unwrap()?;
                    let normalized = normalize_expression(raw_expr, false)?;

                    let tree = AST{kind: tokens[3].clone(), children: vec![
                        AST{ kind: tokens[0].clone(), children: vec![
                            AST{ kind: tokens[2].clone(), children: vec![] }
                        ] },
                        normalized
                    ]};

                    forest.push(tree);
                    parser.skip(tokens.len());

                    match expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon, tokens[tokens.len()-1].location.clone()){
                        Ok(_) => parser.skip(1),
                        Err(e) => errors.push(e)
                    }

                    
                },

                Err(e) =>{
                    errors.push(e);
                    parser.skip(1);
                }
            }


        }else if parser.on_regex(&var_assign_infer){
            match parser.slice_regex(&var_assign_infer){
                Ok(tokens) => {
                    let raw_expr = parse_expression(&tokens[2..tokens.len()]).unwrap()?;
                    let normalized = normalize_expression(raw_expr, false)?;

                    let tree = AST{kind: tokens[1].clone(), children: vec![
                        AST{ kind: tokens[0].clone(), children: vec![] },
                        normalized
                    ]};

                    forest.push(tree);
                    parser.skip(tokens.len());

                    match expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon, tokens[tokens.len()-1].location.clone()){
                        Ok(_) => parser.skip(1),
                        Err(e) => errors.push(e)
                    }

                },
                Err(e) => {
                    errors.push(e);
                    parser.skip(1);
                }
            }
        }else if parser.on_regex(&var_decl){
            match parser.slice_regex(&var_decl){
                Ok(tokens) => {
                    forest.push(AST{ kind: tokens[0].clone(), children: vec![
                        AST{ kind: tokens[2].clone(), children: vec![] }
                    ] });

                    parser.skip(tokens.len());

                    if semicolon_terminated{
                        match expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon, tokens[tokens.len()-1].location.clone()){
                            Ok(_) => parser.skip(1),
                            Err(e) => errors.push(e)
                        }
                    }
                },

                Err(e) => {
                    errors.push(e);
                    parser.skip(1);
                }
            }
        }else if parser.on_token(TokenType::If){
            let loc = parser.peek().unwrap().location.clone();
            let ast = parse_structure(&mut parser, TokenType::If, true, semicolon_terminated)?;
            if ast.children.len() == 2{
                forest.push(ast);
            
            }else{errors.push(ParsingError::UnexpectedToken {
                expected: None, got: None, location: loc
            })}

        }else if parser.on_token(TokenType::While){
            let loc = parser.peek().unwrap().location.clone();
            let ast = parse_structure(&mut parser, TokenType::While, true, semicolon_terminated)?;
            if ast.children.len() == 2{
                forest.push(ast);
            
            }else{errors.push(ParsingError::UnexpectedToken {
                expected: None, got: None, location: loc
            })}

        }else if parser.on_token(TokenType::Travel){
            let loc = parser.peek().unwrap().location.clone();
            let ast = parse_structure(&mut parser, TokenType::Travel, true, semicolon_terminated)?;
            if ast.children.len() == 3{
                forest.push(ast);
            
            }else{errors.push(ParsingError::UnexpectedToken {
                expected: None, got: None, location: loc
            })}

        }else if parser.on_token(TokenType::Subcanvas){
            let loc = parser.peek().unwrap().location.clone();
            let ast = parse_structure(&mut parser, TokenType::Subcanvas, true, semicolon_terminated)?;
            if ast.children.len() == 5{
                forest.push(ast);
            
            }else{errors.push(ParsingError::UnexpectedToken {
                expected: None, got: None, location: loc
            })}

        }else if parser.on_token(TokenType::Def){
            let mut ast = AST{ kind: parser.peek().unwrap().clone(), children:vec![] };
            parser.skip(1);
            ast.children.push(parse_structure(&mut parser, TokenType::Ident, true, semicolon_terminated)?);
            
            forest.push(ast);

        }else if parser.on_token(TokenType::Ident){
            let mut ast = AST{ kind: parser.peek().unwrap().clone(), children: vec![] };
            parser.skip(1);
            match parser.slice_block(TokenType::LParen, TokenType::RParen){
                Ok(tokens) =>{
                    let mut body = AST { kind: parser.peek().unwrap().clone(), children: vec![] };
                    let params = split_list(TokenType::Comma, tokens);
                    for param in params{
                        if let Some(raw_expr) = parse_expression(&param){
                            body.children.push(normalize_expression(raw_expr?, false)?);
                        }
                    }

                    let loc = ast.kind.location.clone();

                    ast.children.push(body);
                    forest.push(ast);
                    parser.skip(tokens.len()+2);

                    if semicolon_terminated{
                        match expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon, loc){
                            Ok(_) => parser.skip(1),
                            Err(e) => errors.push(e)
                        }
                    }
                },

                Err(e) => errors.push(e)
            }

        }else{
            let got = parser.peek().unwrap().clone();
            errors.push(ParsingError::UnparsedSequence(got.location.clone()));
            parser.skip(1);
        }
    }

    if !errors.is_empty(){ Err(errors) }
    else { Ok(forest) }
}

fn split_list(delimiter:TokenType, tokens:&[Token<TokenType>]) -> Vec<Vec<Token<TokenType>>>{
    let mut list:Vec<Vec<Token<TokenType>>> = vec![];
    let mut param = vec![];
    let mut open_paren = 0;

    for token in tokens{
        if token.kind == delimiter{
            if open_paren != 0{ param.push(token.clone()) }
            else{
                list.push(param.clone());
                param.clear();
            }
        }else{
            if token.kind == TokenType::LParen{ open_paren +=1; }
            else if token.kind == TokenType::RParen{ open_paren -= 1; }
            
            param.push(token.clone());
        }
        
    }

    if !param.is_empty(){
        list.push(param.clone());
        param.clear();
    }

    list
}

fn parse_block(parser:&mut Parser<TokenType, Token<TokenType>>, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let block_begin = parser.peek();
    match parser.slice_block(TokenType::LBracket, TokenType::RBracket){
        Ok(tokens) => {
            let ast = AST{ kind: block_begin.unwrap().clone(), children: parse(tokens, semicolon_terminated)? };

            parser.skip(tokens.len()+2);

            Ok(ast)
        },

        Err(e) => {
            parser.skip(1);
            Err(vec![e])
        }
    }
}

fn parse_structure(parser:&mut Parser<TokenType, Token<TokenType>>, head:TokenType, has_params:bool, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let token = parser.peek().unwrap().clone();
    if let Err(e) = expect(Some(token.kind), head, token.location.clone()){
        return Err(vec![e]);
    }

    let mut errors = vec![];
    let mut structure = AST{ kind: token.clone(), children: vec![] };

    parser.skip(1);
    if has_params {
        match parser.slice_block(TokenType::LParen, TokenType::RParen){
            Ok(tokens) => {
                let params = split_list(TokenType::Comma, tokens);
                
                for param in params{
                    if let Some(raw_expr) = parse_expression(&param){
                        structure.children.push(normalize_expression(raw_expr?, false)?);
                    
                    }else { errors.push(ParsingError::NoTokens) }
                    
                }

                parser.skip(tokens.len()+2);
            },

            Err(e) => return Err(vec![e])
        }

        
    }
    structure.children.push(parse_block(parser, semicolon_terminated)?);

    if !errors.is_empty(){ Err(errors) }
    else { Ok(structure) }
}


fn parse_expression(tokens: &[Token<TokenType>]) -> Option<Result<AST<Expr<TokenType>>, Vec<ParsingError<TokenType>>>>{
    let mut parser = ExpressionParser::new();
    

    parser.add_operator(TokenType::Not, 1);

    parser.add_operator(TokenType::DoubleEq, 2);
    parser.add_operator(TokenType::NotEq, 2);
    
    parser.add_operator(TokenType::GT, 3);
    parser.add_operator(TokenType::LT, 3);
    parser.add_operator(TokenType::GTEq, 3);
    parser.add_operator(TokenType::LTEq, 3);


    

    parser.add_operator(TokenType::Plus, 4);
    parser.add_operator(TokenType::Minus, 4);
    
    parser.add_operator(TokenType::Mul, 5);
    parser.add_operator(TokenType::Div, 5);

    parser.add_operator(TokenType::Mod, 6);
    



    


    parser.set_high_priority_group(TokenType::LParen, TokenType::RParen);

    parser.parse(tokens)
}

fn normalize_expression(expr:AST<Expr<TokenType>>, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let mut normalized:AST<Token<TokenType>>;

    normalized = match expr.kind{
        Expr::Operator(t) => AST{kind: t, children: vec![]},
        Expr::Operand(t) => AST{kind: t, children: vec![]},
        Expr::Unknown(tokens) => {
            let forest = parse(tokens, semicolon_terminated)?;
            if forest.len() > 1 { return Err(vec![ParsingError::UnparsedSequence(tokens[0].location.clone())]) }

            forest[0].clone()
        }
    };

    for e in expr.children{
        let child = normalize_expression(e, semicolon_terminated)?;
        normalized.children.push(child);
    }

    Ok(normalized)


}

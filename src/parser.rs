use neoglot_lib::{lexer::Token, parser::{Parser, AST, ParsingError, expression::{ExpressionParser, Expr}, expect}, regex::{Regex, RegexElement, Quantifier}};

use crate::TokenType;

fn typed_var_assign_regex() -> Regex<TokenType>{
    Regex::new()
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Colon, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Eq, Quantifier::Exactly(1)))
        .then(RegexElement::NoneOf(vec![
            RegexElement::Item(TokenType::SemiColon, Quantifier::Exactly(1))
        ], Quantifier::OneOrMany))
}

fn inferred_var_assign_regex() -> Regex<TokenType>{
    Regex::new()
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Eq, Quantifier::Exactly(1)))
        .then(RegexElement::NoneOf(vec![
            RegexElement::Item(TokenType::SemiColon, Quantifier::Exactly(1))
        ], Quantifier::OneOrMany))
}

fn type_binding_regex() -> Regex<TokenType>{
    Regex::new()
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Colon, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
}



pub fn parse(tokens:&[Token<TokenType>], semicolon_terminated:bool) -> Result<Vec<AST<Token<TokenType>>>, Vec<ParsingError<TokenType>>>{
    let mut forest:Vec<AST<Token<TokenType>>> = vec![];
    let mut errors:Vec<ParsingError<TokenType>> = vec![];

    let mut parser:Parser<TokenType, Token<TokenType>> = Parser::new(tokens);

    while !parser.finished(){
        if parser.on_regex(&typed_var_assign_regex()){
            match parse_typed_var_assign(&mut parser){
                Ok(ast) => forest.push(ast),
                Err(errs) => {
                    for e in errs { errors.push(e); }
                }
            }

        }else if parser.on_regex(&inferred_var_assign_regex()){
            match parse_inferred_var_assign(&mut parser){
                Ok(ast) => forest.push(ast),
                Err(errs) => {
                    for e in errs { errors.push(e); }
                }
            }
            
        }else if parser.on_regex(&type_binding_regex()){
            match parse_type_binding(&mut parser, semicolon_terminated){
                Ok(ast) => forest.push(ast),
                Err(errs) => {
                    for e in errs { errors.push(e); }
                }
            }
            
        }else if parser.on_token(TokenType::If){
            match parse_if(&mut parser, semicolon_terminated){
                Ok(ast) => forest.push(ast),
                Err(errs) =>{
                    for e in errs { errors.push(e); }
                }
            }

        }else if parser.on_token(TokenType::While){
            match parse_while(&mut parser, semicolon_terminated){
                Ok(ast) => forest.push(ast),
                Err(errs) =>{
                    for e in errs { errors.push(e); }
                }
            }

        }else if parser.on_token(TokenType::Travel){
            match parse_travel(&mut parser, semicolon_terminated){
                Ok(ast) => forest.push(ast),
                Err(errs) => {
                    for e in errs { errors.push(e); }
                }
            }

        }else if parser.on_token(TokenType::Subcanvas){
            match parse_subcanvas(&mut parser, semicolon_terminated){
                Ok(ast) => forest.push(ast),
                Err(errs) => {
                    for e in errs { errors.push(e); }
                }
            }

        }else if parser.on_token(TokenType::Def){
            match parse_def(&mut parser, semicolon_terminated){
                Ok(ast) => forest.push(ast),
                Err(errs) => {
                    for e in errs { errors.push(e); }
                }
            }

        }else if parser.on_token(TokenType::Ident){
            match parse_ident(&mut parser, semicolon_terminated) {
                Ok(ast) => forest.push(ast),
                Err(errs) => {
                    for e in errs { errors.push(e); }
                }
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

fn parse_typed_var_assign(parser:&mut Parser<TokenType, Token<TokenType>>) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    match parser.slice_regex(&typed_var_assign_regex()){
        Ok(tokens) => {
            
            let raw_expr = parse_expression(&tokens[4..tokens.len()]);
            if raw_expr.is_none(){
                parser.skip(1);
                return Err(vec![ParsingError::NoTokens])
            }
            let normalized = match raw_expr.unwrap(){
                Ok(raw_expr) => normalize_expression(raw_expr),
                Err(e) => Err(e)
            };

            if normalized.is_err(){
                parser.skip(1);
                return Err(normalized.unwrap_err());
            }

            let tree = AST{kind: tokens[3].clone(), children: vec![
                AST{ kind: tokens[1].clone(), children: vec![
                    AST{ kind: tokens[0].clone(), children: vec![] },
                    AST{ kind: tokens[2].clone(), children: vec![] }
                ] },
                normalized.unwrap()
            ]};

            

            parser.skip(tokens.len());

            match expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon, tokens[tokens.len()-1].location.clone()){
                Ok(_) => {
                    parser.skip(1);
                    Ok(tree)
                },
                Err(e) => {
                    parser.skip(1);
                    Err(vec![e])
                }
            }

            
        },

        Err(e) =>{
            parser.skip(1);
            Err(vec![e])
        }
    }
}

fn parse_inferred_var_assign(parser:&mut Parser<TokenType, Token<TokenType>>) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    match parser.slice_regex(&inferred_var_assign_regex()){
        Ok(tokens) => {
            let raw_expr = parse_expression(&tokens[2..tokens.len()]);

            if raw_expr.is_none(){
                parser.skip(1);
                return Err(vec![ParsingError::NoTokens])
            }

            let normalized = match  raw_expr.unwrap() {
                Ok(raw_expr) => normalize_expression(raw_expr),
                Err(e) => Err(e)
            };

            if normalized.is_err(){
                parser.skip(1);
                return Err(normalized.unwrap_err());
            }

            let tree = AST{kind: tokens[1].clone(), children: vec![
                AST{ kind: tokens[0].clone(), children: vec![] },
                normalized.unwrap()
            ]};

            parser.skip(tokens.len());

            match expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon, tokens[tokens.len()-1].location.clone()){
                Ok(_) => {
                    parser.skip(1);
                    Ok(tree)
                },
                Err(e) => {
                    parser.skip(1);
                    Err(vec![e])
                }
            }

        },
        Err(e) => {
            parser.skip(1);
            Err(vec![e])
        }
    }
}

fn parse_type_binding(parser:&mut Parser<TokenType, Token<TokenType>>, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>> {
    match parser.slice_regex(&type_binding_regex()){
        Ok(tokens) => {
            let tree = AST{ kind: tokens[1].clone(), children: vec![
                AST{ kind: tokens[0].clone(), children: vec![] },
                AST{ kind: tokens[2].clone(), children: vec![] }
            ] };

            parser.skip(tokens.len());

            if semicolon_terminated{
                match expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon, tokens[tokens.len()-1].location.clone()){
                    Ok(_) => {
                        parser.skip(1);
                        Ok(tree)
                    },
                    Err(e) => {
                        parser.skip(1);
                        Err(vec![e])
                    }
                }
            }else { Ok(tree) }
        },

        Err(e) => {
            parser.skip(1);
            Err(vec![e])
        }
    }
}

fn parse_if(parser:&mut Parser<TokenType, Token<TokenType>>, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let if_tok = parser.pop().unwrap().clone();
    match parser.slice_block(TokenType::LParen, TokenType::RParen){
        Ok(tokens) =>{
            parser.skip(tokens.len()+2);
            if let Some(raw_expr) = parse_expression(tokens){
                match raw_expr{
                    Ok(raw_expr) => {
                        match normalize_expression(raw_expr){
                            Ok(expr) => {
                                let block = parse_block(parser, semicolon_terminated)?;

                                Ok(AST { kind: if_tok, children: vec![
                                    expr,
                                    block
                                ] })
                            },

                            Err(e) => {
                                parser.skip(1);
                                Err(e)
                            }
                        }
                    },

                    Err(e) => {
                        parser.skip(1);
                        Err(e)
                    }
                }

                
            }else{
                parser.skip(1);
                Err(vec![ParsingError::NoTokens])
            }

        },

        Err(e) => {
            parser.skip(1);
            Err(vec![e])
        }
    }
}

fn parse_while(parser:&mut Parser<TokenType, Token<TokenType>>, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let while_tok = parser.pop().unwrap().clone();
    match parser.slice_block(TokenType::LParen, TokenType::RParen){
        Ok(tokens) =>{
            parser.skip(tokens.len()+2);
            if let Some(raw_expr) = parse_expression(tokens){
                match raw_expr {
                    Ok(raw_expr) => {
                        match normalize_expression(raw_expr){
                            Ok(expr) => {
                                let block = parse_block(parser, semicolon_terminated)?;

                                Ok(AST { kind: while_tok, children: vec![
                                    expr,
                                    block
                                ] })
                            },

                            Err(e) => {
                                parser.skip(1);
                                Err(e)
                            }
                        }
                    },

                    Err(e) => {
                        parser.skip(1);
                        Err(e)
                    }
                }

                
            }else{
                parser.skip(1);
                Err(vec![ParsingError::NoTokens])
            }

        },

        Err(e) => {
            parser.skip(1);
            Err(vec![e])
        }
    }
}

fn parse_travel(parser:&mut Parser<TokenType, Token<TokenType>>, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let travel_tok = parser.pop().unwrap().clone();

    match expect(parser.pop().and_then(|e| Some(e.kind)), TokenType::LParen, travel_tok.location.clone()){
        Ok(_) => {},
        Err(e) => {
            parser.skip(1);
            return Err(vec![e]);
        }
    }
   

    let x_ident = parser.pop();
    match expect(x_ident.and_then(|e| Some(e.kind)), TokenType::Ident, travel_tok.location.clone()){
        Ok(_) => {},
        Err(e) => {
            parser.skip(1);
            return Err(vec![e]);
        }
    }
    let x_ident = x_ident.unwrap().clone();

    match expect(parser.pop().and_then(|e| Some(e.kind)), TokenType::Comma, travel_tok.location.clone()){
        Ok(_) => {},
        Err(e) => {
            parser.skip(1);
            return Err(vec![e]);
        }
    }

    let y_ident = parser.pop();
    match expect(y_ident.and_then(|e| Some(e.kind)), TokenType::Ident, travel_tok.location.clone()){
        Ok(_) => {},
        Err(e) => {
            parser.skip(1);
            return Err(vec![e]);
        }
    }
    let y_ident = y_ident.unwrap().clone();
    
    match expect(parser.pop().and_then(|e| Some(e.kind)), TokenType::RParen, travel_tok.location.clone()){
        Ok(_) => {},
        Err(e) => {
            parser.skip(1);
            return Err(vec![e]);
        }
    }


    let block = parse_block(parser, semicolon_terminated)?;

    Ok(AST { kind: travel_tok, children: vec![
        AST{ kind: x_ident, children: vec![] },
        AST{ kind: y_ident, children: vec![] },
        block
    ] })
}

fn parse_subcanvas(parser:&mut Parser<TokenType, Token<TokenType>>, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let subcanvas_tok = parser.pop().unwrap().clone();
    let mut errors = vec![];
    let mut children = vec![];

    match parser.slice_block(TokenType::LParen, TokenType::RParen){
        Ok(tokens) => {
            parser.skip(tokens.len()+2);
            let args = split_list(TokenType::Comma, tokens);
            if args.len() != 4 {
                errors.push(ParsingError::UnparsedSequence(subcanvas_tok.location.clone()));
            }

            for param in args{
                if let Some(raw_expr) = parse_expression(&param){
                    match raw_expr{
                        Ok(raw_expr) => {
                            match normalize_expression(raw_expr){
                                Ok(expr) => children.push(expr),

                                Err(errs) => {
                                    for e in errs { errors.push(e); }
                                }
                            }
                        },

                        Err(errs) => {
                            for e in errs { errors.push(e); }
                        }
                    }
                    
                }
            }

            match parse_block(parser, semicolon_terminated){
                Ok(ast) => children.push(ast),
                Err(errs) =>{
                    for e in errs { errors.push(e); }
                }
            }

            if !errors.is_empty(){
                parser.skip(1);
                Err(errors) 
            }
            else{
                Ok(AST{ kind: subcanvas_tok, children })
            }
                    
            
        },

        Err(e) => {
            parser.skip(1);
            Err(vec![e])
        }
    }
}

fn parse_def(parser:&mut Parser<TokenType, Token<TokenType>>, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let def_tok = parser.pop().unwrap().clone();
    let mut def_ast = AST{ kind: def_tok.clone(), children:vec![] };

    let ident_tok = parser.pop();
    
    match expect(ident_tok.and_then(|t| Some(t.kind)), TokenType::Ident, def_tok.location.clone()){
        Ok(_) => {
            let ident_tok = ident_tok.unwrap().clone();
            let mut ident_ast = AST{ kind: ident_tok, children: vec![] };

            match parser.slice_block(TokenType::LParen, TokenType::RParen){
                Ok(tokens) =>{
                    let params = split_list(TokenType::Comma, tokens);
                    let mut errors = vec![];

                    for param in params{
                        match parse_type_binding(&mut Parser::new(&param), false){
                            Ok(ast) => ident_ast.children.push(ast),
                            Err(errs) =>{
                                for e in errs { errors.push(e); }
                            }
                        }
                    }

                    def_ast.children.push(ident_ast);

                    parser.skip(tokens.len()+2);
                    if let Some(token) = parser.peek(){
                        let token = token.clone();
                        if token.kind == TokenType::Colon{
                            parser.skip(1);

                            match expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::Ident, token.location.clone()){
                                Ok(_) => {
                                    let ident = parser.pop().unwrap().clone();
                                    def_ast.children.push(AST { kind: ident, children: vec![] });
                                },

                                Err(e) => errors.push(e)
                            }

                        }

                        match parse_block(parser, semicolon_terminated){
                            Ok(ast) => def_ast.children.push(ast),
                            Err(errs) => {
                                for e in errs { errors.push(e); }
                            }
                        }

                    }else{ errors.push(ParsingError::NoTokens) }

                    if !errors.is_empty(){
                        parser.skip(1);
                        Err(errors)
                    }
                    else{
                        Ok(def_ast)
                    }


                    
                },

                Err(e) =>{
                    parser.skip(1);
                    Err(vec![e])
                }
            }
        },

        Err(e) => {
            parser.skip(1);
            Err(vec![e])
        }
    }
}

fn parse_ident(parser:&mut Parser<TokenType, Token<TokenType>>, semicolon_terminated:bool) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let mut ident_ast = AST{ kind: parser.pop().unwrap().clone(), children: vec![] };
    
    match parser.slice_block(TokenType::LParen, TokenType::RParen){
        Ok(tokens) => {
            let mut errors = vec![];
            let args = split_list(TokenType::Comma, tokens);

            for arg in args{
                if let Some(raw_expr) = parse_expression(&arg){
                    match raw_expr{
                        Ok(raw_expr) => {
                            match normalize_expression(raw_expr){
                                Ok(expr) => ident_ast.children.push(expr),
                                Err(errs) => {
                                    for e in errs { errors.push(e); }
                                }
                            }
                        },
                        Err(errs) => {
                            for e in errs { errors.push(e); }
                        }
                    }

                }else{ errors.push(ParsingError::NoTokens); }
            }

            parser.skip(tokens.len()+2);

            if semicolon_terminated{
                match expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon, ident_ast.kind.location.clone()){
                    Ok(_) => parser.skip(1),
                    Err(e) => errors.push(e)
                }
            }

            if !errors.is_empty(){
                parser.skip(1);
                Err(errors)
            }
            else { Ok(ident_ast) }

        },

        Err(e) => {
            parser.skip(1);
            Err(vec![e])
        }
    }

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

fn normalize_expression(expr:AST<Expr<TokenType>>) -> Result<AST<Token<TokenType>>, Vec<ParsingError<TokenType>>>{
    let mut normalized:AST<Token<TokenType>>;

    normalized = match expr.kind{
        Expr::Operator(t) => {
            if t.kind == TokenType::Def
            || t.kind == TokenType::While
            || t.kind == TokenType::If
            || t.kind == TokenType::Subcanvas
            || t.kind == TokenType::Travel
            || t.kind == TokenType::Colon
            || t.kind == TokenType::Eq{
                return Err(vec![ParsingError::UnparsedSequence(t.location.clone())]);
            }
            AST{kind: t, children: vec![]}
        },
        Expr::Operand(t) => {
            if t.kind == TokenType::Def
            || t.kind == TokenType::While
            || t.kind == TokenType::If
            || t.kind == TokenType::Subcanvas
            || t.kind == TokenType::Travel
            || t.kind == TokenType::Colon
            || t.kind == TokenType::Eq{
                return Err(vec![ParsingError::UnparsedSequence(t.location.clone())]);
            }
            AST{kind: t, children: vec![]}
        },
        Expr::Unknown(tokens) => {
            let forest = parse(tokens, false)?;
            if forest.len() > 1 { return Err(vec![ParsingError::UnparsedSequence(tokens[0].location.clone())]) }

            if forest[0].kind.kind == TokenType::Def
            || forest[0].kind.kind == TokenType::While
            || forest[0].kind.kind == TokenType::If
            || forest[0].kind.kind == TokenType::Subcanvas
            || forest[0].kind.kind == TokenType::Travel
            || forest[0].kind.kind == TokenType::Colon
            || forest[0].kind.kind == TokenType::Eq{
                return Err(vec![ParsingError::UnparsedSequence(tokens[0].location.clone())]);
            }

            forest[0].clone()
        }
    };

    for e in expr.children{
        let child = normalize_expression(e)?;
        normalized.children.push(child);
    }

    Ok(normalized)


}

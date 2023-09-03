use neoglot_lib::{lexer::Token, parser::{Parser, AST, expression::{ExpressionParser, Expr, Operator, Position}, expect}, regex::{Regex, RegexElement, Quantifier}, report};

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

fn return_regex() -> Regex<TokenType>{
    Regex::new()
        .then(RegexElement::Item(TokenType::Return, Quantifier::Exactly(1)))
        .then(RegexElement::NoneOf(vec![
            RegexElement::Item(TokenType::SemiColon, Quantifier::Exactly(1))
        ], Quantifier::ZeroOrMany))
        .then(RegexElement::Item(TokenType::SemiColon, Quantifier::Exactly(1)))
}

fn dot_access_regex() -> Regex<TokenType>{
    Regex::new()
        .then(RegexElement::Item(TokenType::Ident, Quantifier::Exactly(1)))
        .then(RegexElement::Item(TokenType::Dot, Quantifier::Exactly(1)))
}



pub fn parse(tokens:&[Token<TokenType>], semicolon_terminated:bool) -> Option<Vec<AST<Token<TokenType>>>>{
    let mut forest:Vec<AST<Token<TokenType>>> = vec![];
    let mut sucess = true;

    let mut parser:Parser<TokenType> = Parser::new(tokens);

    while !parser.finished(){
        if parser.on_token(TokenType::SingleComment){
            parser.skip(1);
            continue;
        }

        if parser.on_token(TokenType::Import){
            let next = parser.peek_at(1);
            if !expect(next.and_then(|e| Some(e.kind)), TokenType::String){
                report("Expected a string", parser.peek().unwrap().location.clone());
                parser.skip(1);
                sucess = false;
            }else{
                let import_tok = parser.pop().unwrap().clone();
                let str_tok = parser.pop().unwrap().clone();

                if ! expect(parser.peek().and_then(|e| Some(e.kind)), TokenType::SemiColon){
                    report("Expected ';' at the end", import_tok.location.clone());
                    sucess = false;
                    parser.skip(1);
                }else{
                    forest.push(AST{
                        kind: import_tok,
                        children : vec![
                            AST{kind: str_tok, children: vec![]}
                        ]
                    });
                    parser.skip(1);

                }
            }
        
        }else if parser.on_token(TokenType::Pub){
            let next = parser.peek_at(1);
            if !expect(next.and_then(|e| Some(e.kind)), TokenType::Def){
                report("Visibility modifier only accepted on function declaration", parser.peek().unwrap().location.clone());
                parser.skip(1);
                sucess = false;
            }else{
                let pub_tok = parser.peek().unwrap().clone();
                parser.skip(1);
                match parse_def(&mut parser, semicolon_terminated){
                    Some(ast) => {
                        forest.push(AST{
                            kind: pub_tok,
                            children: vec![ast]
                        })
                    },
                    None => sucess = false
                }
            }
        
        }else if parser.on_regex(&typed_var_assign_regex()){
            match parse_typed_var_assign(&mut parser){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }

        }else if parser.on_regex(&inferred_var_assign_regex()){
            match parse_inferred_var_assign(&mut parser){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }
            
        }else if parser.on_regex(&type_binding_regex()){
            match parse_type_binding(&mut parser, semicolon_terminated){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }

        }else if parser.on_regex(&return_regex()){
            match parse_return(&mut parser){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }
        }else if parser.on_regex(&dot_access_regex()){
            match parse_dot_access(&mut parser){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }
            
        }else if parser.on_token(TokenType::If){
            match parse_if(&mut parser, semicolon_terminated){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }

        }else if parser.on_token(TokenType::While){
            match parse_while(&mut parser, semicolon_terminated){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }

        }else if parser.on_token(TokenType::Travel){
            match parse_travel(&mut parser, semicolon_terminated){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }

        }else if parser.on_token(TokenType::Subcanvas){
            match parse_subcanvas(&mut parser, semicolon_terminated){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }

        }else if parser.on_token(TokenType::Def){
            match parse_def(&mut parser, semicolon_terminated){
                Some(ast) => forest.push(ast),
                None => sucess = false
            }

        }else if parser.on_token(TokenType::Ident){
            match parse_ident(&mut parser, semicolon_terminated) {
                Some(ast) => forest.push(ast),
                None => sucess = false
            }

        }else{
            let got = parser.peek().unwrap().clone();
            report(&format!("Unexpected token: '{}'", got.literal), got.location);
            sucess = false;
            parser.skip(1);
        }
    }

    if !sucess{ None }
    else { Some(forest) }
}

fn parse_dot_access(parser:&mut Parser<TokenType>) -> Option<AST<Token<TokenType>>>{
    let mut tokens = vec![];
    let mut semicolon_terminated = false;
    while let Some(token) = parser.pop(){
        if token.kind == TokenType::SemiColon{
            semicolon_terminated = true;
            break;
        }

        tokens.push(token.clone());
    }

    if !semicolon_terminated{
        report("Expected ';' at the end", tokens[tokens.len()-1].location.clone());
        parser.skip(1);
        return None;
    }

    let raw_expr = parse_expression(&tokens);

    if raw_expr.is_none(){
        report("Could not parse expression", tokens[0].location.clone());
        parser.skip(1);
        return None;
    }

    let normalized = match normalize_expression(raw_expr.unwrap()){
        Some(expr) => Some(expr),
        None => None
    };

    if normalized.is_none(){
        return None;
    }

    let normalized = normalized.unwrap();
    let mut valid = true;
    if normalized.kind.kind != TokenType::Dot{
        report("This is not a statement", tokens[0].location.clone());
        valid = false;
    }

    if normalized.children.len() != 2{
        report("The dot operator only takes two operand", tokens[0].location.clone());
        valid = false;
    }

    if valid{
        Some(normalized)
    }else{ None }
}


fn parse_typed_var_assign(parser:&mut Parser<TokenType>) -> Option<AST<Token<TokenType>>>{
    match parser.slice_regex(&typed_var_assign_regex()){
        Some(tokens) => {
            
            let raw_expr = parse_expression(&tokens[4..tokens.len()]);
            if raw_expr.is_none(){
                parser.skip(tokens.len());
                report("Could not parse expression", tokens[4].location.clone());

                if !expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon){
                    report("Expected ';' at the end", tokens[tokens.len()-1].location.clone());
                }

                parser.skip(1);

                return None;
            }
            let normalized = match normalize_expression(raw_expr.unwrap()){
                Some(expr) => Some(expr),
                None => None
            };

            if normalized.is_none(){
                parser.skip(tokens.len());

                if !expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon){
                    report("Expected ';' at the end", tokens[tokens.len()-1].location.clone());
                }
                parser.skip(1);

                return None;
            }

            let tree = AST{kind: tokens[3].clone(), children: vec![
                AST{ kind: tokens[1].clone(), children: vec![
                    AST{ kind: tokens[0].clone(), children: vec![] },
                    AST{ kind: tokens[2].clone(), children: vec![] }
                ] },
                normalized.unwrap()
            ]};

            

            parser.skip(tokens.len());

            let r = if !expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon){
                report("Expected ';' at the end", tokens[tokens.len()-1].location.clone());
                None
            }else{
                Some(tree)
            };

            parser.skip(1);
            r

            
        },

        None =>{
            report("Expected sequence 'identifier:identifier = <expr>;'", parser.peek().unwrap().location.clone());
            parser.skip(1);
            None
        }
    }
}

fn parse_inferred_var_assign(parser:&mut Parser<TokenType>) -> Option<AST<Token<TokenType>>>{
    match parser.slice_regex(&inferred_var_assign_regex()){
        Some(tokens) => {
            let raw_expr = parse_expression(&tokens[2..tokens.len()]);

            if raw_expr.is_none(){
                parser.skip(tokens.len());
                report("Could not parse expression", tokens[2].location.clone());
                
                if !expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon){
                    report("Expected ';' at the end", tokens[tokens.len()-1].location.clone());
                }

                parser.skip(1);

                return None;
            }

            let normalized = match normalize_expression(raw_expr.unwrap()) {
                Some(expr) => Some(expr),
                None => None
            };

            if normalized.is_none(){
                parser.skip(tokens.len());

                if !expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon){
                    report("Expected ';' at the end", tokens[tokens.len()-1].location.clone());
                }

                parser.skip(1);

                return None;
            }

            let tree = AST{kind: tokens[1].clone(), children: vec![
                AST{ kind: tokens[0].clone(), children: vec![] },
                normalized.unwrap()
            ]};

            parser.skip(tokens.len());

            let r = if !expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon){
                report("Expected ';' at the end", tokens[tokens.len()-1].location.clone());
                None
            }else{
                Some(tree)
            };

            parser.skip(1);
            r

        },
        None => {
            report("Expected sequence 'identifier = <expr>;'", parser.peek().unwrap().location.clone());
            parser.skip(1);
            None
        }
    }
}

fn parse_return(parser:&mut Parser<TokenType>) -> Option<AST<Token<TokenType>>>{
    match parser.slice_regex(&return_regex()){
        Some(tokens) => {
            parser.skip(tokens.len());
            let expr = tokens.get(1..tokens.len()-1).unwrap_or_default();

            if expr.is_empty(){
                Some(AST { kind: tokens[0].clone(), children: vec![] })
            }else{
                match parse_expression(expr){
                    Some(expr) => {
                        Some(AST { kind: tokens[0].clone(), children: vec![normalize_expression(expr)?] })
                    },

                    None => {
                        report("Could not parse expression", tokens[0].location.clone());
                        None
                    }
                }
            }
        },

        None => {
            report("Expected sequence 'return [<expr>];'", parser.peek().unwrap().location.clone());
            parser.skip(1);
            None
        }
    }

    
}

fn parse_type_binding(parser:&mut Parser<TokenType>, semicolon_terminated:bool) -> Option<AST<Token<TokenType>>> {
    match parser.slice_regex(&type_binding_regex()){
        Some(tokens) => {
            let tree = AST{ kind: tokens[1].clone(), children: vec![
                AST{ kind: tokens[0].clone(), children: vec![] },
                AST{ kind: tokens[2].clone(), children: vec![] }
            ] };

            parser.skip(tokens.len());

            if semicolon_terminated{
                if expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon){
                    parser.skip(1);
                    Some(tree)
                }else{
                    match parser.peek(){
                        Some(tok) => {
                            report(&format!("Expected ';' but instead got '{}'", tok.literal), tok.location.clone());
                        },
    
                        None => {
                            report("Expected ';' at the end", tokens[0].location.clone());
                        }
                    }
                    parser.skip(1);
                    None
                }
            }else { Some(tree) }
        },

        None => {
            report("Expected sequence 'identifier:identifier''", parser.peek().unwrap().location.clone());
            parser.skip(1);
            None
        }
    }
}

fn parse_if(parser:&mut Parser<TokenType>, semicolon_terminated:bool) -> Option<AST<Token<TokenType>>>{
    let if_tok = parser.pop().unwrap().clone();
    match parser.slice_block(TokenType::LParen, TokenType::RParen){
        Some(tokens) =>{
            let r_paren = parser.peek_at(tokens.len()+1).unwrap().clone();
            parser.skip(tokens.len()+2);
            if let Some(raw_expr) = parse_expression(tokens){
                match normalize_expression(raw_expr){
                    Some(expr) => {

                        if !expect(parser.peek().and_then(|e| Some(e.kind)), TokenType::LBracket){
                            report("Expected block '{...}'", r_paren.location.clone());
                            parser.skip(1);
                            return None;
                        }

                        let block = parse_block(parser, semicolon_terminated)?;

                        if expect(parser.peek().and_then(|e| Some(e.kind)), TokenType::Else){
                            Some(AST { kind: if_tok, children: vec![
                                expr,
                                block,
                                parse_else(parser, semicolon_terminated)?
                            ] })
                        }else{
                            Some(AST { kind: if_tok, children: vec![
                                expr,
                                block
                            ] })
                        }
                    },

                    None => {
                        parser.skip(1);
                        None
                    }
                }

                
            }else{
                report("Could not parse expression", if_tok.location.clone());
                None
            }

        },

        None => {
            report("Expected sequence '(<expr>)'", if_tok.location.clone());
            None
        }
    }
}

fn parse_else(parser:&mut Parser<TokenType>, semicolon_terminated:bool) -> Option<AST<Token<TokenType>>>{
    let else_tok = parser.pop().unwrap().clone();
    if let Some(token) = parser.peek(){
        if token.kind == TokenType::If{
            Some(AST{ kind: else_tok.clone(), children: vec![parse_if(parser, semicolon_terminated)?] })
        }else{
            Some(AST{ kind: else_tok.clone(), children: vec![parse_block(parser, semicolon_terminated)?] })
        }

    }else{
        report("Expected block '{...} or 'if' structure", else_tok.location.clone());
        parser.skip(1);
        None
    }
}

fn parse_while(parser:&mut Parser<TokenType>, semicolon_terminated:bool) -> Option<AST<Token<TokenType>>>{
    let while_tok = parser.pop().unwrap().clone();
    match parser.slice_block(TokenType::LParen, TokenType::RParen){
        Some(tokens) =>{
            let r_paren = parser.peek_at(tokens.len()+1).unwrap().clone();
            parser.skip(tokens.len()+2);
            if let Some(raw_expr) = parse_expression(tokens){
                match normalize_expression(raw_expr){
                    Some(expr) => {

                        if !expect(parser.peek().and_then(|e| Some(e.kind)), TokenType::LBracket){
                            report("Expected block '{...}'", r_paren.location.clone());
                            parser.skip(1);
                            return None;
                        }

                        let block = parse_block(parser, semicolon_terminated)?;

                        Some(AST { kind: while_tok, children: vec![
                            expr,
                            block
                        ] })
                    },

                    None => {
                        parser.skip(1);
                        None
                    }
                }

                
            }else{
                report("Could not parse expression", while_tok.location.clone());
                None
            }

        },

        None => {
            report("Expected sequence '(<expr>)'", while_tok.location.clone());
            None
        }
    }
}

fn parse_travel(parser:&mut Parser<TokenType>, semicolon_terminated:bool) -> Option<AST<Token<TokenType>>>{
    let travel_tok = parser.pop().unwrap().clone();

    let l_paren = parser.pop();
    if !expect(l_paren.and_then(|e| Some(e.kind)), TokenType::LParen){
        match l_paren{
            Some(tok) => {
                report(&format!("Expected '(' but instead got '{}'", tok.literal), tok.location.clone());
            },

            None => {
                report("Expected '('", travel_tok.location.clone());
            }
        }
        return None;
    }

   

    let x_ident = parser.pop();
    if !expect(x_ident.and_then(|e| Some(e.kind)), TokenType::Ident){
        match x_ident{
            Some(tok) => {
                report(&format!("Expected identifier but instead got '{}'", tok.literal), tok.location.clone());
            },

            None => {
                report("Expected identifier", travel_tok.location.clone());
            }
        }
        return None;
    }
    let x_ident = x_ident.unwrap().clone();

    let comma = parser.pop();
    if !expect(comma.and_then(|e| Some(e.kind)), TokenType::Comma){
        match comma{
            Some(tok) => {
                report(&format!("Expected ',' but instead got '{}'", tok.literal), tok.location.clone());
            },

            None => {
                report("Expected ','", x_ident.location.clone());
            }
        }
        return None;
    }

    let y_ident = parser.pop();
    if !expect(y_ident.and_then(|e| Some(e.kind)), TokenType::Ident){
        match y_ident{
            Some(tok) => {
                report(&format!("Expected identifier but instead got '{}'", tok.literal), tok.location.clone());
            },

            None => {
                report("Expected identifier", x_ident.location.clone());
            }
        }
        return None;
    }
    let y_ident = y_ident.unwrap().clone();
    
    let r_paren = parser.pop();
    if !expect(r_paren.and_then(|e| Some(e.kind)), TokenType::RParen){
        match r_paren{
            Some(tok) => {
                report(&format!("Expected ')' but instead got '{}'", tok.literal), tok.location.clone());
            },

            None => {
                report("Expected ')'", x_ident.location.clone());
            }
        }
        return None;
    }
    let r_paren = r_paren.unwrap().clone();

    if !expect(parser.peek().and_then(|e| Some(e.kind)), TokenType::LBracket){
        report("Expected block '{...}'", r_paren.location.clone());
        return None;
    }

    let block = parse_block(parser, semicolon_terminated)?;

    Some(AST { kind: travel_tok, children: vec![
        AST{ kind: x_ident, children: vec![] },
        AST{ kind: y_ident, children: vec![] },
        block
    ] })
}

fn parse_subcanvas(parser:&mut Parser<TokenType>, semicolon_terminated:bool) -> Option<AST<Token<TokenType>>>{
    let subcanvas_tok = parser.pop().unwrap().clone();
    let mut success = true;
    let mut children = vec![];

    match parser.slice_block(TokenType::LParen, TokenType::RParen){
        Some(tokens) => {
            let r_paren = parser.peek_at(tokens.len()+1).unwrap().clone();
            parser.skip(tokens.len()+2);

            let args = split_list(TokenType::Comma, tokens);
            if args.is_none(){
                report("Invalid arguments list", subcanvas_tok.location.clone());
                success = false;
            }else{
                let args = args.unwrap();
                if args.len() != 4 {
                    success = false;
                    report(&format!("subcanvas takes 4 arguments but {} were provided", args.len()), subcanvas_tok.location.clone());
                }

                for param in args{
                    if let Some(raw_expr) = parse_expression(&param){
                        match normalize_expression(raw_expr){
                            Some(expr) => children.push(expr),
    
                            None => success = false
                        }
                        
                    }else{
                        report("Could not parse expression", param[0].location.clone());
                        success = false;
                    }
                }
            }
            

            if !expect(parser.peek().and_then(|e| Some(e.kind)), TokenType::LBracket){
                report("Expected block '{...}'", r_paren.location.clone());
                return None;
            }

            children.push(parse_block(parser, semicolon_terminated)?);

            if success{ Some(AST{ kind: subcanvas_tok, children }) }
            else{ None }
                    
            
        },

        None => {
            report("Expected sequence '(<expr>, <expr>, <expr>, <expr>)'", subcanvas_tok.location.clone());
            parser.skip(1);
            None
        }
    }
}

fn parse_def(parser:&mut Parser<TokenType>, semicolon_terminated:bool) -> Option<AST<Token<TokenType>>>{
    let def_tok = parser.pop().unwrap().clone();
    let mut def_ast = AST{ kind: def_tok.clone(), children:vec![] };

    let ident_tok = parser.pop();
    
    if expect(ident_tok.and_then(|t| Some(t.kind)), TokenType::Ident){
        let ident_tok = ident_tok.unwrap().clone();
        let mut ident_ast = AST{ kind: ident_tok.clone(), children: vec![] };

        match parser.slice_block(TokenType::LParen, TokenType::RParen){
            Some(tokens) =>{
                parser.skip(tokens.len()+2);
                let mut success = true;
                let params = split_list(TokenType::Comma, tokens);

                if params.is_none(){
                    report("Invalid parameters list", def_tok.location.clone());
                    success = false;
                }

                for param in params.unwrap_or_default(){
                    match parse_type_binding(&mut Parser::new(&param), false){
                        Some(ast) => ident_ast.children.push(ast),
                        None =>{ success = false; }
                    }
                }

                def_ast.children.push(ident_ast);

                if let Some(token) = parser.peek(){
                    let token = token.clone();
                    if token.kind == TokenType::Colon{
                        parser.skip(1);

                        if expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::Ident){
                            let ident = parser.pop().unwrap().clone();
                            def_ast.children.push(AST { kind: ident, children: vec![] });
                        }else{
                            match parser.peek(){
                                Some(tok) => {
                                    report(&format!("Expected identifier but instead got '{}'", tok.literal), tok.location.clone());
                                    parser.skip(1);
                                    return None;
                                },

                                None => {
                                    report("Expected identifier", def_tok.location.clone());
                                    parser.skip(1);
                                    return None;
                                }
                            }
                        }

                    }

                    def_ast.children.push(parse_block(parser, semicolon_terminated)?);

                }else{
                    report("Expected return type or body", def_tok.location.clone());
                    parser.skip(1);
                    return None;
                }

                if success{ Some(def_ast) }
                else{ None }


                    
            },

            None =>{
                report("Expected sequence '(identifier:identifier, identifier:identifier...)'", def_tok.location.clone());
                parser.skip(1);
                None
            }
        }
    }else{
        match ident_tok{
            Some(tok) => {
                report(&format!("Expected identifier but instead got '{}'", tok.literal), tok.location.clone());
            },

            None => {
                report("Expected identifier", def_tok.location.clone());
            }
        }
        parser.skip(1);
        None
    }
}

fn parse_ident(parser:&mut Parser<TokenType>, semicolon_terminated:bool) -> Option<AST<Token<TokenType>>>{
    let mut ident_ast = AST{ kind: parser.pop().unwrap().clone(), children: vec![] };

    
    match parser.slice_block(TokenType::LParen, TokenType::RParen){
        Some(tokens) => {
            let mut args_ast = AST{kind: parser.peek().unwrap().clone(), children: vec![]};
            parser.skip(tokens.len()+2);

            let mut success = true;
            let args = split_list(TokenType::Comma, tokens);

            if args.is_none(){
                report("Invalid arguments list", ident_ast.kind.location.clone());
                success = false;
            }

            for arg in args.unwrap_or_default(){
                if let Some(raw_expr) = parse_expression(&arg){
                    match normalize_expression(raw_expr){
                        Some(expr) => args_ast.children.push(expr),
                        None => {
                            success = false;
                        }
                    }

                }else{
                    report("Could not parse expression", arg[0].location.clone());
                    success = false;
                }
            }

            ident_ast.children.push(args_ast);

            if semicolon_terminated{
                if expect(parser.peek().and_then(|t| Some(t.kind)), TokenType::SemiColon){
                    parser.skip(1);
                }else{
                    match parser.peek(){
                        Some(tok) => {
                            report(&format!("Expected ';' but instead got '{}'", tok.literal), tok.location.clone());
                        },

                        None => {
                            report("Expected ';'", ident_ast.kind.location.clone());
                        }
                    }
                    parser.skip(1);
                    return None;
                }
            }

            if success{ Some(ident_ast) }
            else { None }

        },

        None => {
            report("Expected sequence '(<expr>, <expr>...)'", ident_ast.kind.location.clone());
            parser.skip(1);
            None
        }
    }

}

fn split_list(delimiter:TokenType, tokens: &[Token<TokenType>]) -> Option<Vec<Vec<Token<TokenType>>>>{
    let mut list:Vec<Vec<Token<TokenType>>> = vec![];
    let mut param = vec![];
    let mut open_paren = 0;
    let mut num_delimiters = 0;

    for token in tokens{
        if token.kind == delimiter{
            if open_paren != 0{ param.push(token.clone()) }
            else{
                num_delimiters += 1;
                if !param.is_empty(){
                    list.push(param.clone());
                    param.clear();
                }
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

    let valid =if list.len() == 0{
        num_delimiters == 0
    }else{
        num_delimiters+1 == list.len()
    };

    if valid { Some(list) }
    else { None }
}

fn parse_block(parser:&mut Parser<TokenType>, semicolon_terminated:bool) -> Option<AST<Token<TokenType>>>{
    let block_begin = parser.peek();
    match parser.slice_block(TokenType::LBracket, TokenType::RBracket){
        Some(tokens) => {
            let ast = AST{ kind: block_begin.unwrap().clone(), children: parse(tokens, semicolon_terminated)? };

            parser.skip(tokens.len()+2);

            Some(ast)
        },

        None => {
            report("Expected a block: '{...}'", parser.peek().unwrap().location.clone());
            parser.skip(1);
            None
        }
    }
}



fn parse_expression(tokens: &[Token<TokenType>]) -> Option<AST<Expr<TokenType>>>{
    let mut parser = ExpressionParser::new();
    

    
    parser.add_operator(Operator{kind: TokenType::DoubleEq, position: Position::Infix}, 1);
    parser.add_operator(Operator{kind: TokenType::NotEq, position: Position::Infix}, 1);

    parser.add_operator(Operator { kind: TokenType::Or, position: Position::Infix }, 2);
    parser.add_operator(Operator { kind: TokenType::And, position: Position::Infix }, 3);
    
    parser.add_operator(Operator{kind: TokenType::GT, position: Position::Infix}, 4);
    parser.add_operator(Operator{kind: TokenType::LT, position: Position::Infix}, 4);
    parser.add_operator(Operator{kind: TokenType::GTEq, position: Position::Infix}, 4);
    parser.add_operator(Operator{kind: TokenType::LTEq, position: Position::Infix}, 4);
    
    parser.add_operator(Operator{kind: TokenType::Not, position: Position::Prefix}, 5);

    

    parser.add_operator(Operator{kind: TokenType::Plus, position: Position::Infix}, 6);
    parser.add_operator(Operator{kind: TokenType::Minus, position: Position::Infix}, 6);
    
    parser.add_operator(Operator{kind: TokenType::Mul, position: Position::Infix}, 7);
    parser.add_operator(Operator{kind: TokenType::Div, position: Position::Infix}, 7);

    parser.add_operator(Operator{kind: TokenType::Mod, position: Position::Infix}, 8);
    parser.add_operator(Operator{kind: TokenType::Pow, position: Position::Infix}, 8);

    parser.add_operator(Operator { kind: TokenType::Dot, position: Position::Infix }, 9);
    



    


    parser.set_high_priority_group(TokenType::LParen, TokenType::RParen);

    parser.parse(tokens, &|parser, tokens| {
        tokens[0].kind == TokenType:: Ident && parser.is_in_group(tokens.get(1..).unwrap_or_default())
    })
}

fn illegal_in_expression(kind:TokenType) -> bool{
    kind == TokenType::Def
    || kind == TokenType::While
    || kind == TokenType::If
    || kind == TokenType::Subcanvas
    || kind == TokenType::Travel
    || kind == TokenType::Colon
    || kind == TokenType::Eq
}

fn normalize_expression(expr:AST<Expr<TokenType>>) -> Option<AST<Token<TokenType>>>{
    let mut normalized:AST<Token<TokenType>>;

    normalized = match expr.kind{
        Expr::Operator(t) => {
            if illegal_in_expression(t.kind){
                report(&format!("Illegal token in expression: '{}'", t.literal), t.location);
                return None;
            }
            AST{kind: t, children: vec![]}
        },
        Expr::Operand(t) => {
            if illegal_in_expression(t.kind){
                report(&format!("Illegal token in expression: '{}'", t.literal), t.location);
                return None;
            }
            AST{kind: t, children: vec![]}
        },
        Expr::Unknown(tokens) => {
            let forest = parse(tokens, false)?;
            if forest.len() > 1 {
                report("Invalid expression", forest[0].kind.location.clone());
                return None;
            }

            if illegal_in_expression(forest[0].kind.kind){
                report(&format!("Illegal token in expression: '{}'", forest[0].kind.literal), forest[0].kind.location.clone());
                return None;
            }

            forest[0].clone()
        }
    };

    for e in expr.children{
        let child = normalize_expression(e)?;
        normalized.children.push(child);
    }

    Some(normalized)


}

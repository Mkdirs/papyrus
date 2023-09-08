use std::{path::{Path, PathBuf}, collections::HashSet, f32::consts::E};

use neoglot_lib::{parser, lexer::{Token, LexingResult, Location}, report};

use crate::{TokenType, environment::{Environment, Type, FuncSign}, tokenize};

type AST = parser::AST<Token<TokenType>>;

pub fn verify(forest: &[AST], requester:Option<&Location>, env: &mut Environment) -> bool{
    let mut success = true;

    for tree in forest{
        if tree.kind.kind == TokenType::Colon{
            if !verify_binding(tree, env){
                success = false;
            }else{
                let _type = get_binding_type(tree).unwrap();
                env.push_var(&tree.children[0].kind.literal, _type);
            }

        }else if tree.kind.kind == TokenType::Eq{
            if !verify_assign(tree, env){ success = false; }
        
        }else if tree.kind.kind == TokenType::Ident{
            if !verify_func_call(tree, false, env){ success = false; }
        
        }else if tree.kind.kind == TokenType::Return{
            if !verify_return(tree, env){ success = false; }
        
        }else if tree.kind.kind == TokenType::If{
            if !verify_if(tree, env){ success = false; }

        }else if tree.kind.kind == TokenType::While{
            if !verify_while(tree, env){ success = false; }

        }else if tree.kind.kind == TokenType::Def{
            if !verify_def(tree, false, env){ success = false; }

        }else if tree.kind.kind == TokenType::Travel{
            if !verify_travel(tree, env){ success = false; }

        }else if tree.kind.kind == TokenType::Subcanvas{
            if !verify_subcanvas(tree, env){ success = false; }

        } else if tree.kind.kind == TokenType::Pub{
            if !verify_def(&tree.children[0], true, env){ success = false; }

        }else if tree.kind.kind == TokenType::Import{
            if !verify_import(tree, requester, env){ success = false; }

        }else if tree.kind.kind == TokenType::Dot{
            if !verify_dot(tree, env){ success = false; }

        }else{
            report("Unhandled case in validating process", tree.kind.location.clone());
            success = false;
        }
    }

    success
}



fn verify_binding(binding_tree:&AST, env:&Environment) -> bool{
    let mut valid = true;
    let name = &binding_tree.children[0].kind;
    let _type = &binding_tree.children[1].kind;

    if env.scope_level == 0 {
        report("This statement is illegal in this scope", binding_tree.kind.location.clone());
        valid = false;
    }

    if env.has_var(&name.literal){
        report(&format!("Variable '{}' already exists", name.literal), name.location.clone());
        valid = false;
    }

    if env.has_import(&name.literal){
        report(&format!("The name '{}' is already taken", name.literal), name.location.clone());
        valid = false;
    }

    if ! env.has_type(&_type.literal){
        report(&format!("Unknown type '{}'", _type.literal), name.location.clone());
        valid = false;
    }


    /*let t = if &_type.literal == "int"{
        Some(Type::Int)
    }else if &_type.literal == "float"{
        Some(Type::Float)
    }else if &_type.literal == "bool"{
        Some(Type::Bool)
    }else if &_type.literal == "color"{
        Some(Type::Color)
    }else{ None };

    if let Some(t) = t{
        //env.push_var(&name.literal, t);
    }*/

    valid
}

fn verify_assign(assign_tree:&AST, env:&mut Environment) -> bool{
    let mut valid = true;

    let left = &assign_tree.children[0];
    let expr = &assign_tree.children[1];
    let mut expected_type = Type::Void;
    let mut push_new_var = false;
    let mut var_name = "";

    if env.scope_level == 0 {
        report("This statement is illegal in this scope", assign_tree.kind.location.clone());
        valid = false;
    }

    if left.kind.kind == TokenType::Colon{
        push_new_var = true;
        var_name = &left.children[0].kind.literal;

        valid = verify_binding(left, env);
        if valid{
            expected_type = get_binding_type(left).unwrap();
        }

    }else if !env.has_var(&left.kind.literal){
        report(&format!("Variable '{}' does not exists", left.kind.literal), left.kind.location.clone());
        valid = false;
    }else{
        expected_type = env.get_var(&left.kind.literal).unwrap().0;
    }

    if valid{
        if let Some(result_type) = get_expr_return_type(expr, env){
            if expected_type != result_type{
                report(&format!("Expected type '{:?}' but instead got '{:?}'", expected_type, result_type), left.kind.location.clone());
                valid = false;
            }else if push_new_var{
                env.push_var(var_name, expected_type);
            }
        }else{ valid = false; }
    }

    valid
}

fn verify_func_call(func_call_tree: &AST, foreign:bool, env:&Environment) -> bool{
    let name = func_call_tree.kind.literal.clone();
    let mut params = vec![];
    let mut valid = true;

    if env.scope_level == 0 {
        report("This statement is illegal in this scope", func_call_tree.kind.location.clone());
        valid = false;
    }

    for arg in &func_call_tree.children[0].children{
        let t = get_expr_return_type(&arg, env);

        if t.is_none(){ valid = false; }
        else{ params.push(t.unwrap()); }
    }

    let func_sign = FuncSign{ name, params };

    if !env.has_func_sign(&func_sign) && !foreign{
        report(&format!("Function '{}' does not exists", func_sign), func_call_tree.kind.location.clone());
        valid = false;

    }else if env.has_ctx("in_travel") || env.has_ctx("in_subcanvas"){
        if func_sign == (FuncSign{ name: "create_canvas".to_string(), params: vec![Type::Int, Type::Int]})
        || func_sign == (FuncSign{ name: "save_canvas".to_string(), params: vec![]}){
            report("This function is not allowed in this scope", func_call_tree.kind.location.clone());
            valid = false;
        }
    }

    valid
}

fn verify_return(return_tree: &AST, env:&Environment) -> bool{
    let mut valid = true;

    if env.scope_level == 0 {
        report("This statement is illegal in this scope", return_tree.kind.location.clone());
        valid = false;
    }

    let mut return_type:Type = Type::Void;

    if !return_tree.children.is_empty(){
        let expr = &return_tree.children[0];

        if !verify_expr(expr, env){
            valid = false;
        }else if let Some(_type) = get_expr_return_type(expr, env){
            return_type = _type;
        }else{ valid = false; }
    }

    if env.has_var("?exit_type"){
        let exit_type = env.get_var("?exit_type").unwrap().0;
        if return_type != exit_type{
            report(&format!("Expected type '{:?}' but instead got '{:?}'", exit_type, return_type), return_tree.kind.location.clone());
            valid = false;
        }

    }else if return_type != Type::Void{
        report("No return value expected", return_tree.kind.location.clone());
        valid = false;
    }

    valid
}

fn verify_if(if_tree:&AST, env:&Environment) -> bool{
    let expr = &if_tree.children[0];
    let block = &if_tree.children[1];

    let mut valid = true;

    if env.scope_level == 0 {
        report("This statement is illegal in this scope", if_tree.kind.location.clone());
        valid = false;
    }
    
    
    if !verify_expr(expr, env){
        valid = false;
    }else if let Some(_type) = get_expr_return_type(expr, env){
        if _type != Type::Bool{
            report(&format!("Expected type '{:?}' but instead got '{:?}'", Type::Bool, _type), if_tree.kind.location.clone());
            valid = false;
        }
    }else { valid = false; }
    

    let mut block_env = env.clone();
    block_env.scope_level += 1;
    if !verify(&block.children, None, &mut block_env){ valid = false; }

    if if_tree.children.len() == 3{
        let else_tree = &if_tree.children[2];
        if !verify_else(else_tree, env){ valid = false; }
    }


    valid
}

fn verify_else(else_tree:&AST, env:&Environment) -> bool{
    let mut valid = true;

    let child = &else_tree.children[0];

    if child.kind.kind == TokenType::If{
        if !verify_if(child, env) { valid = false; }
    }else{
        let mut block_env = env.clone();
        block_env.scope_level += 1;

        if !verify(&child.children, None, &mut block_env){ valid = false; }
    }

    valid
}

fn verify_while(while_tree:&AST, env:&Environment) -> bool{
    let expr = &while_tree.children[0];
    let block = &while_tree.children[1];
    let mut valid = true;

    if env.scope_level == 0 {
        report("This statement is illegal in this scope", while_tree.kind.location.clone());
        valid = false;
    }

    if !verify_expr(expr, env){
        valid = false;
    
    }else if let Some(_type) = get_expr_return_type(expr, env){
        if _type != Type::Bool{
            report(&format!("Expected type '{:?}' but instead got '{:?}'", Type::Bool, _type), while_tree.kind.location.clone());
            valid = false;
        }
    }else{ valid = false; }

    let mut block_env = env.clone();
    block_env.scope_level += 1;
    block_env.add_ctx("in_while");

    if ! verify(&block.children, None, &mut block_env){ valid = false; }

    valid
}

fn verify_def(def_tree: &AST, is_public:bool, env:&mut Environment) -> bool{
    let mut valid = true;

    let func_tree = &def_tree.children[0];
    let name = func_tree.kind.literal.clone();
    let params = &func_tree.children;
    let block = &def_tree.children[def_tree.children.len()-1];

    if env.scope_level != 0{
        report("Function declaration is illegal in this scope", def_tree.kind.location.clone());
        valid = false;
    }

    let expected_return_type = if def_tree.children.len() == 3{
        let t = def_tree.children[1].kind.literal.clone();
        if ! env.has_type(&t){
            report(&format!("Unknown type '{}'", t), def_tree.children[1].kind.location.clone());
            valid = false;
            None
        }else{
            Some(get_type(t).unwrap())
        }

    }else{ Some(Type::Void) };

    let mut block_env = env.clone();
    block_env.scope_level = 1;

    for param in params{
        if !verify_binding(param, &block_env){ valid = false; }
        else{
            let t = param.children[1].kind.literal.clone();
            block_env.push_var(&param.children[0].kind.literal, get_type(t).unwrap());
        }
    }

    block_env.push_var("?exit_type", expected_return_type.unwrap_or(Type::Void));
    

    if !verify(&block.children, None, &mut block_env){ valid = false; }

    if let Some(last) = block.children.last(){
        if expected_return_type.unwrap_or(Type::Void) != Type::Void &&  last.kind.kind != TokenType::Return{
            report("Expected a return statement", last.kind.location.clone());
            valid = false;
        }
    }else{
        if expected_return_type.unwrap_or(Type::Void) != Type::Void{
            report("Expected a return statement", def_tree.kind.location.clone());
            valid = false;
        }
    }

    if env.has_import(&name){
        report(&format!("The name '{name}' is already taken"), def_tree.kind.location.clone());
        valid = false;
    }

    if valid{
        let func_sign = FuncSign{
            name,
            params: params.iter().map(|e| get_type(e.children[1].kind.literal.clone()).unwrap()).collect()
        };

        if !env.has_func_sign(&func_sign){
            if is_public{
                env.push_public_func(Path::new(&def_tree.kind.location.file), func_sign.clone(), expected_return_type.unwrap());
            }
            
            env.push_func_sign(func_sign, expected_return_type.unwrap());
        }else{
            report(&format!("Function '{}' already exists", func_sign), def_tree.kind.location.clone());
            valid = false;
        }
    }

    valid
}

fn verify_travel(travel_tree: &AST, env: &Environment) -> bool{
    let mut valid = true;

    let label_x = &travel_tree.children[0].kind.literal;
    let label_y = &travel_tree.children[1].kind.literal;
    let block = &travel_tree.children[2];

    if env.scope_level == 0 {
        report("This statement is illegal in this scope", travel_tree.kind.location.clone());
        valid = false;
    }

    if env.has_var(label_x){
        report(&format!("Name '{}' already exists", label_x), travel_tree.kind.location.clone());
        valid = false;
    }

    if env.has_var(label_y){
        report(&format!("Name '{}' already exists", label_y), travel_tree.kind.location.clone());
        valid = false;
    }

    let mut block_env = env.clone();
    block_env.scope_level += 1;
    block_env.add_ctx("in_travel");

    block_env.push_var(label_x, Type::Int);
    block_env.push_var(label_y, Type::Int);

    if !verify(&block.children, None, &mut block_env){ valid = false; }

    valid
}

fn verify_subcanvas(subcanvas_tree: &AST, env: &Environment) -> bool{
    let mut valid = true;

    let block = subcanvas_tree.children.last().unwrap();

    if env.scope_level == 0 {
        report("This statement is illegal in this scope", subcanvas_tree.kind.location.clone());
        valid = false;
    }

    if env.has_ctx("in_travel"){
        report("subcanvas is illegal in this scope", subcanvas_tree.kind.location.clone());
        valid = false;
    }

    for arg in subcanvas_tree.children[0..4].iter(){
        if !verify_expr(arg, env){
            valid = false;
        }else if let Some(_type) = get_expr_return_type(arg, env){
            if _type != Type::Int{
                report(&format!("Expected type '{:?}' but instead got '{:?}'", Type::Int, _type), arg.kind.location.clone());
                valid = false;
            }
        }else { valid = false; }
    }

    let mut block_env = env.clone();
    block_env.scope_level += 1;
    block_env.add_ctx("in_subcanvas");

    if !verify(&block.children, None, &mut block_env){ valid = false; }

    valid
}

fn verify_import(import_tree: &AST, requester:Option<&Location>, env:&mut Environment) -> bool{
    let has_aliasing = import_tree.children[0].kind.kind == TokenType::As;
    let string_lit = if has_aliasing{
        &import_tree.children[0].children[0].kind.literal

    }else{
        &import_tree.children[0].kind.literal
    };
    let content = string_lit.get(1..string_lit.len()-1).unwrap();

    if env.scope_level != 0{
        report("This statement is not allowed in this scope", import_tree.kind.location.clone());
        return false;
    }


    if Path::new(content).extension().is_some(){
        report("The path should not contain extension", import_tree.kind.location.clone());
        return false;
    }

    let self_path = Path::new(&import_tree.kind.location.file);


    let mut s = String::from(content);
    s.push_str(".pprs");
    let script_path = &if Path::new(&s).is_relative(){
        self_path.parent().unwrap().join(Path::new(&s))
    }else{
        Path::new(&s).to_path_buf()
    };


    if !script_path.exists(){
        report(&format!("The path {} doesn't exist", script_path.display()), import_tree.kind.location.clone());
        return false;
    }

    if !script_path.is_file(){
        report(&format!("The path {} is not a file", script_path.display()), import_tree.kind.location.clone());
        return false;
    }

    let name = if has_aliasing{
        &import_tree.children[0].children[1].kind.literal
    }else{
        script_path.file_stem().unwrap().to_str().unwrap()
    };
    if env.has_import(name){
        report(&format!("The name '{name}' is already taken"), import_tree.kind.location.clone());
        return false;
    }

    if self_path == script_path{
        report("A script cannot self import", import_tree.kind.location.clone());
        return false;
    }

    if let Some(requester) = requester{
        if Path::new(&requester.file) == script_path{
            report("Circular dependency is not allowed", requester.clone());
            report("Circular dependency is not allowed", import_tree.kind.location.clone());
            return false;
        }
    }

    if env.imports().values().any(|e| e == script_path){
        report("You cannot import twice the same script", import_tree.kind.location.clone());
        return false;
    }

    let valid = if !env.cached_imports.contains_key(script_path){
        match tokenize(script_path.to_str().unwrap()){
            LexingResult::Ok(forest) => {
                match crate::parser::parse(&forest, true){
                    Some(forest) => {
                        let mut other_env = Environment::default();
                        let res = verify(&forest, Some(&import_tree.kind.location), &mut other_env);
                        env.cached_imports.insert(script_path.to_path_buf(), forest);
                        env.cached_imports.extend(other_env.cached_imports);
                        env.public_functions.extend(other_env.public_functions);

                        res
                    },
    
                    None =>{
                        report(&format!("Could not parse {}", script_path.display()), import_tree.kind.location.clone());
                        false
                    }
                }
            },
    
            LexingResult::Err(errs) => {
                for e in errs{
                    eprintln!("{}", e);
                }
                false
            }
        }
    }else{
        let mut other_env = Environment::default();
        let res = verify(&env.cached_imports[script_path], Some(&import_tree.kind.location), &mut other_env);
        env.cached_imports.extend(other_env.cached_imports);
        env.public_functions.extend(other_env.public_functions);

        res
    };
    

    if valid{
        env.push_import(name, script_path);
    }

    valid
}

fn verify_dot(dot_tree:&AST, env: &Environment) -> bool{
    get_expr_return_type(dot_tree, env).is_some()
}

fn verify_expr(expr: &AST, env:&Environment) -> bool{

    if expr.kind.kind == TokenType::Dot{
        verify_dot(expr, env)
    
    }else if expr.kind.kind == TokenType::Ident{
        if expr.children.is_empty(){
            if !env.has_var(&expr.kind.literal){
                report(&format!("Variable '{}' does not exists", expr.kind.literal), expr.kind.location.clone());
                return false;
            }
            true
        }else{
            verify_func_call(expr, false, env)
        }


    }else if expr.children.is_empty(){
        true
    }else{
        let mut valid = true;
        for child in &expr.children{
            if !verify_expr(child, env){ valid = false; }
        }

        valid
    }
}

fn get_binding_type(binding_tree:&AST) -> Option<Type>{
    get_type(binding_tree.children[1].kind.literal.clone())
}

pub fn get_type(name: String) -> Option<Type>{
    if &name == "int"{
        Some(Type::Int)
    }else if &name == "float"{
        Some(Type::Float)
    }else if &name == "bool"{
        Some(Type::Bool)
    }else if &name == "color"{
        Some(Type::Color)
    }else{ None }
}

fn get_expr_return_type(expr: &AST, env:&Environment) -> Option<Type>{
    if expr.kind.kind == TokenType::Ident{
        if expr.children.is_empty(){
            if !env.has_var(&expr.kind.literal){
                report(&format!("Variable '{}' does not exists", expr.kind.literal), expr.kind.location.clone());
            }
            env.get_var(&expr.kind.literal).and_then(|e| Some(e.0))
        }else{
            
            if !verify_func_call(expr, false, env){ return None; }

            let name = expr.kind.literal.clone();
            let mut params = vec![];
            for arg in &expr.children[0].children{
                params.push(get_expr_return_type(&arg, env).unwrap());
            }

            let func_sign = FuncSign{ name, params };

            env.get_func_return(&func_sign)
        }
    }else if expr.children.is_empty(){
        match expr.kind.kind{
            TokenType::Int => Some(Type::Int),
            TokenType::Float => Some(Type::Float),
            TokenType::Bool => Some(Type::Bool),
            TokenType::Hex => Some(Type::Color),

            _ => panic!("Unexpected operand")
        }
    }else{

        match expr.kind.kind{
            TokenType::Plus => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Int),
                    (Type::Float, Type:: Float) => Some(Type::Float),

                    (Type::Int, Type::Float) => Some(Type::Float),
                    (Type::Float, Type::Int) => Some(Type::Float),

                    _ => {
                        report(&format!("Operator '+' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::Minus => {
                if expr.children.len() == 1{

                    match get_expr_return_type(&expr.children[0], env)?{
                        Type::Int => Some(Type::Int),
                        Type::Float => Some(Type::Float),

                        t => {
                            report(&format!("Operator unary '-' is not defined for type '{:?}'", t), expr.kind.location.clone());
                            None
                        }
                    }

                }else{
                    let left = get_expr_return_type(&expr.children[0], env)?;
                    let right = get_expr_return_type(&expr.children[1], env)?;
                    match (left, right){
                        (Type::Int, Type::Int) => Some(Type::Int),
                        (Type::Float, Type:: Float) => Some(Type::Float),
    
                        (Type::Int, Type::Float) => Some(Type::Float),
                        (Type::Float, Type::Int) => Some(Type::Float),
    
                        _ => {
                            report(&format!("Operator binary '-' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                            None
                        }
                    }
                }

                
            },

            TokenType::Mul => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Int),
                    (Type::Float, Type:: Float) => Some(Type::Float),

                    (Type::Int, Type::Float) => Some(Type::Float),
                    (Type::Float, Type::Int) => Some(Type::Float),

                    _ => {
                        report(&format!("Operator '*' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::Div => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Int),
                    (Type::Float, Type:: Float) => Some(Type::Float),

                    (Type::Int, Type::Float) => Some(Type::Float),
                    (Type::Float, Type::Int) => Some(Type::Float),

                    _ => {
                        report(&format!("Operator '/' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::Mod => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Int),

                    _ => {
                        report(&format!("Operator '%' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::Pow => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Int),
                    (Type::Float, Type::Float) => Some(Type::Float),

                    (Type::Int, Type::Float) => Some(Type::Float),
                    (Type::Float, Type::Int) => Some(Type::Float),

                    _ => {
                        report(&format!("Operator '%' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::DoubleEq => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Bool),
                    (Type::Float, Type:: Float) => Some(Type::Bool),
                    (Type::Bool, Type::Bool) => Some(Type::Bool),
                    (Type::Color, Type::Color) => Some(Type::Bool),

                    _ => {
                        report(&format!("Operator '==' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::NotEq => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Bool),
                    (Type::Float, Type:: Float) => Some(Type::Bool),
                    (Type::Bool, Type::Bool) => Some(Type::Bool),
                    (Type::Color, Type::Color) => Some(Type::Bool),

                    _ => {
                        report(&format!("Operator '!=' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::And => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Bool, Type::Bool) => Some(Type::Bool),

                    _ => {
                        report(&format!("Operator '&&' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::Or => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Bool, Type::Bool) => Some(Type::Bool),

                    _ => {
                        report(&format!("Operator '||' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::GT => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Bool),
                    (Type::Float, Type:: Float) => Some(Type::Bool),
                    /*(Type::Int, Type::Float) => Some(Type::Bool),
                    (Type::Float, Type::Int) => Some(Type::Bool),*/

                    _ => {
                        report(&format!("Operator '>' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::LT => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Bool),
                    (Type::Float, Type:: Float) => Some(Type::Bool),
                    /*(Type::Int, Type::Float) => Some(Type::Bool),
                    (Type::Float, Type::Int) => Some(Type::Bool),*/

                    _ => {
                        report(&format!("Operator '<' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::GTEq => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Bool),
                    (Type::Float, Type:: Float) => Some(Type::Bool),
                    /*(Type::Int, Type::Float) => Some(Type::Bool),
                    (Type::Float, Type::Int) => Some(Type::Bool),*/

                    _ => {
                        report(&format!("Operator '>=' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::LTEq => {
                let left = get_expr_return_type(&expr.children[0], env)?;
                let right = get_expr_return_type(&expr.children[1], env)?;

                match (left, right){
                    (Type::Int, Type::Int) => Some(Type::Bool),
                    (Type::Float, Type:: Float) => Some(Type::Bool),
                    /*(Type::Int, Type::Float) => Some(Type::Bool),
                    (Type::Float, Type::Int) => Some(Type::Bool),*/

                    _ => {
                        report(&format!("Operator '<=' is not defined for types '{:?}' and '{:?}'", left, right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::Not => {
                let right = get_expr_return_type(&expr.children[0], env)?;

                match right{
                    Type::Bool => Some(Type::Bool),

                    _ => {
                        report(&format!("Operator '!' is not defined for type '{:?}'", right), expr.kind.location.clone());
                        None
                    }
                }
            },

            TokenType::Dot => {
                if expr.children[0].children.len() != 0{
                    report("You cannot chain this", expr.kind.location.clone());
                    return None;
                }

                if expr.children[0].kind.kind != TokenType::Ident{
                    report("Expected an identifier", expr.children[0].kind.location.clone());
                    return None;
                }

                if !env.has_import(&expr.children[0].kind.literal){
                    report(&format!("Script {}.pprs was not found", expr.children[0].kind.literal), expr.kind.location.clone());
                    return None;
                }

                if expr.children[1].children.len() != 1{
                    report("Expected function call", expr.kind.location.clone());
                    return None;
                }

                if !verify_func_call(&expr.children[1], true, env){ return None; }

                let path = env.imports().get(&expr.children[0].kind.literal).unwrap();
                let name = expr.children[1].kind.literal.clone();
                let mut params = vec![];
                for arg in &expr.children[1].children[0].children{
                    params.push(get_expr_return_type(&arg, env).unwrap());
                }

                let func_sign = FuncSign{ name, params };
                if !env.has_public_func(&path, &func_sign){
                    report(&format!("The function {func_sign} was not found"), expr.kind.location.clone());
                    return None;
                }

                env.public_functions.iter().filter(|(p, func, _)| (p == path) && (func == &func_sign)).map(|e| e.2).next()
            },
            _ => panic!("Unexpected operator")
        }
    }
}
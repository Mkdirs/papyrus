use std::collections::HashMap;

use neoglot_lib::{parser, lexer::Token};

use crate::TokenType;

type AST = parser::AST<Token<TokenType>>;


#[derive(Debug, PartialEq, Eq)]
pub enum Param{
    Value(u32),
    Register(String)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction{
    Copy(Param, String),
    Add(Param, Param, String),
    Sub(Param, Param, String),
    Mul(Param, Param, String),
    Div(Param, Param, String),
    GT(Param, Param, String),
    LT(Param, Param, String),
    Eq(Param, Param, String),
    GE(Param, Param, String),
    LE(Param, Param, String),
    NE(Param, Param, String),
    And(Param, Param, String),
    Or(Param, Param, String),
    Not(Param, String),
    Label(String)
}

#[derive(Debug, Clone)]
struct Context{
    pub num_vars : u32,
    pub renamed_user_vars : HashMap<String, String>
}

impl Context{
    pub fn add_user_var(&mut self, name:String){
        let new_name = self.get_unique_var_name();
        self.renamed_user_vars.insert(name, new_name);
        self.num_vars += 1;
    }


    pub fn rename_user_var(&mut self, name:String, new:String){
        self.renamed_user_vars.insert(name, new);
        self.num_vars += 1;
    }

    pub fn get_renamed_user_var(&self, name: &String) -> &String{
        &self.renamed_user_vars[name]
    }

    pub fn get_unique_var_name(&self) -> String{
        format!("r{}", self.num_vars)
    }
}

pub fn parse(forest: Vec<AST>) -> Vec<Instruction>{
    let mut instructions = vec![Instruction::Label("main".to_string())];

    let mut ctx = Context { num_vars: 0, renamed_user_vars: HashMap::new() };

    for tree in forest{
        if tree.kind.kind == TokenType::Colon{
            add_var_in_context(tree, &mut ctx);
        
        }else if tree.kind.kind == TokenType::Eq{

            instructions.append(&mut parse_assign(tree, &mut ctx));
        }
    }

    instructions
}

fn add_var_in_context(binding_tree:AST, ctx: &mut Context){
    let name = binding_tree.children[0].kind.literal.clone();
    ctx.add_user_var(name);
}

fn to_param(token: &Token<TokenType>, ctx: &Context) -> Param{
    if token.kind == TokenType::Ident{
        Param::Register(ctx.get_renamed_user_var(&token.literal).clone())
    }else if token.kind == TokenType::Int{
        Param::Value(token.literal.parse::<i32>().expect("Unable to parse to int") as u32)
    
    }else if token.kind == TokenType::Float{
        Param::Value(token.literal.parse::<f32>().expect("Unable to parse to float").to_bits())
    
    }else if token.kind == TokenType::Bool{
        if &token.literal == "true"{
            Param::Value(1)
        }else if &token.literal == "false"{
            Param::Value(0)
        }else{ panic!("Should not be there") }

    }else if token.kind == TokenType::Hex{
        todo!("Parse hex into u32")
    }else{
        panic!("Should not be there")
    }
}
fn expand_binary_expr(expr: &AST, ctx: &mut Context) -> Vec<Instruction>{
    let left = &expr.children[0];
    let right = &expr.children[1];
    let mut instructions = vec![];

    let args = if left.children.is_empty() && right.children.is_empty(){
        let p1 = to_param(&left.kind, ctx);
        let p2 = to_param(&right.kind, ctx);
        let last_reg = ctx.get_unique_var_name();
       
        (p1, p2, last_reg)
    
    }else if !left.children.is_empty() && right.children.is_empty(){
        instructions.append(&mut expand_expr(left, ctx));
        let last_reg = ctx.get_unique_var_name();

        let p1 = Param::Register(last_reg);
        let p2 = to_param(&right.kind, ctx);
        ctx.num_vars += 1;
        let reg = ctx.get_unique_var_name();

        (p1, p2, reg)
    }else if left.children.is_empty() && !right.children.is_empty(){
        instructions.append(&mut expand_expr(right, ctx));

        let last_reg = ctx.get_unique_var_name();

        let p1 = to_param(&left.kind, ctx);
        let p2 = Param::Register(last_reg);
        
        ctx.num_vars += 1;
        let reg = ctx.get_unique_var_name();

        (p1, p2, reg)
    }else{
        instructions.append(&mut expand_expr(left, ctx));
        let p1 = Param::Register(ctx.get_unique_var_name());
        ctx.num_vars += 1;

        instructions.append(&mut expand_expr(right, ctx));
        let p2 = Param::Register(ctx.get_unique_var_name());
        ctx.num_vars += 1;
        
        let reg = ctx.get_unique_var_name();

        (p1, p2, reg)
    };

    if expr.kind.kind == TokenType::Plus{
        instructions.push(Instruction::Add(args.0, args.1, args.2));

    }else if expr.kind.kind == TokenType::Minus{
        instructions.push(Instruction::Sub(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::Mul{
        instructions.push(Instruction::Mul(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::Div{
        instructions.push(Instruction::Div(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::GT{
        instructions.push(Instruction::GT(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::LT{
        instructions.push(Instruction::LT(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::And{
        instructions.push(Instruction::And(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::Or{
        instructions.push(Instruction::Or(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::DoubleEq{
        instructions.push(Instruction::Eq(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::GTEq{
        instructions.push(Instruction::GE(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::LTEq{
        instructions.push(Instruction::LE(args.0, args.1, args.2));
   
    }else if expr.kind.kind == TokenType::NotEq{
        instructions.push(Instruction::NE(args.0, args.1, args.2));
    
    }else{ panic!("This is not a binary operation"); }

    instructions
}
fn expand_expr(expr:&AST, ctx: &mut Context) -> Vec<Instruction>{

    if expr.kind.kind == TokenType::Plus
    || expr.kind.kind == TokenType::Minus
    || expr.kind.kind == TokenType::Mul
    || expr.kind.kind == TokenType::Div
    || expr.kind.kind == TokenType::GT
    || expr.kind.kind == TokenType::LT
    || expr.kind.kind == TokenType::And
    || expr.kind.kind == TokenType::Or
    || expr.kind.kind == TokenType::DoubleEq
    || expr.kind.kind == TokenType::GTEq
    || expr.kind.kind == TokenType::LTEq
    || expr.kind.kind == TokenType::NotEq{
        expand_binary_expr(expr, ctx)
    }else if expr.kind.kind == TokenType::Not{
        let right = &expr.children[0];
        if right.children.is_empty(){
            let p = to_param(&right.kind, ctx);
            let last_reg = ctx.get_unique_var_name();
       
            
            return vec![Instruction::Not(p, last_reg)];
        }else{
            let mut instructions = expand_expr(right, ctx);
            let p = Param::Register(ctx.get_unique_var_name());
            ctx.num_vars += 1;

            let last_reg = ctx.get_unique_var_name();

            instructions.push(Instruction::Not(p, last_reg));

            return instructions;
        }
    }else{
        panic!("Should not be there")
    }
}

fn parse_assign(assign_tree:AST, ctx: &mut Context) -> Vec<Instruction>{


    let name = if assign_tree.children[0].kind.kind == TokenType::Colon{
        assign_tree.children[0].children[0].kind.literal.clone()
    }else{
        assign_tree.children[0].kind.literal.clone()
    };

    let expr = &assign_tree.children[1];
    let mut instructions = vec![];

    if !expr.children.is_empty(){
        instructions.append(&mut expand_expr(expr, ctx));
        let last_reg = ctx.get_unique_var_name();
        ctx.rename_user_var(name, last_reg);
        
    }else{
        let normalized_name = if !ctx.renamed_user_vars.contains_key(&name){
            ctx.add_user_var(name.clone());
            ctx.get_renamed_user_var(&name).clone()
        }else{
            ctx.renamed_user_vars[&name].clone()
        };

        let param = to_param(&expr.kind, ctx);
        instructions.push(Instruction::Copy(param, normalized_name));
    }

    instructions
}
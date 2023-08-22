use std::collections::HashMap;

use neoglot_lib::{parser, lexer::Token};

use crate::TokenType;

type AST = parser::AST<Token<TokenType>>;


#[derive(Debug, PartialEq, Eq, Clone)]
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
    
    Push(Param, Param),
    Merge(Param, Param),
    Put(Param, Param, Param),
    Fill(Param),
    Pop,
    Save,

    JT(Param, String),
    JF(Param, String),
    
    Label(String),

    Jump(String),

    Call(String, Vec<Param>),
    Ret
}

#[derive(Debug, Clone, Default)]
pub struct Context{
    registers : Vec<String>,
    labels: Vec<String>,
    pub num_while_labels : u32
}

impl Context{
    pub fn add_register(&mut self, name:String){
        self.registers.push(name);
    }

    pub fn create_temp_register(&mut self) -> String{
        self.add_register(format!("r{}", self.registers.len()));
        self.registers.last().unwrap().clone()
    }

    pub fn create_temp_label(&mut self, tag: &str) -> String{
        let n = self.labels.iter().filter(|e| e.starts_with(tag)).count();
        self.labels.push(format!("{tag}{n}"));
        self.labels.last().unwrap().clone()
    }


    /*pub fn get_unique_while_label(&self) -> String{
        format!("while{}", self.num_while_labels)
    }

    pub fn get_unique_while_label_end(&self) -> String{
        format!("end_while{}", self.num_while_labels)
    }*/
}

pub fn parse(forest: &Vec<AST>, ctx: &mut Context) -> Vec<Instruction>{
    let mut instructions = vec![];


    for tree in forest{
        if tree.kind.kind == TokenType::Colon{
            add_var_in_context(&tree, ctx);
        
        }else if tree.kind.kind == TokenType::Eq{

            instructions.append(&mut parse_assign(tree, ctx));
        }else if tree.kind.kind == TokenType::Def{
            instructions.append(&mut parse_def(tree));
        
        }else if tree.kind.kind == TokenType::Return{
            instructions.append(&mut parse_return(tree, ctx));
        
        }else if tree.kind.kind == TokenType::Ident{
            instructions.append(&mut parse_func_call(tree, ctx));
        
        }else if tree.kind.kind == TokenType::While{
            instructions.append(&mut parse_while(tree, ctx));
        
        }else if tree.kind.kind == TokenType::If{
            let root_scope = ctx.create_temp_label("root_scope");
            instructions.append(&mut parse_if(tree, ctx, true, root_scope.clone()));
            instructions.push(Instruction::Label(root_scope));
        }else if tree.kind.kind == TokenType::Subcanvas{
            instructions.append(&mut parse_subcanvas(tree, ctx));
        }
    }

    instructions
}

fn add_var_in_context(binding_tree: &AST, ctx: &mut Context){
    let name = binding_tree.children[0].kind.literal.clone();
    ctx.add_register(name);
}


fn to_param(token: &Token<TokenType>, ctx: &Context) -> Param{
    if token.kind == TokenType::Ident{
        Param::Register(token.literal.clone())
        //Param::Register(ctx.get_renamed_user_var(&token.literal).clone())
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
        let lit = token.literal.clone();
        Param::Value(u32::from_str_radix(&lit[1..], 16).expect("Unable to parse to u32"))
    }else{
        panic!("Should not be there")
    }
}


fn expand_binary_expr(expr: &AST, ctx: &mut Context, return_reg:String) -> Vec<Instruction>{
    let left = &expr.children[0];
    let right = &expr.children[1];
    let mut instructions = vec![];

    let args = if left.children.is_empty() && right.children.is_empty(){
        let p1 = to_param(&left.kind, ctx);
        let p2 = to_param(&right.kind, ctx);
        
        //ctx.num_vars += 1;
        //let last_reg = ctx.get_unique_var_name();
       
        (p1, p2, return_reg)
    
    }else if !left.children.is_empty() && right.children.is_empty(){
        let reg = ctx.create_temp_register();
        instructions.append(&mut expand_expr(left, ctx, reg.clone()));
        //let last_reg = ctx.get_unique_var_name();

        let p1 = Param::Register(reg);
        let p2 = to_param(&right.kind, ctx);
        //ctx.num_vars += 1;
        //let reg = ctx.get_unique_var_name();

        (p1, p2, return_reg)
    }else if left.children.is_empty() && !right.children.is_empty(){
        let reg = ctx.create_temp_register();
        instructions.append(&mut expand_expr(right, ctx, reg.clone()));

        //let last_reg = ctx.get_unique_var_name();

        let p1 = to_param(&left.kind, ctx);
        let p2 = Param::Register(reg);
        
        //ctx.num_vars += 1;
        //let reg = ctx.get_unique_var_name();

        (p1, p2, return_reg)
    }else{
        let reg = ctx.create_temp_register();
        instructions.append(&mut expand_expr(left, ctx, reg.clone()));
        let p1 = Param::Register(reg);
        //ctx.num_vars += 1;

        let reg2 = ctx.create_temp_register();
        instructions.append(&mut expand_expr(right, ctx, reg2.clone()));
        let p2 = Param::Register(reg2);
        //ctx.num_vars += 1;
        
        //let reg = ctx.get_unique_var_name();

        (p1, p2, return_reg)
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
fn expand_expr(expr:&AST, ctx: &mut Context, return_reg: String) -> Vec<Instruction>{

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
        expand_binary_expr(expr, ctx, return_reg)
    }else if expr.kind.kind == TokenType::Not{
        let right = &expr.children[0];
        if right.children.is_empty(){
            let p = to_param(&right.kind, ctx);
            //let last_reg = ctx.get_unique_var_name();
       
            
            return vec![Instruction::Not(p, return_reg)];
        }else{
            let reg = ctx.create_temp_register();
            let mut instructions = expand_expr(right, ctx, reg.clone());
            let p = Param::Register(reg);
            //ctx.num_vars += 1;

            //let last_reg = ctx.get_unique_var_name();

            instructions.push(Instruction::Not(p, return_reg));

            return instructions;
        }

    }else if expr.kind.kind == TokenType::Ident{
        let mut instructions = parse_func_call(expr, ctx);
        //let reg = ctx.get_unique_var_name();
        //ctx.num_vars += 1;

        instructions.push(Instruction::Copy(Param::Register("rt".to_string()), return_reg));

        return instructions;

    }else{
        panic!("Should not be there")
    }
}

fn parse_assign(assign_tree: &AST, ctx: &mut Context) -> Vec<Instruction>{


    let name = if assign_tree.children[0].kind.kind == TokenType::Colon{
        assign_tree.children[0].children[0].kind.literal.clone()
    }else{
        assign_tree.children[0].kind.literal.clone()
    };

    let expr = &assign_tree.children[1];
    let mut instructions = vec![];

    if !ctx.registers.contains(&name){
        ctx.add_register(name.clone());
    }
    
    /*let normalized_name = if !ctx.renamed_user_vars.contains_key(&name){
        ctx.add_user_var(name.clone());
        ctx.get_renamed_user_var(&name).clone()
    }else{
        ctx.renamed_user_vars[&name].clone()
    };*/

    if !expr.children.is_empty(){
        instructions.append(&mut expand_expr(expr, ctx, name.clone()));

        /*if expr.kind.kind == TokenType::Ident{
            instructions.push(Instruction::Copy(Param::Register("rt".to_string()), normalized_name));
        }else{
            //ctx.rename_user_var(name, ctx.get_unique_var_name());
            //let last_reg = ctx.get_unique_var_name();

            //instructions.push(Instruction::Copy(Param::Register(last_reg), normalized_name));
        }*/

        //let last_reg = ctx.get_unique_var_name();
        
        //ctx.rename_user_var(name, last_reg);
        //ctx.num_vars += 1;
        /*if expr.kind.kind != TokenType::Ident{
            instructions.append(&mut expand_expr(expr, ctx));
            let last_reg = ctx.get_unique_var_name();
            ctx.rename_user_var(name, last_reg);
            //ctx.num_vars += 1;
        }else{
            instructions.append(&mut parse_func_call(expr, ctx));
            let last_reg = ctx.get_unique_var_name();
            
            instructions.push(Instruction::Copy(Param::Register("rt".to_string()), last_reg));
        }*/
        
    }else{
        /*let normalized_name = if !ctx.renamed_user_vars.contains_key(&name){
            ctx.add_user_var(name.clone());
            ctx.get_renamed_user_var(&name).clone()
        }else{
            ctx.renamed_user_vars[&name].clone()
        };*/

        let param = to_param(&expr.kind, ctx);
        instructions.push(Instruction::Copy(param, name));
    }

    instructions
}

fn parse_def(def_tree: &AST) -> Vec<Instruction>{
    let mut instructions = vec![];
    let func_tree = &def_tree.children[0];
    let block = &def_tree.children[def_tree.children.len()-1];

    let mut ctx = Context::default();

    let name = func_tree.kind.literal.clone();
    instructions.push(Instruction::Label(name));

    for (i, param) in func_tree.children.iter().enumerate(){
        add_var_in_context(param, &mut ctx);
        //rename_var(param, format!("p{i}"), &mut ctx);
    }

    instructions.append(&mut parse(&block.children, &mut ctx));
    instructions.push(Instruction::Ret);

    instructions
}

fn parse_return(return_tree: &AST, ctx: &mut Context) -> Vec<Instruction>{
    if return_tree.children.is_empty(){ return vec![]; }

    let expr = &return_tree.children[0];

    if !expr.children.is_empty(){
        expand_expr(expr, ctx, "rt".to_string())
    }else{
        let p = to_param(&expr.kind, ctx);

        vec![Instruction::Copy(p, "rt".to_string())]
    }
}
fn parse_func_call(func_call_tree: &AST, ctx: &mut Context) -> Vec<Instruction>{
    let name = func_call_tree.kind.literal.clone();
    let mut instructions = vec![];
    let mut params = vec![];


    for arg in &func_call_tree.children[0].children{
        if !arg.children.is_empty(){
            let reg = ctx.create_temp_register();
            instructions.append(&mut expand_expr(arg, ctx, reg.clone()));
            //let last_reg = ctx.get_unique_var_name();
            params.push(Param::Register(reg));
        }else{
            params.push(to_param(&arg.kind, ctx));
        }
    }

    if &name == "create_canvas"{
        instructions.push(Instruction::Push(params[0].clone(), params[1].clone()));
    }else if &name == "put"{
        instructions.push(Instruction::Put(params[0].clone(), params[1].clone(), params[2].clone()));

    }else if &name == "fill"{
        instructions.push(Instruction::Fill(params[0].clone()));

    }else if &name == "save_canvas"{
        instructions.push(Instruction::Save);
        instructions.push(Instruction::Pop);
    }else{
        instructions.push(Instruction::Call(name, params));
    }


    instructions
}

fn parse_while(while_tree: &AST, ctx: &mut Context) -> Vec<Instruction>{
    let expr = &while_tree.children[0];
    let block = &while_tree.children[1];


    let while_start = ctx.create_temp_label("while");//ctx.get_unique_while_label();
    let mut instructions = vec![Instruction::Label(while_start.clone())];
    let end_label = format!("end_{}", while_start);
    ctx.num_while_labels += 1;


    let param = if expr.children.is_empty(){
        to_param(&expr.kind, ctx)
    }else{
        let reg = ctx.create_temp_register();
        instructions.append(&mut expand_expr(expr, ctx, reg.clone()));
        //let last_reg = ctx.get_unique_var_name();
        Param::Register(reg)
    };
    instructions.push(Instruction::JF(param, end_label.clone()));

    instructions.append(&mut parse(&block.children, ctx));
    
    instructions.push(Instruction::Jump(while_start));
    instructions.push(Instruction::Label(end_label));

    instructions
}

fn parse_if(if_tree: &AST, ctx: &mut Context, root:bool, root_scope_label:String) -> Vec<Instruction>{
    let expr = &if_tree.children[0];
    let block = &if_tree.children[1];
    let mut instructions = vec![];

    let param = if expr.children.is_empty(){
        to_param(&expr.kind, ctx)
    }else{
        let reg = ctx.create_temp_register();
        instructions.append(&mut expand_expr(expr, ctx, reg.clone()));
        Param::Register(reg)
    };

    let start = if root {
        ctx.create_temp_label("if")
    }else {
        ctx.create_temp_label("elif")
    };
    
    if if_tree.children.len() != 3{
        instructions.push(Instruction::JF(param, root_scope_label));
        instructions.append(&mut parse(&block.children, ctx));
    }else{
        let else_tree = &if_tree.children[2];
        
        if else_tree.children[0].kind.kind == TokenType::If{
            let n = ctx.labels.iter().filter(|e| e.starts_with("elif")).count();

            let else_if_start = format!("elif{}", n);
            instructions.push(Instruction::JF(param, else_if_start.clone()));
            instructions.append(&mut parse(&block.children, ctx));
            instructions.push(Instruction::Jump(root_scope_label.clone()));
            
            instructions.push(Instruction::Label(else_if_start));
            instructions.append(&mut parse_if(&else_tree.children[0], ctx, false, root_scope_label));
        }else{
            let else_labl = ctx.create_temp_label("else");
            instructions.push(Instruction::JF(param, else_labl.clone()));
            instructions.append(&mut parse(&block.children, ctx));
            instructions.push(Instruction::Jump(root_scope_label));
           
            instructions.push(Instruction::Label(else_labl));
            instructions.append(&mut parse(&else_tree.children[0].children, ctx));

        }
    };


    instructions
}

fn parse_subcanvas(subcanvas_tree: &AST, ctx: &mut Context) -> Vec<Instruction>{
    let mut instructions = vec![];

    let ofst_x = &subcanvas_tree.children[0];
    let ofst_y = &subcanvas_tree.children[1];

    let width = &subcanvas_tree.children[2];
    let height = &subcanvas_tree.children[3];

    let block = &subcanvas_tree.children[4];

    let x_param = if ofst_x.children.is_empty(){
        to_param(&ofst_x.kind, ctx)
    }else{
        let reg = ctx.create_temp_register();
        instructions.append(&mut expand_expr(ofst_x, ctx, reg.clone()));
        Param::Register(reg)
    };

    let y_param = if ofst_y.children.is_empty(){
        to_param(&ofst_y.kind, ctx)
    }else{
        let reg = ctx.create_temp_register();
        instructions.append(&mut expand_expr(ofst_y, ctx, reg.clone()));
        Param::Register(reg)
    };

    let width_param = if width.children.is_empty(){
        to_param(&width.kind, ctx)
    }else{
        let reg = ctx.create_temp_register();
        instructions.append(&mut expand_expr(width, ctx, reg.clone()));
        Param::Register(reg)
    };

    let height_param = if height.children.is_empty(){
        to_param(&height.kind, ctx)
    }else{
        let reg = ctx.create_temp_register();
        instructions.append(&mut expand_expr(height, ctx, reg.clone()));
        Param::Register(reg)
    };

    instructions.push(Instruction::Push(width_param, height_param));
    instructions.append(&mut parse(&block.children, ctx));
    instructions.push(Instruction::Merge(x_param, y_param));

    instructions
}
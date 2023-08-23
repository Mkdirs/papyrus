use std::collections::HashMap;

use neoglot_lib::{parser, lexer::Token};

use crate::{TokenType, environment::Type, validator::get_type};

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

    Addf(Param, Param, String),
    Subf(Param, Param, String),
    Mulf(Param, Param, String),
    Divf(Param, Param, String),
    
    GT(Param, Param, String),
    LT(Param, Param, String),
    Eq(Param, Param, String),
    GE(Param, Param, String),
    LE(Param, Param, String),
    NE(Param, Param, String),
    And(Param, Param, String),
    Or(Param, Param, String),
    Not(Param, String),

    Flt(Param, String),
    Int(Param, String),
    
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

#[derive(Debug, Clone)]
pub struct Context{
    registers : Vec<String>,
    labels: Vec<String>,
    bindings: HashMap<String, Type>,
    func_returns: HashMap<String, Type>,
    pub num_while_labels : u32
}

impl Default for Context{
    fn default() -> Self {
        Context { 
            registers: Vec::default(),
            labels: Vec::default(),
            bindings: HashMap::default(),
            func_returns: HashMap::from_iter([
                ("float".to_string(), Type::Float),
                ("int".to_string(), Type::Int)
            ]),
            num_while_labels: u32::default()
        }
    }
}

impl Context{
    pub fn add_register(&mut self, name:String, _type: Option<Type>){
        self.registers.push(name.clone());
        if let Some(t) = _type{
            self.bindings.insert(name, t);
        }
    }

    pub fn create_temp_register(&mut self, _type:Option<Type>) -> String{
        self.add_register(format!("r{}", self.registers.len()), _type);
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
            instructions.append(&mut parse_def(tree, ctx));
        
        }else if tree.kind.kind == TokenType::Return{
            instructions.append(&mut parse_return(tree, ctx));
        
        }else if tree.kind.kind == TokenType::Ident{
            instructions.append(&mut parse_func_call(tree, ctx));
        
        }else if tree.kind.kind == TokenType::While{
            instructions.append(&mut parse_while(tree, ctx));
        
        }else if tree.kind.kind == TokenType::If{
            let root_scope = ctx.create_temp_label("root_scope");
            instructions.append(&mut parse_if(tree, ctx, root_scope.clone()));
            instructions.push(Instruction::Label(root_scope));
        }else if tree.kind.kind == TokenType::Subcanvas{
            instructions.append(&mut parse_subcanvas(tree, ctx));
        }
    }

    instructions
}

fn add_var_in_context(binding_tree: &AST, ctx: &mut Context){
    let name = binding_tree.children[0].kind.literal.clone();
    let t = get_type(binding_tree.children[1].kind.literal.clone()).unwrap();
    ctx.add_register(name, Some(t));
}


fn to_param(token: &Token<TokenType>, ctx: &Context) -> (Param, Type){
    if token.kind == TokenType::Ident{
        (Param::Register(token.literal.clone()), ctx.bindings.get(&token.literal).unwrap().clone() )
        //Param::Register(ctx.get_renamed_user_var(&token.literal).clone())
    }else if token.kind == TokenType::Int{
        (
            Param::Value(token.literal.parse::<i32>().expect("Unable to parse to int") as u32),
            Type::Int
        )
    
    }else if token.kind == TokenType::Float{
        (
            Param::Value(token.literal.parse::<f32>().expect("Unable to parse to float").to_bits()),
            Type::Float
        )
    
    }else if token.kind == TokenType::Bool{
        if &token.literal == "true"{
            (Param::Value(1), Type::Bool)
        }else if &token.literal == "false"{
            (Param::Value(0), Type::Bool)
        }else{ panic!("Should not be there") }

    }else if token.kind == TokenType::Hex{
        let lit = token.literal.clone();
        (
            Param::Value(u32::from_str_radix(&lit[1..], 16).expect("Unable to parse to u32")),
            Type::Color
        )
    }else{
        panic!("Should not be there")
    }
}


fn expand_binary_expr(expr: &AST, ctx: &mut Context, return_reg:String) -> (Vec<Instruction>, Type){
    let left = &expr.children[0];
    let right = &expr.children[1];
    let mut instructions = vec![];
    let left_type:Type;
    let right_type:Type;

    let args = if left.children.is_empty() && right.children.is_empty(){
        let p1:Param;
        let p2:Param;
        (p1, left_type) = to_param(&left.kind, ctx);
        (p2, right_type) = to_param(&right.kind, ctx);
        
        //ctx.num_vars += 1;
        //let last_reg = ctx.get_unique_var_name();
       
        (p1, p2, return_reg)
    
    }else if !left.children.is_empty() && right.children.is_empty(){
        let reg = ctx.create_temp_register(None);
        let mut instr:Vec<Instruction>;
        
        (instr, left_type) = expand_expr(left, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), left_type);
        
        instructions.append(&mut instr);
        //let last_reg = ctx.get_unique_var_name();

        let p1 = Param::Register(reg);
        let p2:Param;
        (p2, right_type) = to_param(&right.kind, ctx);
        //ctx.num_vars += 1;
        //let reg = ctx.get_unique_var_name();

        (p1, p2, return_reg)
    }else if left.children.is_empty() && !right.children.is_empty(){
        let reg = ctx.create_temp_register(None);
        let mut instr:Vec<Instruction>;
        (instr, right_type) = expand_expr(right, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), right_type);
        instructions.append(&mut instr);

        //let last_reg = ctx.get_unique_var_name();

        let p1:Param;
        (p1, left_type) = to_param(&left.kind, ctx);
        let p2 = Param::Register(reg);
        
        //ctx.num_vars += 1;
        //let reg = ctx.get_unique_var_name();

        (p1, p2, return_reg)
    }else{
        let reg = ctx.create_temp_register(None);
        let mut instr:Vec<Instruction>;
        
        (instr, left_type) = expand_expr(left, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), left_type);
        instructions.append(&mut instr);
        let p1 = Param::Register(reg);
        //ctx.num_vars += 1;

        let reg2 = ctx.create_temp_register(None);
        (instr, right_type) = expand_expr(right, ctx, reg2.clone());
        ctx.bindings.insert(reg2.clone(), right_type);
        instructions.append(&mut instr);
        let p2 = Param::Register(reg2);
        //ctx.num_vars += 1;
        
        //let reg = ctx.get_unique_var_name();

        (p1, p2, return_reg)
    };

    let _type = if expr.kind.kind == TokenType::Plus{
        if left_type == Type::Float && right_type == Type::Float{
            instructions.push(Instruction::Addf(args.0, args.1, args.2));
            Type::Float
        
        }else if left_type == Type::Int && right_type ==Type::Int{
            instructions.push(Instruction::Add(args.0, args.1, args.2));
            Type::Int
        
        }else if left_type == Type::Int && right_type == Type::Float{
            let reg = String::from("rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.0, reg.clone()));
            instructions.push(Instruction::Addf(Param::Register(reg), args.1, args.2));
            Type::Float
        
        }else{
            let reg = String::from("rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.1, reg.clone()));
            instructions.push(Instruction::Addf(args.0, Param::Register(reg), args.2));
            Type::Float
        }
        
        

    }else if expr.kind.kind == TokenType::Minus{
        if left_type == Type::Float && right_type == Type::Float{
            instructions.push(Instruction::Subf(args.0, args.1, args.2));
            Type::Float
        
        }else if left_type == Type::Int && right_type ==Type::Int{
            instructions.push(Instruction::Sub(args.0, args.1, args.2));
            Type::Int
        
        }else if left_type == Type::Int && right_type == Type::Float{
            let reg = String::from("rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.0, reg.clone()));
            instructions.push(Instruction::Subf(Param::Register(reg), args.1, args.2));
            Type::Float
        
        }else{
            let reg = String::from("rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.1, reg.clone()));
            instructions.push(Instruction::Subf(args.0, Param::Register(reg), args.2));
            Type::Float
        }
        //instructions.push(Instruction::Sub(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::Mul{
        if left_type == Type::Float && right_type == Type::Float{
            instructions.push(Instruction::Mulf(args.0, args.1, args.2));
            Type::Float
        
        }else if left_type == Type::Int && right_type ==Type::Int{
            instructions.push(Instruction::Mul(args.0, args.1, args.2));
            Type::Int
        
        }else if left_type == Type::Int && right_type == Type::Float{
            let reg = String::from("rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.0, reg.clone()));
            instructions.push(Instruction::Mulf(Param::Register(reg), args.1, args.2));
            Type::Float
        
        }else{
            let reg = String::from("rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.1, reg.clone()));
            instructions.push(Instruction::Mulf(args.0, Param::Register(reg), args.2));
            Type::Float
        }
        //instructions.push(Instruction::Mul(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::Div{
        if left_type == Type::Float && right_type == Type::Float{
            instructions.push(Instruction::Divf(args.0, args.1, args.2));
            Type::Float
        
        }else if left_type == Type::Int && right_type ==Type::Int{
            instructions.push(Instruction::Div(args.0, args.1, args.2));
            Type::Int
        
        }else if left_type == Type::Int && right_type == Type::Float{
            let reg = String::from("rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.0, reg.clone()));
            instructions.push(Instruction::Divf(Param::Register(reg), args.1, args.2));
            Type::Float
        
        }else{
            let reg = String::from("rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.1, reg.clone()));
            instructions.push(Instruction::Divf(args.0, Param::Register(reg), args.2));
            Type::Float
        }
        //instructions.push(Instruction::Div(args.0, args.1, args.2));
    
    }else if expr.kind.kind == TokenType::GT{
        instructions.push(Instruction::GT(args.0, args.1, args.2));
        Type::Bool
    
    }else if expr.kind.kind == TokenType::LT{
        instructions.push(Instruction::LT(args.0, args.1, args.2));
        Type::Bool
    
    }else if expr.kind.kind == TokenType::And{
        instructions.push(Instruction::And(args.0, args.1, args.2));
        Type::Bool
    
    }else if expr.kind.kind == TokenType::Or{
        instructions.push(Instruction::Or(args.0, args.1, args.2));
        Type::Bool
    
    }else if expr.kind.kind == TokenType::DoubleEq{
        instructions.push(Instruction::Eq(args.0, args.1, args.2));
        Type::Bool
    
    }else if expr.kind.kind == TokenType::GTEq{
        instructions.push(Instruction::GE(args.0, args.1, args.2));
        Type::Bool
    
    }else if expr.kind.kind == TokenType::LTEq{
        instructions.push(Instruction::LE(args.0, args.1, args.2));
        Type::Bool
   
    }else if expr.kind.kind == TokenType::NotEq{
        instructions.push(Instruction::NE(args.0, args.1, args.2));
        Type::Bool
    
    }else{ panic!("This is not a binary operation"); };

    (instructions, _type)
}
fn expand_expr(expr:&AST, ctx: &mut Context, return_reg: String) -> (Vec<Instruction>, Type){

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
            let (p, _) = to_param(&right.kind, ctx);
            //let last_reg = ctx.get_unique_var_name();
       
            
            return (vec![Instruction::Not(p, return_reg)], Type::Bool);
        }else{
            let reg = ctx.create_temp_register(Some(Type::Bool));
            let (mut instructions, _) = expand_expr(right, ctx, reg.clone());
            let p = Param::Register(reg);
            //ctx.num_vars += 1;

            //let last_reg = ctx.get_unique_var_name();

            instructions.push(Instruction::Not(p, return_reg));

            return (instructions, Type::Bool);
        }

    }else if expr.kind.kind == TokenType::Ident{
        let mut instructions = parse_func_call(expr, ctx);
        //let reg = ctx.get_unique_var_name();
        //ctx.num_vars += 1;

        instructions.push(Instruction::Copy(Param::Register("rt".to_string()), return_reg));


        return (instructions, ctx.func_returns.get(&expr.kind.literal).unwrap_or(&Type::Void).clone());

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
        ctx.add_register(name.clone(), None);
    }
    
    /*let normalized_name = if !ctx.renamed_user_vars.contains_key(&name){
        ctx.add_user_var(name.clone());
        ctx.get_renamed_user_var(&name).clone()
    }else{
        ctx.renamed_user_vars[&name].clone()
    };*/

    if !expr.children.is_empty(){
        let (mut instr, t) = expand_expr(expr, ctx, name.clone());
        ctx.bindings.insert(name, t);
        instructions.append(&mut instr);

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

        let (param, t) = to_param(&expr.kind, ctx);
        ctx.bindings.insert(name.clone(), t);
        instructions.push(Instruction::Copy(param, name));
    }

    instructions
}

fn parse_def(def_tree: &AST, parent:&mut Context) -> Vec<Instruction>{
    let mut instructions = vec![];
    let func_tree = &def_tree.children[0];
    let block = &def_tree.children[def_tree.children.len()-1];

    if def_tree.children.len() == 3{
        parent.func_returns.insert(func_tree.kind.literal.clone(), get_type(def_tree.children[1].kind.literal.clone()).unwrap());
    }else{
        parent.func_returns.insert(func_tree.kind.literal.clone(), Type::Void);
    }

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
        let (instr, t) = expand_expr(expr, ctx, "rt".to_string());
        ctx.bindings.insert("rt".to_string(), t);
        
        instr
    }else{
        let (p, t) = to_param(&expr.kind, ctx);
        ctx.bindings.insert("rt".to_string(), t);

        vec![Instruction::Copy(p, "rt".to_string())]
    }
}
fn parse_func_call(func_call_tree: &AST, ctx: &mut Context) -> Vec<Instruction>{
    let name = func_call_tree.kind.literal.clone();
    let mut instructions = vec![];
    let mut params = vec![];


    for arg in &func_call_tree.children[0].children{
        if !arg.children.is_empty(){
            let reg = ctx.create_temp_register(None);
            let (mut instr, t) = expand_expr(arg, ctx, reg.clone());
            ctx.bindings.insert(reg.clone(), t);

            instructions.append(&mut instr);
            //let last_reg = ctx.get_unique_var_name();
            params.push(Param::Register(reg));
        }else{
            let (p, _) = to_param(&arg.kind, ctx);
            params.push(p);
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
    }else if &name == "float"{
        let reg = String::from("rt");
        ctx.bindings.insert(reg.clone(), Type::Float);

        instructions.push(Instruction::Flt(params[0].clone(), reg));
    
    }else if &name == "int"{
        let reg = String::from("rt");
        ctx.bindings.insert(reg.clone(), Type::Float);

        instructions.push(Instruction::Int(params[0].clone(), reg));

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
        let (p, _) = to_param(&expr.kind, ctx);
        p
    }else{
        let reg = ctx.create_temp_register(None);
        let (mut instr, t) = expand_expr(expr, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), t);

        instructions.append(&mut instr);
        //let last_reg = ctx.get_unique_var_name();
        Param::Register(reg)
    };
    instructions.push(Instruction::JF(param, end_label.clone()));

    instructions.append(&mut parse(&block.children, ctx));
    
    instructions.push(Instruction::Jump(while_start));
    instructions.push(Instruction::Label(end_label));

    instructions
}

fn parse_if(if_tree: &AST, ctx: &mut Context, root_scope_label:String) -> Vec<Instruction>{
    let expr = &if_tree.children[0];
    let block = &if_tree.children[1];
    let mut instructions = vec![];

    let param = if expr.children.is_empty(){
        let (p, _) = to_param(&expr.kind, ctx);
        p
    }else{
        let reg = ctx.create_temp_register(None);
        let (mut instr, t) = expand_expr(expr, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), t);


        instructions.append(&mut instr);
        Param::Register(reg)
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
            instructions.append(&mut parse_if(&else_tree.children[0], ctx, root_scope_label));
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
        let (p, _) = to_param(&ofst_x.kind, ctx);
        p
    }else{
        let reg = ctx.create_temp_register(None);
        let (mut instr, t) = expand_expr(ofst_x, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), t);


        instructions.append(&mut instr);
        Param::Register(reg)
    };

    let y_param = if ofst_y.children.is_empty(){
        let (p, _) = to_param(&ofst_y.kind, ctx);
        p
    }else{
        let reg = ctx.create_temp_register(None);
        let (mut instr, t) = expand_expr(ofst_y, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), t);


        instructions.append(&mut instr);
        Param::Register(reg)
    };

    let width_param = if width.children.is_empty(){
        let (p, _) = to_param(&width.kind, ctx);
        p
    }else{
        let reg = ctx.create_temp_register(None);
        let (mut instr, t) = expand_expr(width, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), t);


        instructions.append(&mut instr);
        Param::Register(reg)
    };

    let height_param = if height.children.is_empty(){
        let (p, _) = to_param(&height.kind, ctx);
        p
    }else{
        let reg = ctx.create_temp_register(None);
        let (mut instr, t) = expand_expr(height, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), t);


        instructions.append(&mut instr);
        Param::Register(reg)
    };

    instructions.push(Instruction::Push(width_param, height_param));
    instructions.append(&mut parse(&block.children, ctx));
    instructions.push(Instruction::Merge(x_param, y_param));

    instructions
}
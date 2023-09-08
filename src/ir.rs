use std::{collections::HashMap, path::{PathBuf, Path}};

use neoglot_lib::{parser, lexer::Token};

use crate::{TokenType, environment::{Type, FuncSign}, validator::get_type};

type AST = parser::AST<Token<TokenType>>;



#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Param{
    Value(u32),
    Register(String)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Instruction{
    Import(String, String),
    Copy(Param, String),
    
    Add(Param, Param, String),
    Sub(Param, Param, String),
    Mul(Param, Param, String),
    Div(Param, Param, String),
    Mod(Param, Param, String),
    Pow(Param, Param, String),

    Addf(Param, Param, String),
    Subf(Param, Param, String),
    Mulf(Param, Param, String),
    Divf(Param, Param, String),
    Powf(Param, Param, String),

    Neg(Param, String),
    Negf(Param, String),
    
    GT(Param, Param, String),
    LT(Param, Param, String),

    GTf(Param, Param, String),
    LTf(Param, Param, String),

    Eq(Param, Param, String),

    GE(Param, Param, String),
    LE(Param, Param, String),

    GEf(Param, Param, String),
    LEf(Param, Param, String),
    
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
    Sample(Param, Param, String),
    Width(String),
    Height(String),

    //JT(Param, String),
    JF(Param, String),
    
    Label(String),

    Jump(String),

    Call(String, Vec<Param>),
    Ret,

    Red(Param, String),
    Green(Param, String),
    Blue(Param, String),
    Alpha(Param, String),
    RGBA(Param, Param, Param, Param, String)
}

#[derive(Debug, Clone)]
struct Context{
    registers : Vec<String>,
    labels: Vec<String>,
    bindings: HashMap<String, Type>,
    pub func_returns: HashMap<FuncSign, Type>,
    pub renamed_vars: HashMap<String, String>,
    top_function: String,
    imports: Vec<Script>,
    pub func_labels: HashMap<FuncSign, String>,
    path_aliases: HashMap<String, PathBuf>
}

impl Default for Context{
    fn default() -> Self {
        Context { 
            registers: Vec::default(),
            labels: Vec::default(),
            bindings: HashMap::default(),
            func_returns: HashMap::from_iter([
                (
                    FuncSign{
                        name: "float".to_string(),
                        params: vec![Type::Int]
                    }, Type::Float
                ),
                (
                    FuncSign{
                        name: "int".to_string(),
                        params: vec![Type::Float]
                    }, Type::Int
                ),

                (
                    FuncSign{
                        name: "sample".to_string(),
                        params: vec![Type::Int, Type::Int]
                    }, Type::Color
                ),

                (
                    FuncSign{
                        name: "width".to_string(),
                        params: vec![]
                    }, Type::Int
                ),

                (
                    FuncSign{
                        name: "height".to_string(),
                        params: vec![]
                    }, Type::Int
                ),

                (
                    FuncSign{
                        name: "red".to_string(),
                        params: vec![Type::Color]
        
                    }, Type::Int
                ),

                (
                    FuncSign{
                        name: "green".to_string(),
                        params: vec![Type::Color]
        
                    }, Type::Int
                ),
        
                (
                    FuncSign{
                        name: "blue".to_string(),
                        params: vec![Type::Color]
        
                    }, Type::Int
                ),
        
                (
                    FuncSign{
                        name: "alpha".to_string(),
                        params: vec![Type::Color]
        
                    }, Type::Int
                ),

                (
                    FuncSign{
                        name: "rgba".to_string(),
                        params: vec![Type::Int, Type::Int, Type::Int, Type::Int]
                    }, Type::Color
                )
            ]),
            renamed_vars: HashMap::default(),
            top_function: String::default(),
            imports: vec![],
            func_labels: HashMap::from_iter([

                (
                    FuncSign{
                        name: "float".to_string(),
                        params: vec![Type::Int]
                    }, "float".to_string()
                ),

                (
                    FuncSign{
                        name: "int".to_string(),
                        params: vec![Type::Float]
                    }, "int".to_string()
                ),

                (
                    FuncSign{
                        name: "sample".to_string(),
                        params: vec![Type::Int, Type::Int]
                    }, "sample".to_string()
                ),

                (
                    FuncSign{
                        name: "width".to_string(),
                        params: vec![]
                    }, "width".to_string()
                ),

                (
                    FuncSign{
                        name: "height".to_string(),
                        params: vec![]
                    }, "height".to_string()
                ),

                (
                    FuncSign{
                        name: "red".to_string(),
                        params: vec![Type::Color]
        
                    }, "red".to_string()
                ),

                (
                    FuncSign{
                        name: "green".to_string(),
                        params: vec![Type::Color]
        
                    }, "green".to_string()
                ),
        
                (
                    FuncSign{
                        name: "blue".to_string(),
                        params: vec![Type::Color]
        
                    }, "blue".to_string()
                ),
        
                (
                    FuncSign{
                        name: "alpha".to_string(),
                        params: vec![Type::Color]
        
                    }, "alpha".to_string()
                ),

                (
                    FuncSign{
                        name: "rgba".to_string(),
                        params: vec![Type::Int, Type::Int, Type::Int, Type::Int]
                    }, "rgba".to_string()
                )
            ]),
            path_aliases: HashMap::new()
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
        self.add_register(format!("_r{}", self.registers.len()), _type);
        self.registers.last().unwrap().clone()
    }

    pub fn add_label(&mut self, name: &str){
        if self.top_function.is_empty(){
            let n = self.labels.iter().filter(|e| e.starts_with(name)).count();

            if n == 0{
                self.labels.push(name.to_string());
            }else{
                self.labels.push(format!("{name}{n}"));
            }

        }else{
            let n = self.labels.iter().filter(|e| e.starts_with(&format!("{}_{name}", self.top_function))).count();

            if n == 0{
                self.labels.push(format!("{}_{name}", self.top_function));
            }else{
                self.labels.push(format!("{}_{name}{n}", self.top_function));
            }
        }
    }

    pub fn create_temp_label(&mut self, tag: &str) -> String{
        self.add_label(&format!("_{tag}"));
        self.labels.last().unwrap().clone()
    }

    pub fn has_script(&self, path:&Path) -> bool{
        self.imports.iter().any(|e| &e.path == path)
    }


}

#[derive(Debug, Clone)]
pub struct Script{
    pub path: PathBuf,
    pub program: Vec<Instruction>
}

#[derive(Debug)]
pub struct Runtime{
    pub scripts: Vec<Script>
}

pub fn parse(forest: &Vec<AST>) -> Runtime{
    let path = Path::new(&forest[0].kind.location.file).to_path_buf();
    let mut ctx = Context::default();

    let program = _parse(forest, &mut ctx);

    let main_file = Script{path, program};
    let mut scripts = vec![main_file];

    for imported_script in ctx.imports{
        if imported_script.path != scripts[0].path{
            scripts.push(imported_script);
        }
    }


    Runtime { scripts}
}


fn _parse(forest: &Vec<AST>, ctx: &mut Context) -> Vec<Instruction>{
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
            let (mut instr, _) = parse_func_call(tree, None, ctx);
            instructions.append(&mut instr);
        
        }else if tree.kind.kind == TokenType::While{
            instructions.append(&mut parse_while(tree, ctx));
        
        }else if tree.kind.kind == TokenType::If{
            let root_scope = ctx.create_temp_label("root_scope");
            instructions.append(&mut parse_if(tree, ctx, root_scope.clone()));
            instructions.push(Instruction::Label(root_scope));
        
        }else if tree.kind.kind == TokenType::Subcanvas{
            instructions.append(&mut parse_subcanvas(tree, ctx));
        
        }else if tree.kind.kind == TokenType::Pub{
            instructions.append(&mut parse_def(&tree.children[0], ctx));
        
        }else if tree.kind.kind == TokenType::Import{
            let self_path = Path::new(&tree.kind.location.file);
            let has_aliasing = tree.children[0].kind.kind == TokenType::As;

            let str_literal = if has_aliasing{
                &tree.children[0].children[0].kind.literal
            }else{
                &tree.children[0].kind.literal
            };
            let mut content = String::from(&str_literal[1..str_literal.len()-1]);
            content.push_str(".pprs");

            let path = if Path::new(&content).is_relative(){
                self_path.parent().unwrap().join(Path::new(&content))
            }else{
                Path::new(&content).to_path_buf()
            };

            let forest = crate::prepare(path.to_str().unwrap());
            let mut import_ctx = Context::default();
            let program = _parse(&forest, &mut import_ctx);

            let script = Script{path: path.clone(), program};

            if !ctx.has_script(&path){
                ctx.imports.push(script);
            }

            let script_name = if has_aliasing{
                &tree.children[0].children[1].kind.literal
            }else{
                path.file_stem().unwrap().to_str().unwrap()
            };

            for (sign, ret) in import_ctx.func_returns{
                let s = FuncSign{name: format!("{}.{}", script_name, sign.name), params: sign.params};
                ctx.func_returns.insert(s, ret);
            }

            for (sign, label) in import_ctx.func_labels{
                let s = FuncSign{name: format!("{}.{}", script_name, sign.name), params: sign.params};
                ctx.func_labels.insert(s, label);
            }

            for imported_script in import_ctx.imports{
                if !ctx.has_script(&imported_script.path){
                    ctx.imports.push(imported_script);
                }
            }

            ctx.path_aliases.insert(script_name.to_string(), path);
        
        }else if tree.kind.kind == TokenType::Dot{
            let script_name = tree.children[0].kind.literal.clone();
            let (mut instr, _) = parse_func_call(&tree.children[1], Some(script_name), ctx);
            instructions.append(&mut instr);
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
        let n = token.literal.clone();
        let name = if ctx.renamed_vars.contains_key(&n){
            ctx.renamed_vars.get(&n).unwrap().clone()
        }else{
            n
        };
        (Param::Register(name.clone()), ctx.bindings.get(&name).unwrap().clone() )
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
            (Param::Value(1), Type::Bool)
        }else{ panic!("Should not be there") }

    }else if token.kind == TokenType::Hex{
        let lit = token.literal.clone();
        (
            Param::Value(u32::from_str_radix(&lit[1..], 16).expect("Unable to parse to u64")),
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
       
        (p1, p2, return_reg)
    
    }else if !left.children.is_empty() && right.children.is_empty(){
        let reg = ctx.create_temp_register(None);
        let mut instr:Vec<Instruction>;
        
        (instr, left_type) = expand_expr(left, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), left_type);
        
        instructions.append(&mut instr);

        let p1 = Param::Register(reg);
        let p2:Param;
        (p2, right_type) = to_param(&right.kind, ctx);

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
            let reg = String::from("_rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.0, reg.clone()));
            instructions.push(Instruction::Addf(Param::Register(reg), args.1, args.2));
            Type::Float
        
        }else{
            let reg = String::from("_rt");
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
            let reg = String::from("_rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.0, reg.clone()));
            instructions.push(Instruction::Subf(Param::Register(reg), args.1, args.2));
            Type::Float
        
        }else{
            let reg = String::from("_rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.1, reg.clone()));
            instructions.push(Instruction::Subf(args.0, Param::Register(reg), args.2));
            Type::Float
        }
    
    }else if expr.kind.kind == TokenType::Mul{
        if left_type == Type::Float && right_type == Type::Float{
            instructions.push(Instruction::Mulf(args.0, args.1, args.2));
            Type::Float
        
        }else if left_type == Type::Int && right_type ==Type::Int{
            instructions.push(Instruction::Mul(args.0, args.1, args.2));
            Type::Int
        
        }else if left_type == Type::Int && right_type == Type::Float{
            let reg = String::from("_rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.0, reg.clone()));
            instructions.push(Instruction::Mulf(Param::Register(reg), args.1, args.2));
            Type::Float
        
        }else{
            let reg = String::from("_rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.1, reg.clone()));
            instructions.push(Instruction::Mulf(args.0, Param::Register(reg), args.2));
            Type::Float
        }
    
    }else if expr.kind.kind == TokenType::Div{
        if left_type == Type::Float && right_type == Type::Float{
            instructions.push(Instruction::Divf(args.0, args.1, args.2));
            Type::Float
        
        }else if left_type == Type::Int && right_type ==Type::Int{
            instructions.push(Instruction::Div(args.0, args.1, args.2));
            Type::Int
        
        }else if left_type == Type::Int && right_type == Type::Float{
            let reg = String::from("_rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.0, reg.clone()));
            instructions.push(Instruction::Divf(Param::Register(reg), args.1, args.2));
            Type::Float
        
        }else{
            let reg = String::from("_rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.1, reg.clone()));
            instructions.push(Instruction::Divf(args.0, Param::Register(reg), args.2));
            Type::Float
        }

    }else if expr.kind.kind == TokenType::Pow{
        if left_type == Type::Float && right_type == Type::Float{
            instructions.push(Instruction::Powf(args.0, args.1, args.2));
            Type::Float
        
        }else if left_type == Type::Int && right_type ==Type::Int{
            instructions.push(Instruction::Pow(args.0, args.1, args.2));
            Type::Int
        
        }else if left_type == Type::Int && right_type == Type::Float{
            let reg = String::from("_rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.0, reg.clone()));
            instructions.push(Instruction::Powf(Param::Register(reg), args.1, args.2));
            Type::Float
        
        }else{
            let reg = String::from("_rt");
            ctx.bindings.insert(reg.clone(), Type::Float);

            instructions.push(Instruction::Flt(args.1, reg.clone()));
            instructions.push(Instruction::Powf(args.0, Param::Register(reg), args.2));
            Type::Float
        }

    }else if expr.kind.kind == TokenType::Mod{
        instructions.push(Instruction::Mod(args.0, args.1, args.2));
        Type::Int
    
    }else if expr.kind.kind == TokenType::GT{
        if left_type == Type::Int && right_type == Type::Int{
            instructions.push(Instruction::GT(args.0, args.1, args.2));
            Type::Bool
        }else{
            instructions.push(Instruction::GTf(args.0, args.1, args.2));
            Type::Bool
        }
        
    
    }else if expr.kind.kind == TokenType::LT{
        if left_type == Type::Int && right_type == Type::Int{
            instructions.push(Instruction::LT(args.0, args.1, args.2));
            Type::Bool
        }else{
            instructions.push(Instruction::LTf(args.0, args.1, args.2));
            Type::Bool
        }
    
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
        if left_type == Type::Int && right_type == Type::Int{
            instructions.push(Instruction::GE(args.0, args.1, args.2));
            Type::Bool
        }else{
            instructions.push(Instruction::GEf(args.0, args.1, args.2));
            Type::Bool
        }
    
    }else if expr.kind.kind == TokenType::LTEq{
        if left_type == Type::Int && right_type == Type::Int{
            instructions.push(Instruction::LE(args.0, args.1, args.2));
            Type::Bool
        }else{
            instructions.push(Instruction::LEf(args.0, args.1, args.2));
            Type::Bool
        }
   
    }else if expr.kind.kind == TokenType::NotEq{
        instructions.push(Instruction::NE(args.0, args.1, args.2));
        Type::Bool
    
    }else{ panic!("This is not a binary operation"); };

    (instructions, _type)
}


fn expand_unary_expr(expr: &AST, ctx: &mut Context, return_reg:String) -> (Vec<Instruction>, Type){
    let operand = &expr.children[0];
    let mut instructions = vec![];
    let _type:Type;

    let args = if operand.children.is_empty(){
        let p:Param;
        (p, _type) = to_param(&operand.kind, ctx);

        (p, return_reg)
    }else{
        let reg = ctx.create_temp_register(None);
        let mut instr:Vec<Instruction>;
        
        (instr, _type) = expand_expr(operand, ctx, reg.clone());
        ctx.bindings.insert(reg.clone(), _type);
        instructions.append(&mut instr);
        let p = Param::Register(reg);

        (p, return_reg)
    };

    if expr.kind.kind == TokenType::Not{
        instructions.push(Instruction::Not(args.0, args.1));

    }else if expr.kind.kind == TokenType::Minus{
        if _type == Type::Int{
            instructions.push(Instruction::Neg(args.0, args.1));
        }else{
            instructions.push(Instruction::Negf(args.0, args.1));
        }


    }else{ panic!("This is not an unary operation"); };

    (instructions, _type)
}


fn expand_expr(expr:&AST, ctx: &mut Context, return_reg: String) -> (Vec<Instruction>, Type){

    if expr.kind.kind == TokenType::Dot{
        let script_name = expr.children[0].kind.literal.clone();
        let (mut instructions, func_sign) = parse_func_call(&expr.children[1], Some(script_name), ctx);
        
        instructions.push(Instruction::Copy(Param::Register("_rt".to_string()), return_reg));
        return (instructions, ctx.func_returns.get(&func_sign).unwrap_or(&Type::Void).clone())
    
    }else if expr.children.len() == 2{
        expand_binary_expr(expr, ctx, return_reg)

    }else if expr.kind.kind == TokenType::Ident{
        let (mut instructions, func_sign) = parse_func_call(expr, None,ctx);

        instructions.push(Instruction::Copy(Param::Register("_rt".to_string()), return_reg));

        return (instructions, ctx.func_returns.get(&func_sign).unwrap_or(&Type::Void).clone());
    }else{
        expand_unary_expr(expr, ctx, return_reg)
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

    let normalized_name = if !ctx.renamed_vars.contains_key(&name){
        let r = ctx.create_temp_register(None);
        ctx.renamed_vars.insert(name, r.clone());
        r
    }else{ ctx.renamed_vars.get(&name).unwrap().clone() };


    if !expr.children.is_empty(){
        let (mut instr, t) = expand_expr(expr, ctx, normalized_name.clone());
        ctx.bindings.insert(normalized_name, t);
        instructions.append(&mut instr);

        
    }else{

        let (param, t) = to_param(&expr.kind, ctx);
        ctx.bindings.insert(normalized_name.clone(), t);
        instructions.push(Instruction::Copy(param, normalized_name));
    }

    instructions
}

fn parse_def(def_tree: &AST, parent:&mut Context) -> Vec<Instruction>{
    let mut instructions = vec![];
    let func_tree = &def_tree.children[0];
    let block = &def_tree.children[def_tree.children.len()-1];

    let ret_type = if def_tree.children.len() == 3{
        get_type(def_tree.children[1].kind.literal.clone()).unwrap()
    }else{
        Type::Void
    };


    let mut ctx = Context::default();
    ctx.top_function = func_tree.kind.literal.clone();

    parent.add_label(&func_tree.kind.literal);

    instructions.push(Instruction::Label(parent.labels.last().unwrap().clone()));

    let mut params = vec![];

    for (i, param) in func_tree.children.iter().enumerate(){
        let r = format!("p{i}");
        let t = get_type(param.children[1].kind.literal.clone());
        params.push(t.unwrap());
        ctx.add_register(r.clone(), t);
        ctx.renamed_vars.insert(param.children[0].kind.literal.clone(), r);
    }

    let sign = FuncSign{name: func_tree.kind.literal.clone(), params};

    parent.func_labels.insert(sign.clone(), parent.labels.last().unwrap().clone());
    parent.func_returns.insert(sign, ret_type);
    
    ctx.path_aliases = parent.path_aliases.clone();
    ctx.func_labels = parent.func_labels.clone();
    ctx.func_returns = parent.func_returns.clone();

    instructions.append(&mut _parse(&block.children, &mut ctx));
    instructions.push(Instruction::Ret);

    instructions
}

fn parse_return(return_tree: &AST, ctx: &mut Context) -> Vec<Instruction>{
    if return_tree.children.is_empty(){ return vec![]; }

    let expr = &return_tree.children[0];

    if !expr.children.is_empty(){
        let (instr, t) = expand_expr(expr, ctx, "rt".to_string());
        ctx.bindings.insert("_rt".to_string(), t);
        
        instr
    }else{
        let (p, t) = to_param(&expr.kind, ctx);
        ctx.bindings.insert("_rt".to_string(), t);

        vec![Instruction::Copy(p, "_rt".to_string())]
    }
}
fn parse_func_call(func_call_tree: &AST, script_name:Option<String>,ctx: &mut Context) -> (Vec<Instruction>, FuncSign){
    let name = func_call_tree.kind.literal.clone();
    let mut instructions = vec![];
    let mut params = vec![];

    let mut params_type = vec![];


    for arg in &func_call_tree.children[0].children{
        if !arg.children.is_empty(){
            let reg = ctx.create_temp_register(None);
            let (mut instr, t) = expand_expr(arg, ctx, reg.clone());
            params_type.push(t);
            ctx.bindings.insert(reg.clone(), t);

            instructions.append(&mut instr);

            params.push(Param::Register(reg));
        }else{
            let (p, t) = to_param(&arg.kind, ctx);
            params.push(p);
            params_type.push(t);
        }
    }

    let sign = if let Some(script_name) = script_name.clone(){
        FuncSign{name: format!("{script_name}.{name}"), params: params_type}
    }else{
        FuncSign{name: name.clone(), params: params_type}
    };

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
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Float);

        instructions.push(Instruction::Flt(params[0].clone(), reg));
    
    }else if &name == "int"{
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Float);

        instructions.push(Instruction::Int(params[0].clone(), reg));
    }else if &name == "sample"{
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Color);

        instructions.push(Instruction::Sample(params[0].clone(), params[1].clone(), reg));

    }else if &name == "width"{
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Int);

        instructions.push(Instruction::Width(reg));

    }else if &name == "height"{
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Int);

        instructions.push(Instruction::Height(reg));

    }else if &name == "red"{
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Int);

        instructions.push(Instruction::Red(params[0].clone(), reg));

    }else if &name == "green"{
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Int);

        instructions.push(Instruction::Green(params[0].clone(), reg));

    }else if &name == "blue"{
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Int);

        instructions.push(Instruction::Blue(params[0].clone(), reg));

    }else if &name == "alpha"{
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Int);

        instructions.push(Instruction::Alpha(params[0].clone(), reg));

    }else if &name == "rgba"{
        let reg = String::from("_rt");
        ctx.bindings.insert(reg.clone(), Type::Color);

        instructions.push(Instruction::RGBA(params[0].clone(), params[1].clone(), params[2].clone(), params[3].clone(), reg));

    }else{
        let unique_name = ctx.func_labels.get(&sign).unwrap().clone();

        if let Some(script_name) = script_name{
            let path = ctx.path_aliases.get(&script_name).unwrap().to_str().unwrap();
            instructions.push(Instruction::Import(path.to_string(), script_name.clone()));
            instructions.push(Instruction::Call(format!("{script_name}.{unique_name}"), params));
        
        }else{
            instructions.push(Instruction::Call(unique_name, params));
        }
    }


    (instructions, sign)
}


fn parse_while(while_tree: &AST, ctx: &mut Context) -> Vec<Instruction>{
    let expr = &while_tree.children[0];
    let block = &while_tree.children[1];


    let while_start = ctx.create_temp_label("while");
    let mut instructions = vec![Instruction::Label(while_start.clone())];
    let end_label = format!("_end_{}", while_start);


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
    instructions.push(Instruction::JF(param, end_label.clone()));

    instructions.append(&mut _parse(&block.children, ctx));
    
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
        instructions.append(&mut _parse(&block.children, ctx));
    }else{
        let else_tree = &if_tree.children[2];
        
        if else_tree.children[0].kind.kind == TokenType::If{
            let n = ctx.labels.iter().filter(|e| e.starts_with("_elif")).count();

            let else_if_start = format!("_elif{}", n);
            instructions.push(Instruction::JF(param, else_if_start.clone()));
            instructions.append(&mut _parse(&block.children, ctx));
            instructions.push(Instruction::Jump(root_scope_label.clone()));
            
            instructions.push(Instruction::Label(else_if_start));
            instructions.append(&mut parse_if(&else_tree.children[0], ctx, root_scope_label));
        }else{
            let else_labl = ctx.create_temp_label("else");
            instructions.push(Instruction::JF(param, else_labl.clone()));
            instructions.append(&mut _parse(&block.children, ctx));
            instructions.push(Instruction::Jump(root_scope_label));
           
            instructions.push(Instruction::Label(else_labl));
            instructions.append(&mut _parse(&else_tree.children[0].children, ctx));

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
    instructions.append(&mut _parse(&block.children, ctx));
    instructions.push(Instruction::Merge(x_param, y_param));

    instructions
}
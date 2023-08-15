use std::{collections::{HashMap, HashSet}, fmt::Display};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Type{
    Int, Float,
    Color,
    Bool,
    Void
}


#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct FuncSign{
    pub name: String,
    pub params: Vec<Type>
}

impl Display for FuncSign{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}({:?})", self.name, self.params))
    }
}

#[derive(Debug, Clone)]
pub struct Environment{
    pub scope_level: usize,
    contexts: HashSet<String>,
    types: HashSet<String>,
    variables: HashMap<String, (Type, Option<String>)>,
    func_signs: HashSet<FuncSign>,
    func_returns: HashMap<FuncSign, Type>
}

impl Default for Environment{
    fn default() -> Self {
        let mut env = Environment::new();
        
        env.push_type("int");
        env.push_type("float");
        env.push_type("bool");
        env.push_type("color");

        env.push_func_sign(FuncSign{
            name: String::from("create_canvas"),
            params: vec![Type::Int, Type::Int]
        }, Type::Void);

        env.push_func_sign(FuncSign{
            name: String::from("save_canvas"),
            params: vec![]
        }, Type::Void);

        env.push_func_sign(FuncSign{
            name: String::from("put"),
            params: vec![Type::Int, Type::Int, Type::Color]
        }, Type::Void);

        env.push_func_sign(FuncSign{
            name: String::from("fill"),
            params: vec![Type::Color]
        }, Type::Void);
        
        env
    }
}

impl Environment{
    pub fn new() -> Self{
        Self { scope_level: 0, contexts: HashSet::new(), types: HashSet::new(), variables: HashMap::new(), func_signs: HashSet::new(), func_returns: HashMap::new() }
    }

    pub fn add_ctx(&mut self, ctx:&str){
        self.contexts.insert(String::from(ctx));
    }
    pub fn has_ctx(&self, ctx:&str) -> bool{
        self.contexts.contains(ctx)
    }


    pub fn push_type(&mut self, name:&str){
        self.types.insert(String::from(name));
    }
    pub fn has_type(&self, name:&str) -> bool{
        self.types.contains(name)
    }


    pub fn push_assign_var(&mut self, name:&str, _type: Type, value: &str){
        self.variables.insert(String::from(name), (_type, Some(String::from(value))));
    }

    pub fn push_var(&mut self, name:&str, _type: Type){
        self.variables.insert(String::from(name), (_type, None));
    }

    pub fn get_var(&self, name:&str) -> Option<&(Type, Option<String>)>{
        self.variables.get(name)
    }

    pub fn has_var(&self, name:&str) -> bool{
        self.variables.contains_key(name)
    }

    pub fn push_func_sign(&mut self, func_sign:FuncSign, return_type: Type){
        self.func_signs.insert(func_sign.clone());
        self.func_returns.insert(func_sign, return_type);
    }

    pub fn has_func_sign(&self, func_sign: &FuncSign) -> bool{
        self.func_signs.contains(func_sign)
    }


    pub fn get_func_return(&self, func_sign: &FuncSign) -> Option<Type>{
        self.func_returns.get(func_sign).copied()
    }
}

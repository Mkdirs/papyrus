use std::{collections::HashMap, path::{Path, PathBuf}};

use crate::{ir::{Instruction, Param, Runtime, Script, ValueType}, to_rgba};


#[derive(Debug)]
struct StackFrame{
    registers: HashMap<String, u64>
}


impl Default for StackFrame{
    fn default() -> Self {
        StackFrame { registers: HashMap::from_iter([("_rt".to_string(), 0)]) }
    }
}

impl StackFrame{
    fn get(&self, register:&str) -> u64{
        *self.registers.get(register).expect(&format!("The register {} was not found", register))
    }

    fn set(&mut self, register: &str, value:u64){
        self.registers.insert(register.to_string(), value);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Canvas{
    pub width: u32,
    pub height: u32,
    pub data: Vec<u64>
}

impl Canvas{
    pub fn new(width: u32, height: u32) -> Self{
        let mut data:Vec<u64> = vec![];
        for _ in 0..width*height{
            data.push(0);
        }
        Canvas { width, height, data}
    }

    pub fn merge(&mut self, offst_x:i32, offst_y:i32, other:Self){
        for y in offst_y..(offst_y+other.height as i32){
            for x in offst_x..(offst_x+other.width as i32){
                if (x >= 0 && x < self.width as i32) && (y >= 0 && y < self.height as i32){
                    let i = y * (self.width as i32) + x;
                    let other_i = (y-offst_y) * (other.width as i32) + (x-offst_x);

                    self.data[i as usize] = other.data[other_i as usize];
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct VM{
    runtime: Runtime,
    prog_counter: usize,
    memory: Vec<StackFrame>,
    canvas: Vec<Canvas>,
    saved_canvas: Vec<Canvas>,
    path_aliases: HashMap<String, PathBuf>
}

impl VM{
    pub fn new(runtime: Runtime) -> Self{
        VM{
            runtime,
            prog_counter: 0,
            memory: vec![StackFrame::default()],
            canvas: vec![],
            saved_canvas: vec![],
            path_aliases: HashMap::new()
        }
    }

    pub fn get_saved_canvas(&self) -> &[Canvas]{
        &self.saved_canvas
    }

    pub fn get_script(&self, path:&Path) -> Option<&Script>{
        self.runtime.scripts.iter().find(|e| &e.path == path)
    }

    fn get_indx_of(&self, label: &str, script_path: &Path) -> usize{
        let script = self.get_script(script_path).expect(&format!("Script {} was not found", script_path.display()));
        script.program.iter().enumerate().find(|(_, instr)| {
            &Instruction::Label(label.to_string()) == *instr
        }).expect(&format!("Label {} was not found", label)).0
    }

    pub fn run(&mut self, script_path: &Path, entry_point:&str){
        self.prog_counter = self.get_indx_of(entry_point, script_path);
        let script = self.get_script(script_path).expect("msg").clone();
        for instr in &script.program{
            println!("{instr:?}");
        }
        loop{
            if self.prog_counter >= script.program.len(){
                panic!("Unexpected end");
            }
            if! self.exec(&script){ break; }
            else{ self.prog_counter +=1; }
        }
        
    }

    fn exec(&mut self, script:&Script) -> bool{
        let instruction = script.program[self.prog_counter].clone();
        match instruction{
            Instruction::Add(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = left+right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::Addf(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = left+right;

                self.memory[0].set(&r, result.to_bits() as u64);
                true
            },

            Instruction::And(a, b, r) =>{
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0,

                    _ => panic!("bools can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0,

                    _ => panic!("bools can't be 64 bits")
                };

                let result = left && right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::Alpha(c, r) => {
                let color = match c {
                    Param::Value(ValueType::Long(v)) => v,
                    Param::Register(reg) => self.memory[0].get(&reg),

                    _ => panic!("colors can't be 32 bits")
                };

                let [_, _, _, a] = to_rgba(color);
                self.memory[0].set(&r, a as u64);
                true
            },

            Instruction::Blue(c, r) => {
                let color = match c {
                    Param::Value(ValueType::Long(v)) => v,
                    Param::Register(reg) => self.memory[0].get(&reg),

                    _ => panic!("colors can't be 32 bits")
                };

                let [_, b, _, _] = to_rgba(color);
                self.memory[0].set(&r, b as u64);
                true
            },

            Instruction::Call(f, params) => {
                let mut stack = StackFrame::default();
                for (i, param) in params.iter().enumerate(){
                    let reg = &format!("p{i}");
                    match param{
                        Param::Value(ValueType::Big(v)) => stack.set(reg, *v as u64),
                        Param::Value(ValueType::Long(v)) => stack.set(reg, *v),
                        Param::Register(r) => stack.set(reg, self.memory[0].get(r))
                    }
                }

                let old_pc = self.prog_counter;
                self.memory.insert(0, stack);

                if f.contains('.'){
                    let list = f.split('.').collect::<Vec<&str>>();
                    let script = list[0];
                    let func_label = &list[1];

                    let old_aliases = self.path_aliases.clone();
                    self.path_aliases.clear();

                    let path = old_aliases.get(script).unwrap();

                    self.run(&path, &func_label);
                    self.path_aliases = old_aliases;
                    
                }else{
                    self.run(&script.path, &f);
                }


                let callee_stack = self.memory.remove(0);

                self.memory[0].set("_rt", callee_stack.get("_rt"));
                self.prog_counter = old_pc;
                true
            },

            Instruction::Copy(a, r) => {
                match a{
                    Param::Value(ValueType::Big(v)) => self.memory[0].set(&r, v as u64),
                    Param::Value(ValueType::Long(v)) => self.memory[0].set(&r, v),
                    Param::Register(reg) => {
                        let v = self.memory[0].get(&reg);
                        self.memory[0].set(&r, v);
                    }
                }
                true
            },

            Instruction::Div(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = left / right;

                self.memory[0].set(&r, result as u64);

                true
            },

            Instruction::Divf(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = left / right;

                self.memory[0].set(&r, result.to_bits() as u64);

                true
            },

            Instruction::Eq(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as u64,
                    Param::Value(ValueType::Long(v)) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as u64,
                    Param::Value(ValueType::Long(v)) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let result = left == right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::Fill(c) => {
                match c{
                    Param::Value(ValueType::Long(v)) => {
                        for i in 0..self.canvas[0].data.len(){
                            self.canvas[0].data[i] = v;
                        }
                    },

                    Param::Register(reg) => {
                        let v = self.memory[0].get(&reg);

                        for i in 0..self.canvas[0].data.len(){
                            self.canvas[0].data[i] = v;
                        }
                    },

                    _ => panic!("colors can't be 32 bits")
                }

                true
            },

            Instruction::Flt(a, r) => {
                let value = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = value as f32;

                self.memory[0].set(&r, result.to_bits() as u64);

                true
            },

            Instruction::GE(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = left >= right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::GEf(a, b, r) => {

                let left = match a{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = left >= right;

                self.memory[0].set(&r, result as u64);

                true
            },

            Instruction::Green(c, r) => {
                let color = match c {
                    Param::Value(ValueType::Long(v)) => v,
                    Param::Register(reg) => self.memory[0].get(&reg),

                    _ => panic!("colors can't be 32 bits")
                };

                let [_, g, _, _] = to_rgba(color);
                self.memory[0].set(&r, g as u64);
                true
            },

            Instruction::GT(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = left > right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::GTf(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = left > right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::Height(r) => {
                self.memory[0].set(&r, self.canvas[0].height as u64);
                true
            },

            Instruction::Import(path, name) => {
                self.path_aliases.insert(name, Path::new(&path).to_path_buf());
                true
            },

            Instruction::Int(a, r) => {
                let value = match a{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = value as i32;

                self.memory[0].set(&r, result as u64);

                true
            },

            Instruction::JF(a, label) => {
                let value = match a{
                    Param::Value(ValueType::Big(v)) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0,

                    _ => panic!("bools can't be 64 bits")
                };

                if !value{
                    self.prog_counter = self.get_indx_of(&label, &script.path);
                }

                true
            },

            Instruction::Jump(label) =>{
                self.prog_counter = self.get_indx_of(&label, &script.path);
                true
            },

            Instruction::LE(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = left <= right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::LEf(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = left <= right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::LT(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = left < right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::LTf(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = left < right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::Label(_) => {true},

            Instruction::Merge(x, y) => {
                let x = match x {
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let y = match y {
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let to_merge = self.canvas.remove(0);

                self.canvas[0].merge(x, y, to_merge);


                true
            },

            Instruction::Mod(a, b, r) => {
                let left = match a {
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b {
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = left % right;

                self.memory[0].set(&r, result as u64);

                true
            },

            Instruction::Mul(a, b, r) => {
                let left = match a {
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b {
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = left * right;

                self.memory[0].set(&r, result as u64);

                true
            },

            Instruction::Mulf(a, b, r) => {
                let left = match a {
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let right = match b {
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = left * right;

                self.memory[0].set(&r, result.to_bits() as u64);

                true
            },

            Instruction::NE(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as u64,
                    Param::Value(ValueType::Long(v)) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as u64,
                    Param::Value(ValueType::Long(v)) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let result = left != right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::Neg(a, r) => {
                let value = match a {
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                self.memory[0].set(&r, (-value) as u64);
                true
            },

            Instruction::Negf(a, r) => {
                let value = match a {
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                self.memory[0].set(&r, (-value).to_bits() as u64);
                true
            },

            Instruction::Not(a, r) => {
                let value = match a {
                    Param::Value(ValueType::Big(v)) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0,

                    _ => panic!("bools can't be 64 bits")
                };

                self.memory[0].set(&r, (!value) as u64);
                true
            },

            Instruction::Or(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0,

                    _ => panic!("bools can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0,

                    _ => panic!("bools can't be 64 bits")
                };

                let result = left || right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::Pop => {
                self.canvas.remove(0);
                true
            },

            Instruction::Pow(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = if right < 0{
                    (left as f32).powf(right as f32) as i32
                }else{
                    left.pow(right as u32)
                };

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::Powf(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = left.powf(right);

                self.memory[0].set(&r, result.to_bits() as u64);
                true
            },

            Instruction::Push(a, b) => {
                let mut left = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let mut right = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                if left < 0 {left = 0;}
                if right < 0 {right = 0;}

                self.canvas.insert(0, Canvas::new(left as u32, right as u32));

                true
            },

            Instruction::Put(x, y, c) => {
                let x = match x{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let y = match y{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let color = match c{
                    Param::Value(ValueType::Long(v)) => v,
                    Param::Register(reg) => self.memory[0].get(&reg),

                    _ => panic!("colors can't be 32 bits")
                };

                let width = self.canvas[0].width;
                let height = self.canvas[0].height;

                if (0 <= x && x < width as i32) && (0 <= y && y < height as i32){
                    let i = y as usize * width as usize + x as usize;
                    self.canvas[0].data[i] = color;
                }


                true
            },

            Instruction::Red(c, r) => {
                let color = match c {
                    Param::Value(ValueType::Long(v)) => v,
                    Param::Register(reg) => self.memory[0].get(&reg),

                    _ => panic!("colors can't be 32 bits")
                };

                let [red, _, _, _] = to_rgba(color);
                self.memory[0].set(&r, red as u64);
                true
            },

            Instruction::RGBA(r, g, b, a, reg) => {
                let mut r = match r{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(register) => self.memory[0].get(&register) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let mut g = match g{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(register) => self.memory[0].get(&register) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let mut b = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(register) => self.memory[0].get(&register) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let mut a = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(register) => self.memory[0].get(&register) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                r = (r.clamp(0, 255) & 0xff) << 24;
                g = (g.clamp(0, 255) & 0xff) << 16;
                b = (b.clamp(0, 255) & 0xff) << 8;
                a = a.clamp(0, 255) & 0xff;

                let color = r | g | b | a;

                self.memory[0].set(&reg, color as u64);


                true
            }

            Instruction::Ret => {false},

            Instruction::Sample(x, y, r) => {
                let x = match x {
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let y = match y {
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let width = self.canvas[0].width;
                let height = self.canvas[0].height;

                if (0 <= x && x < width as i32) && (0 <= y && y < height as i32){
                    let i = y as usize * width as usize + x as usize;
                    self.memory[0].set(&r, self.canvas[0].data[i]);
                }

                true
            }

            Instruction::Save => {
                self.saved_canvas.push(self.canvas[0].clone());
                true
            },

            Instruction::Sub(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32,

                    _ => panic!("ints can't be 64 bits")
                };

                let result = left - right;

                self.memory[0].set(&r, result as u64);
                true
            },

            Instruction::Subf(a, b, r) => {
                let left = match a{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let right = match b{
                    Param::Value(ValueType::Big(v)) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg) as u32),

                    _ => panic!("floats can't be 64 bits")
                };

                let result = left - right;

                self.memory[0].set(&r, result.to_bits() as u64);
                true
            },

            Instruction::Width(r) => {
                self.memory[0].set(&r, self.canvas[0].width as u64);
                true
            }
        }
    }


}


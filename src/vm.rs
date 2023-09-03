use std::{collections::HashMap, path::{Path, PathBuf}};

use crate::ir::{Instruction, Param, Runtime, Script};


#[derive(Debug)]
struct StackFrame{
    registers: HashMap<String, u32>
}


impl Default for StackFrame{
    fn default() -> Self {
        StackFrame { registers: HashMap::from_iter([("_rt".to_string(), 0)]) }
    }
}

impl StackFrame{
    fn get(&self, register:&str) -> u32{
        *self.registers.get(register).expect(&format!("The register {} was not found", register))
    }

    fn set(&mut self, register: &str, value:u32){
        self.registers.insert(register.to_string(), value);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Canvas{
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>
}

impl Canvas{
    pub fn new(width: usize, height: usize) -> Self{
        let mut data:Vec<u32> = vec![];
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
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = left+right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::Addf(a, b, r) => {
                let left = match a{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let right = match b{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = left+right;

                self.memory[0].set(&r, result.to_bits());
                true
            },

            Instruction::And(a, b, r) =>{
                let left = match a{
                    Param::Value(v) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0
                };

                let right = match b{
                    Param::Value(v) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0
                };

                let result = left && right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::Call(f, params) => {
                let mut stack = StackFrame::default();
                for (i, param) in params.iter().enumerate(){
                    let reg = &format!("p{i}");
                    match param{
                        Param::Value(v) => stack.set(reg, *v),
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
                    Param::Value(v) => self.memory[0].set(&r, v),
                    Param::Register(reg) => {
                        let v = self.memory[0].get(&reg);
                        self.memory[0].set(&r, v);
                    }
                }
                true
            },

            Instruction::Div(a, b, r) => {
                let left = match a{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = left / right;

                self.memory[0].set(&r, result as u32);

                true
            },

            Instruction::Divf(a, b, r) => {
                let left = match a{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let right = match b{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = left / right;

                self.memory[0].set(&r, result.to_bits());

                true
            },

            Instruction::Eq(a, b, r) => {
                let left = match a{
                    Param::Value(v) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let right = match b{
                    Param::Value(v) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let result = left == right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::Fill(c) => {
                match c{
                    Param::Value(v) => {
                        for i in 0..self.canvas[0].data.len(){
                            self.canvas[0].data[i] = v;
                        }
                    },

                    Param::Register(reg) => {
                        let v = self.memory[0].get(&reg);

                        for i in 0..self.canvas[0].data.len(){
                            self.canvas[0].data[i] = v;
                        }
                    }
                }

                true
            },

            Instruction::Flt(a, r) => {
                let value = match a{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = value as f32;

                self.memory[0].set(&r, result.to_bits());

                true
            },

            Instruction::GE(a, b, r) => {
                let left = match a{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = left >= right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::GEf(a, b, r) => {

                let left = match a{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let right = match b{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = left >= right;

                self.memory[0].set(&r, result as u32);

                true
            },

            Instruction::GT(a, b, r) => {
                let left = match a{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = left > right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::GTf(a, b, r) => {
                let left = match a{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let right = match b{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = left > right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::Import(path, name) => {
                self.path_aliases.insert(name, Path::new(&path).to_path_buf());
                true
            },

            Instruction::Int(a, r) => {
                let value = match a{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = value as i32;

                self.memory[0].set(&r, result as u32);

                true
            },

            Instruction::JF(a, label) => {
                let value = match a{
                    Param::Value(v) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0
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
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = left <= right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::LEf(a, b, r) => {
                let left = match a{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let right = match b{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = left <= right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::LT(a, b, r) => {
                let left = match a{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = left < right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::LTf(a, b, r) => {
                let left = match a{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let right = match b{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = left < right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::Label(_) => {true},

            Instruction::Merge(x, y) => {
                let x = match x {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let y = match y {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let to_merge = self.canvas.remove(0);

                self.canvas[0].merge(x, y, to_merge);


                true
            },

            Instruction::Mod(a, b, r) => {
                let left = match a {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = left % right;

                self.memory[0].set(&r, result as u32);

                true
            },

            Instruction::Mul(a, b, r) => {
                let left = match a {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = left * right;

                self.memory[0].set(&r, result as u32);

                true
            },

            Instruction::Mulf(a, b, r) => {
                let left = match a {
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let right = match b {
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = left * right;

                self.memory[0].set(&r, result.to_bits());

                true
            },

            Instruction::NE(a, b, r) => {
                let left = match a{
                    Param::Value(v) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let right = match b{
                    Param::Value(v) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let result = left != right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::Neg(a, r) => {
                let value = match a {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                self.memory[0].set(&r, (-value) as u32);
                true
            },

            Instruction::Negf(a, r) => {
                let value = match a {
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                self.memory[0].set(&r, (-value).to_bits());
                true
            },

            Instruction::Not(a, r) => {
                let value = match a {
                    Param::Value(v) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0
                };

                self.memory[0].set(&r, (!value) as u32);
                true
            },

            Instruction::Or(a, b, r) => {
                let left = match a{
                    Param::Value(v) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0
                };

                let right = match b{
                    Param::Value(v) => v != 0,
                    Param::Register(reg) => self.memory[0].get(&reg) != 0
                };

                let result = left || right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::Pop => {
                self.canvas.remove(0);
                true
            },

            Instruction::Pow(a, b, r) => {
                let left = match a{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = if right < 0{
                    (left as f32).powf(right as f32) as i32
                }else{
                    left.pow(right as u32)
                };

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::Powf(a, b, r) => {
                let left = match a{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let right = match b{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = left.powf(right);

                self.memory[0].set(&r, result.to_bits());
                true
            },

            Instruction::Push(a, b) => {
                let mut left = match a{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let mut right = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                if left < 0 {left = 0;}
                if right < 0 {right = 0;}

                self.canvas.insert(0, Canvas::new(left as usize, right as usize));

                true
            },

            Instruction::Put(x, y, c) => {
                let x = match x{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let y = match y{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let color = match c{
                    Param::Value(v) => v as u32,
                    Param::Register(reg) => self.memory[0].get(&reg) as u32
                };

                let width = self.canvas[0].width;
                let height = self.canvas[0].height;

                if (x >= 0 && x < width as i32) && (y >= 0 && y < height as i32){
                    let i = y as usize * width + x as usize;
                    self.canvas[0].data[i] = color;
                }


                true
            },

            Instruction::Ret => {false},

            Instruction::Save => {
                self.saved_canvas.push(self.canvas[0].clone());
                true
            },

            Instruction::Sub(a, b, r) => {
                let left = match a{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let right = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let result = left - right;

                self.memory[0].set(&r, result as u32);
                true
            },

            Instruction::Subf(a, b, r) => {
                let left = match a{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let right = match b{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = left - right;

                self.memory[0].set(&r, result.to_bits());
                true
            }
        }
    }


}


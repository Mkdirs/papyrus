use std::collections::HashMap;

use crate::ir::{Instruction, Param};


#[derive(Debug)]
struct StackFrame{
    registers: HashMap<String, u32>
}


impl Default for StackFrame{
    fn default() -> Self {
        StackFrame { registers: HashMap::from_iter([("rt".to_string(), 0)]) }
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

    pub fn merge(&mut self, offst_x:isize, offst_y:isize, other:Self){
        for y in 0..other.height{
            for x in 0..other.width{
                let my_x = (x as isize) + offst_x;
                let my_y = (y as isize) + offst_y;

                let i = my_y * (self.width as isize) +my_x;
                if i < (self.data.len() as isize) && i >= 0{
                    self.data[i as usize] = other.data[y*other.width+x];
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct VM{
    program: Vec<Instruction>,
    prog_counter: usize,
    memory: Vec<StackFrame>,
    canvas: Vec<Canvas>,
    saved_canvas: Vec<Canvas>
}

impl VM{
    pub fn new(program: Vec<Instruction>) -> Self{
        VM{
            program,
            prog_counter: 0,
            memory: vec![StackFrame::default()],
            canvas: vec![],
            saved_canvas: vec![]
        }
    }

    pub fn get_saved_canvas(&self) -> &[Canvas]{
        &self.saved_canvas
    }

    fn get_indx_of(&self, label: &str) -> usize{
        self.program.iter().enumerate().find(|(_, instr)| {
            &Instruction::Label(label.to_string()) == *instr
        }).expect(&format!("Label {} was not found", label)).0
    }

    pub fn run(&mut self, entry_point:&str){
        self.prog_counter = self.get_indx_of(entry_point);
        loop{
            if self.prog_counter >= self.program.len(){
                panic!("Unexpected end");
            }
            if! self.exec(){ break; }
            else{ self.prog_counter +=1; }
        }
        
    }

    fn exec(&mut self) -> bool{
        let instruction = self.program[self.prog_counter].clone();
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
                self.run(&f);

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
                    self.prog_counter = self.get_indx_of(&label);
                }

                true
            },

            Instruction::Jump(label) =>{
                self.prog_counter = self.get_indx_of(&label);
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
                    Param::Value(v) => v as isize,
                    Param::Register(reg) => self.memory[0].get(&reg) as isize
                };

                let y = match y {
                    Param::Value(v) => v as isize,
                    Param::Register(reg) => self.memory[0].get(&reg) as isize
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

                if x >= 0 && y >= 0{
                    let i = y as usize * self.canvas[0].width + x as usize;
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


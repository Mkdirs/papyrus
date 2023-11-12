use std::{collections::HashMap, path::{Path, PathBuf}};

use image::{ImageBuffer, RgbaImage, Rgba, imageops};

use crate::{ir::{Instruction, Param, Runtime, Script}, to_rgba, from_rgba};


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
    pub width: u32,
    pub height: u32,
    pub data: RgbaImage
    
}



impl Canvas{
    pub fn new(width: u32, height: u32) -> Self{
        let data = ImageBuffer::new(width, height);
        Canvas { width, height, data}
    }

    pub fn put(&mut self, x:u32, y:u32, pixel:u32){
        self.data.put_pixel(x, y, Rgba(to_rgba(pixel)));
    }

    pub fn get(&self, x:u32, y:u32) -> u32{
        let [r, g, b, a] = self.data.get_pixel(x, y).0;
        from_rgba(r, g, b, a)
    }

    pub fn merge(&mut self, offst_x:i32, offst_y:i32, source:Self){
        for y in offst_y..(offst_y+source.height as i32){
            for x in offst_x..(offst_x+source.width as i32){
                if (x >= 0 && x < self.width as i32) && (y >= 0 && y < self.height as i32){

                    let pixel = source.get((x-offst_x) as u32, (y-offst_y) as u32);
                    
                    let [_, _, _, a] = to_rgba(pixel);
                    if a != 0{
                        //TODO: Try blending colors
                        self.put(x as u32, y as u32, pixel);
                    }
                }
            }
        }
    }



    pub fn resize(&mut self, w:u32, h:u32){
        self.width = w;
        self.height = h;
        self.data = imageops::resize(&self.data, w, h, imageops::FilterType::Nearest);
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

            Instruction::Alpha(c, r) => {
                let color = match c {
                    Param::Value(v) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let [_, _, _, a] = to_rgba(color);
                self.memory[0].set(&r, a as u32);
                true
            },

            Instruction::Blue(c, r) => {
                let color = match c {
                    Param::Value(v) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let [_, b, _, _] = to_rgba(color);
                self.memory[0].set(&r, b as u32);
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

            Instruction::Ceil(x, r) => {
                let x = match x{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = x.ceil() as i32;

                self.memory[0].set(&r, result as u32);
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

            Instruction::Cos(x, r) => {
                let x = match x{
                    Param::Value(v) => f32::from_bits(v),

                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = x.cos();

                self.memory[0].set(&r, result.to_bits());
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
                let pixel = match c{
                    Param::Value(v) => v,

                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                for y in 0..self.canvas[0].height{
                    for x in 0..self.canvas[0].width{
                        self.canvas[0].put(x, y, pixel);
                    }
                }

                true
            },

            Instruction::Floor(x, r) => {
                let x = match x{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = x.floor() as i32;

                self.memory[0].set(&r, result as u32);
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

            Instruction::Green(c, r) => {
                let color = match c {
                    Param::Value(v) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let [_, g, _, _] = to_rgba(color);
                self.memory[0].set(&r, g as u32);
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

            Instruction::Height(r) => {
                self.memory[0].set(&r, self.canvas[0].height);
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

                self.canvas.insert(0, Canvas::new(left as u32, right as u32));

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
                    Param::Value(v) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let width = self.canvas[0].width;
                let height = self.canvas[0].height;

                if (0 <= x && x < width as i32) && (0 <= y && y < height as i32){
                    self.canvas[0].put(x as u32, y as u32, color);
                }


                true
            },

            Instruction::Red(c, r) => {
                let color = match c {
                    Param::Value(v) => v,
                    Param::Register(reg) => self.memory[0].get(&reg)
                };

                let [red, _, _, _] = to_rgba(color);
                self.memory[0].set(&r, red as u32);
                true
            },

            Instruction::Resize(w, h) => {
                let mut w = match w {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let mut h = match h {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                if w < 0 {w = 0;}
                if h < 0 {h = 0;}

                self.canvas[0].resize(w as u32, h as u32);
                true
            },

            Instruction::RGBA(r, g, b, a, reg) => {
                let mut r = match r{
                    Param::Value(v) => v as i32,
                    Param::Register(register) => self.memory[0].get(&register) as i32
                };

                let mut g = match g{
                    Param::Value(v) => v as i32,
                    Param::Register(register) => self.memory[0].get(&register) as i32
                };

                let mut b = match b{
                    Param::Value(v) => v as i32,
                    Param::Register(register) => self.memory[0].get(&register) as i32
                };

                let mut a = match a{
                    Param::Value(v) => v as i32,
                    Param::Register(register) => self.memory[0].get(&register) as i32
                };

                r = (r.clamp(0, 255) & 0xff) << 24;
                g = (g.clamp(0, 255) & 0xff) << 16;
                b = (b.clamp(0, 255) & 0xff) << 8;
                a = a.clamp(0, 255) & 0xff;

                let color = r | g | b | a;

                self.memory[0].set(&reg, color as u32);


                true
            }

            Instruction::Ret => {false},

            Instruction::Sample(x, y, r) => {
                let x = match x {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let y = match y {
                    Param::Value(v) => v as i32,
                    Param::Register(reg) => self.memory[0].get(&reg) as i32
                };

                let width = self.canvas[0].width;
                let height = self.canvas[0].height;

                if (0 <= x && x < width as i32) && (0 <= y && y < height as i32){
                    self.memory[0].set(&r, self.canvas[0].get(x as u32, y as u32));
                }

                true
            }

            Instruction::Save => {
                self.saved_canvas.push(self.canvas[0].clone());
                true
            },

            Instruction::Sin(x, r) => {
                let x = match x{
                    Param::Value(v) => f32::from_bits(v),
                    Param::Register(reg) => f32::from_bits(self.memory[0].get(&reg))
                };

                let result = x.sin();

                self.memory[0].set(&r, result.to_bits());
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
            },

            Instruction::Width(r) => {
                self.memory[0].set(&r, self.canvas[0].width);
                true
            }
        }
    }


}


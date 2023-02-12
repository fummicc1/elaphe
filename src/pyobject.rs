use std::{fs::File, io::Write};

use crate::bytecode::{self, ByteCode};

#[allow(dead_code)]
pub enum PyObject<'a> {
    Int(i32, bool),
    Float(f64, bool),
    String(&'a [u8], bool), // 文字列ではなくバイト列に利用
    Ascii(&'a str, bool),
    AsciiShort(&'a str, bool),
    Unicode(&'a str, bool),
    None(bool),
    True(bool),
    False(bool),
    SmallTuple {
        children: Vec<PyObject<'a>>,
        add_ref: bool,
    },
    Code {
        file_name: &'a str,
        code_name: &'a str,
        num_locals: u32,
        stack_size: u32,
        operation_list: Vec<ByteCode>,
        constant_list: Box<PyObject<'a>>,
        name_list: Box<PyObject<'a>>,
        local_list: Box<PyObject<'a>>,
        add_ref: bool,
    },
}

impl PyObject<'_> {
    pub fn new_numeric(value: &str, add_ref: bool) -> PyObject {
        if value.starts_with("0x") || value.starts_with("0X") {
            // 16進数の場合
            let value = i32::from_str_radix(&value[2..], 16).unwrap();
            PyObject::Int(value, add_ref)
        } else {
            match value.parse::<i32>() {
                Ok(value) => {
                    // 整数の場合
                    PyObject::Int(value, add_ref)
                }
                Err(_) => {
                    // 小数の場合
                    let value = value.parse::<f64>().unwrap();
                    PyObject::Float(value, add_ref)
                }
            }
        }
    }

    pub fn new_bytes(value: &[u8], add_ref: bool) -> PyObject {
        PyObject::String(value, add_ref)
    }

    pub fn new_string(value: &str, add_ref: bool) -> PyObject {
        if value.is_ascii() {
            if value.len() < 256 {
                PyObject::AsciiShort(value, add_ref)
            } else {
                PyObject::Ascii(value, add_ref)
            }
        } else {
            PyObject::Unicode(value, add_ref)
        }
    }

    pub fn new_boolean(value: &str, add_ref: bool) -> PyObject {
        match value {
            "true" => PyObject::True(add_ref),
            "false" => PyObject::False(add_ref),
            _ => panic!("Unknown Boolean Literal: {}", value),
        }
    }
}

impl PyObject<'_> {
    pub fn write(&self, file: &mut File) {
        let object_type = self.get_object_type();
        file.write(&[object_type]).unwrap();
        match &*self {
            PyObject::Int(v, _) => {
                file.write(&(v.to_le_bytes())).unwrap();
            }
            PyObject::Float(v, _) => {
                file.write(&(v.to_le_bytes())).unwrap();
            }
            PyObject::String(v, _) => {
                let str_len = v.len() as u32;
                file.write(&str_len.to_le_bytes()).unwrap();
                file.write(v).unwrap();
            }
            PyObject::Ascii(v, _) | PyObject::Unicode(v, _) => {
                let str_len = v.len() as u32;
                file.write(&str_len.to_le_bytes()).unwrap();
                file.write(v.as_bytes()).unwrap();
            }
            PyObject::AsciiShort(v, _) => {
                let str_len = v.len() as u8;
                file.write(&[str_len]).unwrap();
                file.write(v.as_bytes()).unwrap();
            }
            PyObject::SmallTuple { children, add_ref:_ } => {
                let tuple_len = children.len() as u8;
                file.write(&[tuple_len]).unwrap();
                for child in children {
                    child.write(file);
                }
            }
            PyObject::Code {
                file_name,
                code_name,
                num_locals,
                stack_size,
                operation_list,
                constant_list,
                name_list,
                local_list,
                add_ref:_,
            } => {
                file.write(&(0u32.to_le_bytes())).unwrap(); // ArgCount
                file.write(&(0u32.to_le_bytes())).unwrap(); // PosOnlyArgCount
                file.write(&(0u32.to_le_bytes())).unwrap(); // KwOnlyArgCount
                file.write(&(num_locals.to_le_bytes())).unwrap(); // NumLocals
                file.write(&(*stack_size as u32).to_le_bytes()).unwrap(); // StackSize
                file.write(&(64u32.to_le_bytes())).unwrap(); // Flags

                // コードをコンパイルして格納
                let codes = bytecode::compile_code(&operation_list);
                PyObject::new_bytes(&codes, false).write(file);

                // 定数一覧
                constant_list.write(file);

                // 名前一覧
                name_list.write(file);

                // ローカル変数一覧
                local_list.write(file);

                // 自由変数
                PyObject::SmallTuple {
                    children: vec![],
                    add_ref: false,
                }
                .write(file);

                // セル変数
                PyObject::SmallTuple {
                    children: vec![],
                    add_ref: false,
                }
                .write(file);

                // ファイル名
                PyObject::new_string(file_name, true).write(file);

                // 名前
                PyObject::new_string(code_name, true).write(file);

                // first line
                file.write(&(1u32).to_le_bytes()).unwrap();

                // line table
                // StackTraceに使われるが、仕様が不明なので0埋め
                PyObject::new_bytes(&(0u32).to_le_bytes(), true).write(file);
            }
            PyObject::None(_) | PyObject::True(_) | PyObject::False(_) => (),
        };
    }

    fn get_object_type(&self) -> u8 {
        match *self {
            PyObject::Int(_, r) => 0x69 | ((r as u8) << 7),
            PyObject::Float(_, r) => 0x67 | ((r as u8) << 7),
            PyObject::String(_, r) => 0x73 | ((r as u8) << 7),
            PyObject::Ascii(_, r) => 0x61 | ((r as u8) << 7),
            PyObject::AsciiShort(_, r) => 0x7A | ((r as u8) << 7),
            PyObject::Unicode(_, r) => 0x75 | ((r as u8) << 7),
            PyObject::None(r) => 0x4E | ((r as u8) << 7),
            PyObject::True(r) => 0x54 | ((r as u8) << 7),
            PyObject::False(r) => 0x46 | ((r as u8) << 7),
            PyObject::SmallTuple {
                children: _,
                add_ref,
            } => 0x29 | ((add_ref as u8) << 7),
            PyObject::Code {
                file_name: _,
                code_name: _,
                num_locals: _,
                stack_size: _,
                operation_list: _,
                constant_list: _,
                name_list: _,
                local_list: _,
                add_ref,
            } => 0x63 | ((add_ref as u8) << 7),
        }
    }
}

use std::{collections::HashMap, convert::TryInto};

use crate::parser::{ASTBinary, ASTBinaryType, ASTData, ASTDataType, ASTExpression, ASTFunc, ASTIdentifier, ASTNode, ASTNodeType, ASTNumeral, ASTRoot, ASTVar, Parser};

pub enum StorableValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

struct EnvVar {
    address: u16,
    ty: ASTData,
}

impl EnvVar {
    pub fn new(address: u16, ty: ASTData) -> Self {
        Self { address, ty }
    }
}

struct EnvFunc {

}

impl EnvFunc {
    pub fn new() -> Self {
        Self {}
    }
}

enum EnvDecl {
    Var(EnvVar),
    Func(EnvFunc),
}

struct StackFrame {
    base: u16,
    return_addr: u16,
}

struct CallStack {
    frames: Vec<StackFrame>,
}

struct Memory {
    data: [u8; 65536],
    // allocations: HashMap<u16, (u16, ASTDataType)>,
}

impl Memory {
    pub fn new() -> Self {
        Self { data: [0; 65536] }
    }

    fn write_bytes<const N: usize>(&mut self, addr: u16, bytes: [u8; N], limit: Option<usize>) {
        let addr = addr as usize;
        let n = N.min(limit.unwrap_or(usize::MAX));
        self.data[addr..addr+n].copy_from_slice(&bytes[0..n]);
    }

    fn write_full(&mut self, addr: u16, value: &StorableValue, limit: Option<u16>) {
        let limit = limit.map(|l| l as usize);

        match value {
            StorableValue::I8(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::I16(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::I32(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::I64(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::F32(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::F64(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
        }
    }

    pub fn write(&mut self, addr: u16, value: &StorableValue) {
        self.write_full(addr, value, None);
    }

    pub fn write_truncated(&mut self, addr: u16, value: &StorableValue, size: u16) {
        self.write_full(addr, value, Some(size));
    }

    fn get_bytes<const N: usize>(&self, addr: u16) -> [u8; N] {
        let addr = addr as usize;
        self.data[addr..addr+N].try_into().unwrap()
    }

    pub fn read(&self, addr: u16, ty: &ASTDataType) -> StorableValue {
        match ty {
            ASTDataType::Char => StorableValue::I8(i8::from_le_bytes(self.get_bytes(addr))),
            ASTDataType::Short => StorableValue::I16(i16::from_le_bytes(self.get_bytes(addr))),
            ASTDataType::Int => StorableValue::I32(i32::from_le_bytes(self.get_bytes(addr))),
            ASTDataType::Long | ASTDataType::LongLong => StorableValue::I64(i64::from_le_bytes(self.get_bytes(addr))),
            ASTDataType::Float => StorableValue::F32(f32::from_le_bytes(self.get_bytes(addr))),
            ASTDataType::Double => StorableValue::F64(f64::from_le_bytes(self.get_bytes(addr))),
            _ => StorableValue::I8(0), // TODO: implement others later
        }
    }

    fn extract_size<const N: usize>(&self, bytes: [u8; N]) -> u16 {
        N as u16
    }

    pub fn get_size(&self, ty: &ASTDataType) -> u16 {
        let value = self.read(0, ty);

        match value {
            StorableValue::I8(value) => self.extract_size(value.to_le_bytes()),
            StorableValue::I16(value) => self.extract_size(value.to_le_bytes()),
            StorableValue::I32(value) => self.extract_size(value.to_le_bytes()),
            StorableValue::I64(value) => self.extract_size(value.to_le_bytes()),
            StorableValue::F32(value) => self.extract_size(value.to_le_bytes()),
            StorableValue::F64(value) => self.extract_size(value.to_le_bytes()),
        }
    }
}

pub struct Environment {
    decls: HashMap<String, EnvDecl>,
    ptr: u16,
}

impl Environment {
    fn new() -> Self {
        Environment { decls: HashMap::new(), ptr: 0 }
    }

    fn ptr(&self) -> u16 {
        self.ptr
    }

    fn add_var(&mut self, name: String, var: EnvVar, size: u16) {
        self.decls.entry(name).insert_entry(EnvDecl::Var(var));
        self.ptr += size;
    }

    fn get_var(&self, name: &str) -> Option<&EnvVar> {
        self.decls.get(name).and_then(|decl| match decl {
            EnvDecl::Var(var) => Some(var),
            _ => None,
        })
    }

    fn add_func(&mut self, name: String, func: EnvFunc) {
        self.decls.entry(name).insert_entry(EnvDecl::Func(func));
    }
}

pub struct Exec {
    pub memory: [u8; 65536],
}

pub struct Interpreter {
    root: ASTNode,
    memory: Memory,
    env: Environment,
}

macro_rules! apply_binary_op {
    ($left:expr, $right:expr, $op:tt) => {
        match ($left, $right) {
            (StorableValue::I8(l), StorableValue::I8(r)) => StorableValue::I8(l $op r),
            (StorableValue::I16(l), StorableValue::I16(r)) => StorableValue::I16(l $op r),
            (StorableValue::I32(l), StorableValue::I32(r)) => StorableValue::I32(l $op r),
            (StorableValue::I64(l), StorableValue::I64(r)) => StorableValue::I64(l $op r),
            (StorableValue::F32(l), StorableValue::F32(r)) => StorableValue::F32(l $op r),
            (StorableValue::F64(l), StorableValue::F64(r)) => StorableValue::F64(l $op r),
            _ => unreachable!(),
        }
    }
}

impl Interpreter {
    pub fn new(s: &str) -> Self {
        let mut parser = Parser::new(s);
        let root = parser.parse();

        Self { root, memory: Memory::new(), env: Environment::new() }
    }

    pub fn new_with_node(root: ASTNode) -> Self {
        Self { root, memory: Memory::new(), env: Environment::new() }
    }

    fn get_digit(&self, c: char) -> u64 {
        match c {
            '0'..='9' => (c as u8 - b'0') as u64,
            _ => unreachable!(),
        }
    }

    fn exec_identifier(&self, identifier: ASTIdentifier) -> StorableValue {
        let var = self.env.get_var(&identifier.literal).unwrap();
        self.memory.read(var.address, &var.ty.ty)
    }

    fn exec_numeral(&self, numeral: ASTNumeral) -> StorableValue {
        let literal = numeral.literal;
        let mut acc: u64 = 0;

        for c in literal.chars() {
            let d = self.get_digit(c);
            if acc > u64::MAX / 10 {
                panic!("Literal too large");
            }
            acc *= 10;

            if acc > u64::MAX - d {
                panic!("Literal too large");
            }
            acc += d;
        }

        if acc <= i32::MAX as u64 {
            StorableValue::I32(acc as i32)
        } else if acc <= i64::MAX as u64 {
            StorableValue::I64(acc as i64)
        } else {
            StorableValue::I8(0) // temp
        }
    }

    fn rank(&self, value: &StorableValue) -> u8 {
        match value {
            StorableValue::I8(_) => 1,
            StorableValue::I16(_) => 2,
            StorableValue::I32(_) => 3,
            StorableValue::I64(_) => 4,
            StorableValue::F32(_) => 5,
            StorableValue::F64(_) => 6,
        }
    }

    fn convert_to_rank(&self, value: StorableValue, rank: u8) -> StorableValue {
        match (value, rank) {
            (StorableValue::I8(v), 2) => StorableValue::I16(v as i16),
            (StorableValue::I8(v), 3) => StorableValue::I32(v as i32),
            (StorableValue::I8(v), 4) => StorableValue::I64(v as i64),
            (StorableValue::I8(v), 5) => StorableValue::F32(v as f32),
            (StorableValue::I8(v), 6) => StorableValue::F64(v as f64),
            (StorableValue::I16(v), 3) => StorableValue::I32(v as i32),
            (StorableValue::I16(v), 4) => StorableValue::I64(v as i64),
            (StorableValue::I16(v), 5) => StorableValue::F32(v as f32),
            (StorableValue::I16(v), 6) => StorableValue::F64(v as f64),
            (StorableValue::I32(v), 4) => StorableValue::I64(v as i64),
            (StorableValue::I32(v), 5) => StorableValue::F32(v as f32),
            (StorableValue::I32(v), 6) => StorableValue::F64(v as f64),
            (StorableValue::I64(v), 5) => StorableValue::F32(v as f32),
            (StorableValue::I64(v), 6) => StorableValue::F64(v as f64),
            (StorableValue::F32(v), 6) => StorableValue::F64(v as f64),
            _ => unreachable!(),
        }
    }

    fn convert(&self, left: StorableValue, right: StorableValue) -> (StorableValue, StorableValue) {
        let (lrank, rrank) = (self.rank(&left), self.rank(&right));

        if lrank == rrank {
            (left, right)
        } else if lrank < rrank {
            (self.convert_to_rank(left, rrank), right)
        } else {
            (left, self.convert_to_rank(right, lrank))
        }
    }

    fn exec_binary(&self, binary: ASTBinary) -> StorableValue {
        let left = self.exec_expr(*binary.left);
        let right = self.exec_expr(*binary.right);

        let (left, right) = self.convert(left, right);

        match binary.ty {
            ASTBinaryType::Add => apply_binary_op!(left, right, +),
            ASTBinaryType::Sub => apply_binary_op!(left, right, -),
            ASTBinaryType::Mult => apply_binary_op!(left, right, *),
            ASTBinaryType::Div => apply_binary_op!(left, right, /),
            _ => todo!(),
        }
    }

    fn exec_expr(&self, expr: ASTExpression) -> StorableValue {
        match expr {
            ASTExpression::Identifier(id) => self.exec_identifier(id),
            ASTExpression::Numeral(numeral) => self.exec_numeral(numeral),
            ASTExpression::Binary(binary) => self.exec_binary(binary),
            _ => todo!(),
        }
    }

    fn exec_root(&mut self, root: ASTRoot) {
        for node in root.statements {
            self.exec_node(node);
        }
    }

    fn exec_func(&mut self, func: ASTFunc) {
        todo!()
    }

    fn exec_var(&mut self, var: ASTVar) {
        let name = var.name.literal;
        let ty = var.ty;
        let addr = self.env.ptr();
        let val = var.initializer.map(|expr| self.exec_expr(expr));

        let size = self.memory.get_size(&ty.ty);
        self.env.add_var(name, EnvVar::new(addr, ty.clone()), size);
        val.map(|val| self.memory.write_truncated(addr, &val, size));
    }

    fn exec_node(&mut self, node: ASTNode) {
        match node.ty {
            ASTNodeType::Root(root) => self.exec_root(root),
            ASTNodeType::Func(func) => self.exec_func(func),
            ASTNodeType::Var(var) => self.exec_var(var),
            ASTNodeType::EOF => (),
        }
    }

    pub fn execute(&mut self) -> Option<Exec> {
        self.exec_node(self.root.clone());
        Some(Exec { memory: self.memory.data.clone() })
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::*;

    #[test]
    fn simple_assignment() {
        let mut i = Interpreter::new("int a = 1;");
        let exec = i.execute().unwrap();
        println!("{:?}", &exec.memory[0..50]);
        assert_eq!(exec.memory[0], 1);
        assert_eq!(exec.memory[1], 0);
    }

    #[test]
    fn simple_expressions() {
        let mut i = Interpreter::new("int a = 1; int b = a + 1; int c = a + b * 2;");
        let exec = i.execute().unwrap();
        println!("{:?}", &exec.memory[0..50]);
        assert_eq!(exec.memory[0], 1);
        assert_eq!(exec.memory[1], 0);
        assert_eq!(exec.memory[4], 2);
        assert_eq!(exec.memory[5], 0);
        assert_eq!(exec.memory[8], 5);
        assert_eq!(exec.memory[9], 0);
    }

    #[test]
    fn simple_conversions() {
        let mut i = Interpreter::new("short int a = 3; long int b = a + 4; int c = a + b * 4 / 2; int d = b;");
        let exec = i.execute().unwrap();
        println!("{:?}", &exec.memory[0..50]);
        assert_eq!(exec.memory[0], 3);
        assert_eq!(exec.memory[1], 0);
        assert_eq!(exec.memory[2], 7);
        assert_eq!(exec.memory[3], 0);
        assert_eq!(exec.memory[10], 17);
        assert_eq!(exec.memory[11], 0);
        assert_eq!(exec.memory[14], 7);
        assert_eq!(exec.memory[15], 0);
    }

    #[test]
    fn conversion_test() {
        let mut i = Interpreter::new("short int a = 752235;");
        let exec = i.execute().unwrap();
        println!("{:?}", &exec.memory[0..50]);
        assert_eq!(exec.memory[2], 0);
        assert_eq!(exec.memory[3], 0);
    }

}

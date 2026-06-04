use std::{collections::HashMap, convert::TryInto};

use crate::parser::*;

pub enum StorableValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
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
    fn new(address: u16, ty: ASTData) -> Self {
        Self { address, ty }
    }
}

struct EnvFunc {
    ret_ty: ASTData,
    params: Vec<ASTFuncParam>,
    body: Vec<ASTNode>,
}

impl EnvFunc {
    fn new(ret_ty: ASTData, params: Vec<ASTFuncParam>, body: Vec<ASTNode>) -> Self {
        Self { ret_ty, params, body }
    }
}

enum EnvDecl {
    Var(EnvVar),
    Func(EnvFunc),
}

struct StackFrame {
    base: u16,
    return_addr: u16,
    env: Environment,
}

struct CallStack {
    frames: Vec<StackFrame>,
}

impl CallStack {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }
}

struct Memory {
    data: [u8; 65536],
    stack: CallStack,
}

impl Memory {
    pub fn new() -> Self {
        Self { data: [0; 65536], stack: CallStack::new() }
    }

    fn write_bytes<const N: usize>(&mut self, addr: u16, bytes: [u8; N], limit: Option<usize>) {
        let addr = addr as usize;
        let n = N.min(limit.unwrap_or(usize::MAX));
        self.data[addr..addr+n].copy_from_slice(&bytes[0..n]);
    }

    fn write(&mut self, addr: u16, value: &StorableValue, limit: Option<u16>) {
        let limit = limit.map(|l| l as usize);

        match value {
            StorableValue::U8(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::U16(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::U32(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::U64(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::I8(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::I16(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::I32(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::I64(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::F32(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
            StorableValue::F64(value) => self.write_bytes(addr, value.to_le_bytes(), limit),
        }
    }

    pub fn write_truncated(&mut self, addr: u16, value: &StorableValue, size: u16) {
        self.write(addr, value, Some(size));
    }

    fn get_bytes<const N: usize>(&self, addr: u16) -> [u8; N] {
        let addr = addr as usize;
        self.data[addr..addr+N].try_into().unwrap()
    }

    pub fn read(&self, addr: u16, ty: &ASTData) -> StorableValue {
        match (ty.ty, ty.signed) {
            (ASTDataType::Char, false) => StorableValue::U8(u8::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Short, false) => StorableValue::U16(u16::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Int, false) => StorableValue::U32(u32::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Long | ASTDataType::LongLong, false) => StorableValue::U64(u64::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Char, true) => StorableValue::I8(i8::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Short, true) => StorableValue::I16(i16::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Int, true) => StorableValue::I32(i32::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Long | ASTDataType::LongLong, true) => StorableValue::I64(i64::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Float, _) => StorableValue::F32(f32::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Double, _) => StorableValue::F64(f64::from_le_bytes(self.get_bytes(addr))),
            (ASTDataType::Void, _) => unreachable!(),
        }
    }

    fn extract_size<const N: usize>(&self, _: [u8; N]) -> u16 {
        N as u16
    }

    pub fn get_size(&self, ty: &ASTData) -> u16 {
        let value = self.read(0, ty);

        match value {
            StorableValue::U8(value) => self.extract_size(value.to_le_bytes()),
            StorableValue::U16(value) => self.extract_size(value.to_le_bytes()),
            StorableValue::U32(value) => self.extract_size(value.to_le_bytes()),
            StorableValue::U64(value) => self.extract_size(value.to_le_bytes()),
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

    fn get_func(&self, name: &str) -> Option<&EnvFunc> {
        self.decls.get(name).and_then(|decl| match decl {
            EnvDecl::Func(func) => Some(func),
            _ => None,
        })
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
            (StorableValue::U8(l), StorableValue::U8(r)) => StorableValue::U8(l $op r),
            (StorableValue::I16(l), StorableValue::I16(r)) => StorableValue::I16(l $op r),
            (StorableValue::U16(l), StorableValue::U16(r)) => StorableValue::U16(l $op r),
            (StorableValue::I32(l), StorableValue::I32(r)) => StorableValue::I32(l $op r),
            (StorableValue::U32(l), StorableValue::U32(r)) => StorableValue::U32(l $op r),
            (StorableValue::I64(l), StorableValue::I64(r)) => StorableValue::I64(l $op r),
            (StorableValue::U64(l), StorableValue::U64(r)) => StorableValue::U64(l $op r),
            (StorableValue::F32(l), StorableValue::F32(r)) => StorableValue::F32(l $op r),
            (StorableValue::F64(l), StorableValue::F64(r)) => StorableValue::F64(l $op r),
            _ => unreachable!(),
        }
    }
}

macro_rules! apply_logic_binary_op {
    ($left:expr, $right:expr, $op:tt) => {
        match ($left, $right) {
            (StorableValue::I8(l), StorableValue::I8(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
            (StorableValue::U8(l), StorableValue::U8(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
            (StorableValue::I16(l), StorableValue::I16(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
            (StorableValue::U16(l), StorableValue::U16(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
            (StorableValue::I32(l), StorableValue::I32(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
            (StorableValue::U32(l), StorableValue::U32(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
            (StorableValue::I64(l), StorableValue::I64(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
            (StorableValue::U64(l), StorableValue::U64(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
            (StorableValue::F32(l), StorableValue::F32(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
            (StorableValue::F64(l), StorableValue::F64(r)) => StorableValue::I32(if l $op r { 1 } else { 0 }),
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
        self.memory.read(var.address, &var.ty)
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
            StorableValue::U8(_) => 2,
            StorableValue::I16(_) => 3,
            StorableValue::U16(_) => 4,
            StorableValue::I32(_) => 5,
            StorableValue::U32(_) => 6,
            StorableValue::I64(_) => 7,
            StorableValue::U64(_) => 8,
            StorableValue::F32(_) => 9,
            StorableValue::F64(_) => 10,
        }
    }

    fn convert_to_rank(&self, value: StorableValue, rank: u8) -> StorableValue {
        match (value, rank) {
            (StorableValue::I8(v), 2) => StorableValue::U8(v as u8),
            (StorableValue::I8(v), 3) => StorableValue::I16(v as i16),
            (StorableValue::I8(v), 4) => StorableValue::U16(v as u16),
            (StorableValue::I8(v), 5) => StorableValue::I32(v as i32),
            (StorableValue::I8(v), 6) => StorableValue::U32(v as u32),
            (StorableValue::I8(v), 7) => StorableValue::I64(v as i64),
            (StorableValue::I8(v), 8) => StorableValue::U64(v as u64),
            (StorableValue::I8(v), 9) => StorableValue::F32(v as f32),
            (StorableValue::I8(v), 10) => StorableValue::F64(v as f64),
            (StorableValue::U8(v), 3) => StorableValue::I16(v as i16),
            (StorableValue::U8(v), 4) => StorableValue::U16(v as u16),
            (StorableValue::U8(v), 5) => StorableValue::I32(v as i32),
            (StorableValue::U8(v), 6) => StorableValue::U32(v as u32),
            (StorableValue::U8(v), 7) => StorableValue::I64(v as i64),
            (StorableValue::U8(v), 8) => StorableValue::U64(v as u64),
            (StorableValue::U8(v), 9) => StorableValue::F32(v as f32),
            (StorableValue::U8(v), 10) => StorableValue::F64(v as f64),
            (StorableValue::I16(v), 4) => StorableValue::U16(v as u16),
            (StorableValue::I16(v), 5) => StorableValue::I32(v as i32),
            (StorableValue::I16(v), 6) => StorableValue::U32(v as u32),
            (StorableValue::I16(v), 7) => StorableValue::I64(v as i64),
            (StorableValue::I16(v), 8) => StorableValue::U64(v as u64),
            (StorableValue::I16(v), 9) => StorableValue::F32(v as f32),
            (StorableValue::I16(v), 10) => StorableValue::F64(v as f64),
            (StorableValue::U16(v), 5) => StorableValue::I32(v as i32),
            (StorableValue::U16(v), 6) => StorableValue::U32(v as u32),
            (StorableValue::U16(v), 7) => StorableValue::I64(v as i64),
            (StorableValue::U16(v), 8) => StorableValue::U64(v as u64),
            (StorableValue::U16(v), 9) => StorableValue::F32(v as f32),
            (StorableValue::U16(v), 10) => StorableValue::F64(v as f64),
            (StorableValue::I32(v), 6) => StorableValue::U32(v as u32),
            (StorableValue::I32(v), 7) => StorableValue::I64(v as i64),
            (StorableValue::I32(v), 8) => StorableValue::U64(v as u64),
            (StorableValue::I32(v), 9) => StorableValue::F32(v as f32),
            (StorableValue::I32(v), 10) => StorableValue::F64(v as f64),
            (StorableValue::U32(v), 7) => StorableValue::I64(v as i64),
            (StorableValue::U32(v), 8) => StorableValue::U64(v as u64),
            (StorableValue::U32(v), 9) => StorableValue::F32(v as f32),
            (StorableValue::U32(v), 10) => StorableValue::F64(v as f64),
            (StorableValue::I64(v), 8) => StorableValue::U64(v as u64),
            (StorableValue::I64(v), 9) => StorableValue::F32(v as f32),
            (StorableValue::I64(v), 10) => StorableValue::F64(v as f64),
            (StorableValue::U64(v), 9) => StorableValue::F32(v as f32),
            (StorableValue::U64(v), 10) => StorableValue::F64(v as f64),
            (StorableValue::F32(v), 10) => StorableValue::F64(v as f64),
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

    fn exec_binary_arith_logic(&mut self, binary: ASTBinary) -> StorableValue {
        let left = self.exec_expr(*binary.left);
        let right = self.exec_expr(*binary.right);

        let (left, right) = self.convert(left, right);

        match binary.ty {
            ASTBinaryType::Add => apply_binary_op!(left, right, +),
            ASTBinaryType::Sub => apply_binary_op!(left, right, -),
            ASTBinaryType::Mult => apply_binary_op!(left, right, *),
            ASTBinaryType::Div => apply_binary_op!(left, right, /),
            ASTBinaryType::LessThan => apply_logic_binary_op!(left, right, <),
            ASTBinaryType::BiggerThan => apply_logic_binary_op!(left, right, >),
            _ => unreachable!(),
        }
    }

    fn extract_lvalue(&self, expr: ASTExpression) -> ASTIdentifier {
        match expr {
            ASTExpression::Identifier(ident) => ident,
            _ => panic!("Expected an lvalue"),
        }
    }

    fn exec_assignment(&mut self, binary: ASTBinary) -> StorableValue {
        let ident = self.extract_lvalue(*binary.left);
        let res = self.exec_expr(*binary.right);
        
        let var = self.env.get_var(&ident.literal).unwrap();
        self.memory.write_truncated(var.address, &res, self.memory.get_size(&var.ty));

        res
    }

    fn exec_binary(&mut self, binary: ASTBinary) -> StorableValue {
        match binary.ty {
            ASTBinaryType::Assignment => self.exec_assignment(binary),
            _ => self.exec_binary_arith_logic(binary),
        }
    }

    fn exec_invocation(&mut self, invocation: ASTInvocation) -> Option<StorableValue> {
        let a = self.exec_expr(*invocation.left);
        None // TODO
    }

    fn exec_expr(&mut self, expr: ASTExpression) -> StorableValue {
        match expr {
            ASTExpression::Identifier(id) => self.exec_identifier(id),
            ASTExpression::Numeral(numeral) => self.exec_numeral(numeral),
            ASTExpression::Binary(binary) => self.exec_binary(binary),
            _ => todo!(),
        }
    }

    fn exec_expr_statement(&mut self, expr: ASTExpression) {
        _ = self.exec_expr(expr);
    }

    fn exec_root(&mut self, root: ASTRoot) {
        for node in root.statements {
            self.exec_node(node);
        }
    }

    fn exec_func(&mut self, func: ASTFunc) {
        let name = func.name.literal;
        let ty = func.ty;
        let params = func.params;
        let body = func.body;

        self.env.add_func(name, EnvFunc::new(ty, params, body));
    }

    fn exec_var(&mut self, var: ASTVar) {
        let name = var.name.literal;
        let ty = var.ty;
        let addr = self.env.ptr();
        let val = var.initializer.map(|expr| self.exec_expr(expr));

        let size = self.memory.get_size(&ty);
        self.env.add_var(name, EnvVar::new(addr, ty.clone()), size);
        val.map(|val| self.memory.write_truncated(addr, &val, size));
    }

    fn is_truthy(&self, value: StorableValue) -> bool {
        match value {
            StorableValue::I32(v) => v != 0,
            _ => unreachable!(),
        }
    }

    fn exec_while(&mut self, wh: ASTWhile) {
        loop {
            let cond = self.exec_expr(wh.cond.clone());
            if !self.is_truthy(cond) {
                break;
            }

            for node in wh.body.clone() {
                self.exec_node(node);
            }
        }
    }

    fn exec_node(&mut self, node: ASTNode) {
        match node.ty {
            ASTNodeType::Root(root) => self.exec_root(root),

            ASTNodeType::Func(func) => self.exec_func(func),
            ASTNodeType::Var(var) => self.exec_var(var),
            ASTNodeType::Expression(expr) => self.exec_expr_statement(expr),

            ASTNodeType::While(wh) => self.exec_while(wh),

            ASTNodeType::EOF => (),
        };
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

    #[test]
    fn simple_unsigned() {
        let mut i = Interpreter::new("short int a = 3; unsigned long int b = a + 4; int c = a + b * 4 / 2; int d = b;");
        let exec = i.execute().unwrap();
        println!("{:?}", &exec.memory[0..50]);
        assert_eq!(exec.memory[2], 7);
        assert_eq!(exec.memory[3], 0);
    }

    #[test]
    fn simple_while() {
        let mut i = Interpreter::new("int a = 1; while (a < 5) a = a + 1;");
        let exec = i.execute().unwrap();
        println!("{:?}", &exec.memory[0..50]);
        assert_eq!(exec.memory[0], 5);
        assert_eq!(exec.memory[1], 0);
    }

    #[test]
    fn simple_func() {
        let mut i = Interpreter::new("int a = 1; void b() { a = a + 1; } b(); b();");
        let exec = i.execute().unwrap();
        println!("{:?}", &exec.memory[0..50]);
        panic!("Inspection test");
    }

}

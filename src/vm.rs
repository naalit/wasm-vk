use crate::*;

#[derive(Clone, Copy, PartialEq)]
enum Value {
    I32(u32),
    I64(u64),
    F32(f32),
    F64(f64),
}
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::I32(x) => write!(f, "{}u32", x),
            Value::I64(x) => write!(f, "{}u64", x),
            Value::F32(x) => write!(f, "{}f32", x),
            Value::F64(x) => write!(f, "{}f64", x),
        }
    }
}

use std::sync::{Arc, RwLock};

impl TypedDefault for Value {
    fn default(ty: WasmTy) -> Self {
        use WasmTy::*;
        match ty {
            I32 => Value::I32(0),
            I64 => Value::I64(0),
            F32 => Value::F32(0.0),
            F64 => Value::F64(0.0),
        }
    }
}

struct Interpreter {
    memory: Arc<RwLock<Vec<Value>>>,
}
impl Visitor for Interpreter {
    type Output = Value;
    fn visit(&mut self, op: AOp<Value>) -> Value {
        use AOp::*;
        match op {
            Mul(a, b) => match (*a, *b) {
                (Value::I32(a), Value::I32(b)) => Value::I32(a * b),
                _ => panic!("aah2"),
            },
            Add(a, b) => match (*a, *b) {
                (Value::I32(a), Value::I32(b)) => Value::I32(a + b),
                _ => panic!("aah3"),
            },
            I32Const(u) => Value::I32(u),
            Store(ptr, val) => {
                if let Value::I32(ptr) = *ptr {
                    self.memory.write().unwrap()[ptr as usize / 4] = *val;
                    Value::I32(0)
                } else {
                    panic!("{:?} is not a pointer", ptr);
                }
            }
            Load(ptr) => {
                if let Value::I32(ptr) = *ptr {
                    self.memory.read().unwrap()[ptr as usize / 4]
                } else {
                    panic!("{:?} is not a pointer", ptr);
                }
            }
        }
    }
}

pub fn interpret(buffer: &[u32], module: &wasm::Module) -> Vec<u32> {
    let mem = Arc::new(RwLock::new(buffer.iter().map(|x| Value::I32(*x)).collect()));
    for i in 0..buffer.len() {
        let mut am = AM::from_ref(module);
        am.visit(
            0,
            vec![TVal {
                val: Value::I32(i as u32),
                ty: WasmTy::I32,
            }],
            &mut Interpreter {
                memory: mem.clone(),
            },
        )
        .unwrap();
    }
    let m = mem.read().unwrap();
    m.iter()
        .map(|x| {
            if let Value::I32(x) = x {
                *x
            } else {
                panic!("Got {:?} in buffer", x)
            }
        })
        .collect()
}

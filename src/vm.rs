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
    skipping: bool,
    memory: Arc<RwLock<Vec<Value>>>,
    locals: Vec<Value>,
    idx: u32,
}
impl Visitor for Interpreter {
    type Output = Value;
    /// - None means it's not an If or we skipped the If completely, so it doesn't matter for skipping purposes.
    /// - Some(true) means we didn't skip it, and it was an If(true) so we still aren't skipping.
    /// - Some(false) means we didn't skip it, and it was an If(false) so we started skipping. When it ends we'll stop skipping.
    type BlockData = Option<bool>;

    fn br_break(&mut self, block: &Option<bool>) {
        // If this is Some, we didn't skip it. So:
        // - If we started skipping at this block, we're done skipping
        // - If we started skipping after, we ended that block too, so we're done skipping
        // - Otherwise, we were never skipping in the first place
        if block.is_some() {
            self.skipping = false;
        }
    }

    fn start_block(&mut self, op: BlockOp<Value>) -> Option<bool> {
        if self.skipping {
            return None;
        }
        match op {
            BlockOp::If(v) => match *v {
                Value::I32(0) => {
                    self.skipping = true;
                    Some(false)
                }
                Value::I32(_) => Some(true),
                _ => panic!("If only works on i32s, not {:?}!", v),
            },
        }
    }

    fn else_block(&mut self, data: Option<bool>) -> Option<bool> {
        // Don't do anything if it's None, because we're still skipping right over it
        let b = data?;
        self.skipping = b;
        Some(!b)
    }

    fn end_block(&mut self, data: Option<bool>) {
        // If this was a False branch that we didn't skip over completely, we can stop skipping now
        if let Some(false) = data {
            self.skipping = false;
        }
    }

    fn add_local(&mut self, ty: WasmTy, val: Option<Self::Output>) {
        self.locals.push(val.unwrap_or_else(|| Value::default(ty)));
    }

    fn visit(&mut self, op: AOp<Value>) -> Value {
        if self.skipping {
            return Value::I32(0);
        }
        use AOp::*;
        match op {
            GetGlobalImport(module, field) => {
                if module != "spv" {
                    panic!("Unknown namespace {}", module);
                }
                match &*field {
                    "id" => Value::I32(self.idx),
                    _ => panic!("Unknown global {}", field),
                }
            }
            GetLocal(l) => self.locals[l as usize],
            SetLocal(l, v) => {
                self.locals[l as usize] = *v;
                Value::I32(0)
            }
            Eq(a, b) => match (*a, *b) {
                (Value::I32(a), Value::I32(b)) => Value::I32(if a == b { 1 } else { 0 }),
                _ => panic!("aah2"),
            },
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
    let main = module
        .main()
        .expect("No 'main' exported, or it's not a function!");

    let mem = Arc::new(RwLock::new(buffer.iter().map(|x| Value::I32(*x)).collect()));
    for i in 0..buffer.len() {
        let mut am = AM::from_ref(module);
        am.visit(
            main,
            Vec::new(),
            // vec![TVal {
            //     val: Value::I32(i as u32),
            //     ty: WasmTy::I32,
            // }],
            &mut Interpreter {
                skipping: false,
                locals: Vec::new(),
                memory: mem.clone(),
                idx: i as u32,
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

use parity_wasm as wasm;
pub use wasm::elements::Instruction as Op;
pub use wasm::elements::ValueType as WasmTy;

#[derive(Debug)]
pub enum WasmError {
    SerializationError(parity_wasm::SerializationError),
    FunctionNotFound,
}
impl From<parity_wasm::SerializationError> for WasmError {
    fn from(p: parity_wasm::SerializationError) -> WasmError {
        WasmError::SerializationError(p)
    }
}

pub trait TypedDefault {
    fn default(t: WasmTy) -> Self;
}

use std::ops::Deref;

/// Some value with a value type attached
#[derive(Clone, Debug)]
pub struct TVal<V: TypedDefault> {
    pub ty: WasmTy,
    pub val: V,
}
impl<V: TypedDefault> TypedDefault for TVal<V> {
    fn default(ty: WasmTy) -> Self {
        TVal {
            ty,
            val: TypedDefault::default(ty),
        }
    }
}
impl<V: TypedDefault> Deref for TVal<V> {
    type Target = V;
    fn deref(&self) -> &V {
        &self.val
    }
}

/// An instruction, with attached operands of some type
pub enum AOp<T: TypedDefault> {
    Mul(TVal<T>, TVal<T>),
    Add(TVal<T>, TVal<T>),
    /// `Store(ptr, val)`: store `val` at location `ptr`
    ///
    /// `ptr` is a byte offset from the start of linear memory
    /// This ignores the specified offset and alignment - TODO: fix that
    Store(TVal<T>, TVal<T>),
    /// `Load(ptr)`: push the value at location `ptr` onto the top of the stack
    ///
    /// `ptr` is a byte offset from the start of linear memory
    /// This ignores the specified offset and alignment - TODO: fix that
    Load(TVal<T>),
    I32Const(u32),
}

/// Essentially a catamorphism
pub trait Visitor {
    type Output: TypedDefault + Clone;
    fn visit(&mut self, op: AOp<Self::Output>) -> Self::Output;
}

struct AFrame<T: TypedDefault> {
    locals: Vec<TVal<T>>,
}

enum BorrowedOrOwned<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}
impl<'a, T> Deref for BorrowedOrOwned<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self {
            BorrowedOrOwned::Borrowed(x) => x,
            BorrowedOrOwned::Owned(x) => &x,
        }
    }
}

/// An Abstract Machine to visit a module's instructions
pub struct AM<'module, T: TypedDefault> {
    module: BorrowedOrOwned<'module, parity_wasm::elements::Module>,
    stack: Vec<TVal<T>>,
    call_stack: Vec<AFrame<T>>,
}
impl<'module, T: TypedDefault + Clone> AM<'module, T> {
    /// Construct an `AM` from a slice of bytes in the WebAssembly binary format
    /// Note: this function does no validation
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WasmError> {
        Ok(AM {
            module: BorrowedOrOwned::Owned(wasm::deserialize_buffer(bytes)?),
            stack: Vec::new(),
            call_stack: Vec::new(),
        })
    }

    /// Construct an `AM` from an owned parity_wasm `Module`
    /// Note: this function does no validation
    pub fn from_move(module: wasm::elements::Module) -> Self {
        AM {
            module: BorrowedOrOwned::Owned(module),
            stack: Vec::new(),
            call_stack: Vec::new(),
        }
    }

    /// Construct an `AM` from a borrowed parity_wasm `Module`
    /// Note: this function does no validation
    pub fn from_ref(module: &'module wasm::elements::Module) -> Self {
        AM {
            module: BorrowedOrOwned::Borrowed(module),
            stack: Vec::new(),
            call_stack: Vec::new(),
        }
    }

    /// Visits the module with the given visitor
    /// Doesn't type check, the WASM it's passed is assumed to have been validated
    pub fn visit(
        &mut self,
        fun: u32,
        params: Vec<TVal<T>>,
        v: &mut impl Visitor<Output = T>,
    ) -> Result<(), WasmError> {
        let fun = self
            .module
            .code_section()
            .and_then(|x| x.bodies().get(fun as usize));
        if fun.is_none() {
            return Err(WasmError::FunctionNotFound);
        }
        let fun = fun.unwrap();
        let mut locals = params;
        for i in fun.locals() {
            let t = i.value_type();
            locals.push(TVal::default(t));
        }
        self.call_stack.push(AFrame { locals });
        for op in fun.code().elements() {
            match op {
                Op::I32Mul => {
                    let a = self.stack.pop().expect("i32.mul on empty stack!");
                    let b = self
                        .stack
                        .pop()
                        .expect("i32.mul on stack of length 1, not 2!");
                    assert_eq!(a.ty, b.ty, "Type error");
                    let ty = a.ty;
                    self.stack.push(TVal {
                        val: v.visit(AOp::Mul(a, b)),
                        ty,
                    });
                }
                Op::I32Add => {
                    let a = self.stack.pop().expect("i32.add on empty stack!");
                    let b = self
                        .stack
                        .pop()
                        .expect("i32.add on stack of length 1, not 2!");
                    assert_eq!(a.ty, b.ty, "Type error");
                    let ty = a.ty;
                    self.stack.push(TVal {
                        val: v.visit(AOp::Add(a, b)),
                        ty,
                    });
                }
                Op::I32Load(_align, _offset) => {
                    // TODO offset
                    let ptr = self.stack.pop().expect("i32.load on empty stack!");
                    let ty = ptr.ty;
                    self.stack.push(TVal {
                        val: v.visit(AOp::Load(ptr)),
                        ty,
                    });
                }
                Op::I32Store(_align, _offset) => {
                    // TODO offset
                    let val = self.stack.pop().expect("i32.store on empty stack!");
                    let ptr = self
                        .stack
                        .pop()
                        .expect("i32.store on stack of length 1, not 2!");
                    v.visit(AOp::Store(ptr, val));
                }
                Op::I32Const(c) => self.stack.push(TVal {
                    val: v.visit(AOp::I32Const(unsafe { std::mem::transmute(*c) })),
                    ty: WasmTy::I32,
                }),
                Op::GetLocal(i) => self
                    .stack
                    .push(self.call_stack.last().unwrap().locals[*i as usize].clone()),
                Op::SetLocal(i) => {
                    self.call_stack.last_mut().unwrap().locals[*i as usize] = self
                        .stack
                        .pop()
                        .expect("Tried to set local with an empty stack!")
                }
                Op::End => break,
                _ => panic!("{:?} instruction not implemented yet!", op),
            }
        }
        Ok(())
    }
}

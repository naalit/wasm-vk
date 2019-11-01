use parity_wasm as wasm;
pub use wasm::elements::Instruction as Op;
pub use wasm::elements::ValueType as WasmTy;

#[derive(Debug)]
pub enum WasmError {
    SerializationError(parity_wasm::SerializationError),
    FunctionNotFound,
    TypeError,
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
pub struct TVal<V> {
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
impl<V> Deref for TVal<V> {
    type Target = V;
    fn deref(&self) -> &V {
        &self.val
    }
}

/// An instruction, with attached operands of some type
pub enum AOp<T> {
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

    GetLocal(u32),
    SetLocal(u32, TVal<T>),
}

/// Essentially a catamorphism
pub trait Visitor {
    type Output: Clone;
    fn visit(&mut self, op: AOp<Self::Output>) -> Self::Output;
    fn add_local(&mut self, ty: WasmTy, val: Option<Self::Output>);
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
pub struct AM<'module, T> {
    module: BorrowedOrOwned<'module, parity_wasm::elements::Module>,
    stack: Vec<TVal<T>>,
}
impl<'module, T: Clone> AM<'module, T> {
    /// Construct an `AM` from a slice of bytes in the WebAssembly binary format
    /// Note: this function does no validation
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WasmError> {
        Ok(AM {
            module: BorrowedOrOwned::Owned(wasm::deserialize_buffer(bytes)?),
            stack: Vec::new(),
        })
    }

    /// Construct an `AM` from an owned parity_wasm `Module`
    /// Note: this function does no validation
    pub fn from_move(module: wasm::elements::Module) -> Self {
        AM {
            module: BorrowedOrOwned::Owned(module),
            stack: Vec::new(),
        }
    }

    /// Construct an `AM` from a borrowed parity_wasm `Module`
    /// Note: this function does no validation
    pub fn from_ref(module: &'module wasm::elements::Module) -> Self {
        AM {
            module: BorrowedOrOwned::Borrowed(module),
            stack: Vec::new(),
        }
    }

    /// Visits the module with the given visitor
    /// Doesn't type check, the WASM it's passed is assumed to have been validated
    pub fn visit(
        &mut self,
        fun_idx: u32,
        params: Vec<TVal<T>>,
        v: &mut impl Visitor<Output = T>,
    ) -> Result<(), WasmError> {
        let fun = self
            .module
            .code_section()
            .and_then(|x| x.bodies().get(fun_idx as usize));
        if fun.is_none() {
            return Err(WasmError::FunctionNotFound);
        }
        let fun = fun.unwrap();

        let fun_ty = self.module.function_section().unwrap().entries()[fun_idx as usize];
        let wasm::elements::Type::Function(fun_ty) =
            &self.module.type_section().unwrap().types()[fun_ty.type_ref() as usize];

        for (p, i) in params.into_iter().zip(0..) {
            if fun_ty.params().get(i) != Some(&p.ty) {
                return Err(WasmError::TypeError);
            }
            v.add_local(p.ty, Some(p.val));
        }

        for i in fun.locals() {
            v.add_local(i.value_type(), None);
        }

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
                Op::GetLocal(i) => self.stack.push(TVal {
                    val: v.visit(AOp::GetLocal(*i)),
                    ty: WasmTy::I32,
                }),
                Op::SetLocal(i) => {
                    v.visit(AOp::SetLocal(
                        *i,
                        self.stack
                            .pop()
                            .expect("Tried to set local with an empty stack!"),
                    ));
                }
                Op::End => break,
                _ => panic!("{:?} instruction not implemented yet!", op),
            }
        }
        Ok(())
    }
}

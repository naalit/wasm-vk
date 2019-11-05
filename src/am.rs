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
    Eq(TVal<T>, TVal<T>),
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

    GetGlobalImport(String, String),

    GetLocal(u32),
    SetLocal(u32, TVal<T>),
}

pub enum BlockOp<T> {
    If(TVal<T>),
}

/// Essentially a catamorphism
pub trait Visitor {
    /// Some sort of data representing an operand or result of an instruction.
    type Output;
    /// Some sort of data attached to blocks (e.g. labels)
    type BlockData;

    /// Called after the last `add_local` call
    fn init(&mut self) {}

    fn br_break(&mut self, block: &Self::BlockData);

    /// Starts the type of block `op`, returning a BlockData to keep track of this block
    ///
    /// This is a good time to change any internal state relating to the current block.
    /// Note, though, that you shouldn't need to maintain any kind of stack - just use BlockData.
    fn start_block(&mut self, op: BlockOp<Self::Output>) -> Self::BlockData;
    /// Ends the If block at `if_data`, returning a new BlockData for the Else block
    fn else_block(&mut self, if_data: Self::BlockData) -> Self::BlockData;
    /// Ends the block at `data`
    fn end_block(&mut self, data: Self::BlockData);

    /// Visits one instruction (that isn't a block start or end)
    fn visit(&mut self, op: AOp<Self::Output>) -> Self::Output;

    /// Add a local variable of type `ty`.
    /// If `val` is `Some`, this is a parameter with the value `val`.
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
    globals: Vec<(String, String, WasmTy)>,
    stack: Vec<TVal<T>>,
}
impl<'module, T: Clone> AM<'module, T> {
    fn from_bo(module: BorrowedOrOwned<'module, parity_wasm::elements::Module>) -> Self {
        let globals = module.import_section().map(|imports| {
            let mut v = Vec::new();
            for i in imports.entries() {
                if let wasm::elements::External::Global(t) = i.external() {
                    v.push((i.module().to_owned(), i.field().to_owned(), t.content_type()));
                }
            }
            v
        }).unwrap_or_else(Vec::new);
        AM {
            module,
            globals,
            stack: Vec::new(),
        }
    }

    /// Construct an `AM` from a slice of bytes in the WebAssembly binary format
    /// Note: this function does no validation
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WasmError> {
        Ok(AM::from_bo(
            BorrowedOrOwned::Owned(wasm::deserialize_buffer(bytes)?)))
    }

    /// Construct an `AM` from an owned parity_wasm `Module`
    /// Note: this function does no validation
    pub fn from_move(module: wasm::elements::Module) -> Self {
        AM::from_bo(BorrowedOrOwned::Owned(module))
    }

    /// Construct an `AM` from a borrowed parity_wasm `Module`
    /// Note: this function does no validation
    pub fn from_ref(module: &'module wasm::elements::Module) -> Self {
        AM::from_bo(BorrowedOrOwned::Borrowed(module))
    }

    /// Visits the module with the given visitor
    /// Doesn't type check, the WASM it's passed is assumed to have been validated
    pub fn visit<V: Visitor<Output=T>>(
        &mut self,
        fun_idx: u32,
        params: Vec<TVal<T>>,
        v: &mut V,
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

        struct TypeMap<T> {
            i_32: T,
            i_64: T,
            f_32: T,
            f_64: T,
        }
        impl<T> TypeMap<T> {
            fn new(mut f: impl FnMut(WasmTy) -> T) -> Self {
                TypeMap {
                    i_32: f(WasmTy::I32),
                    i_64: f(WasmTy::I64),
                    f_32: f(WasmTy::F32),
                    f_64: f(WasmTy::F64),
                }
            }
            fn get(&self, t: WasmTy) -> &T {
                match t {
                    WasmTy::I32 => &self.i_32,
                    WasmTy::I64 => &self.i_64,
                    WasmTy::F32 => &self.f_32,
                    WasmTy::F64 => &self.f_64,
                }
            }
        }

        let mut n_locals = 0;

        for (p, i) in params.iter().zip(0..) {
            n_locals += 1;
            if fun_ty.params().get(i) != Some(&p.ty) {
                return Err(WasmError::TypeError);
            }
            v.add_local(p.ty, None);//Some(p.val));
        }

        for i in fun.locals() {
            for _ in 0..i.count() {
                n_locals += 1;
                v.add_local(i.value_type(), None);
            }
        }

        let tmps = TypeMap::new(|t| { v.add_local(t, None); n_locals += 1; n_locals - 1 });

        v.init();

        for (p, i) in params.into_iter().zip(0..) {
            v.visit(AOp::SetLocal(i, p));
        }

        let mut blocks = Vec::new();

        for op in fun.code().elements() {
            macro_rules! binop {
                ($x:ident) => {{
                    let a = self
                        .stack
                        .pop()
                        .expect(&format!("{} on empty stack!", stringify!($x)));
                    let b = self
                        .stack
                        .pop()
                        .expect(&format!("{} on stack of length 1, not 2!", stringify!($x)));
                    assert_eq!(a.ty, b.ty, "Type error");
                    let ty = a.ty;
                    self.stack.push(TVal {
                        val: v.visit(AOp::$x(a, b)),
                        ty,
                    });
                }};
            }

            match op {
                Op::GetGlobal(i) => {
                    let (module, field, t) = &self.globals[*i as usize];
                    let g = v.visit(AOp::GetGlobalImport(module.to_string(), field.to_string()));
                    self.stack.push(TVal { val: g, ty: *t });
                }
                Op::Br(i) => {
                    let data: &(V::BlockData, bool) = &blocks[blocks.len() - *i as usize - 1];
                    // let mut data: (V::BlockData, bool) = blocks.pop().expect("'br' outside of block!");
                    // for _ in 0..*i {
                    //     data = blocks.pop().expect("'br i' in less than 'i' blocks!");
                    // }
                    let mut result = None;
                    if let Some(s) = self.stack.pop() {
                        result = Some(s.ty);
                        v.visit(AOp::SetLocal(*tmps.get(s.ty), s));
                    }
                    v.br_break(&data.0);
                    if let Some(t) = result {
                        v.visit(AOp::GetLocal(*tmps.get(t)));
                    }
                }
                Op::If(_) => {
                    let data =
                        v.start_block(BlockOp::If(self.stack.pop().expect("If on empty stack!")));
                    blocks.push((data, true));
                }
                Op::Else => {
                    if let Some((data, true)) = blocks.pop() {
                        let data = v.else_block(data);
                        blocks.push((data, false));
                    } else {
                        panic!("Else without If!");
                    }
                }
                Op::End => {
                    if let Some((data, is_if)) = blocks.pop() {
                        // If this ends an If, insert a fake else block
                        if is_if {
                            let data = v.else_block(data);
                            v.end_block(data);
                        } else {
                            v.end_block(data);
                        }
                    } else {
                        // End the main function
                        break;
                    }
                }
                Op::I32Mul => binop!(Mul),
                Op::I32Eq => binop!(Eq),
                Op::I32Add => binop!(Add),
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
                _ => panic!("{:?} instruction not implemented yet!", op),
            }
        }
        Ok(())
    }
}

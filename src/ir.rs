use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum INumOp {
    Add,
    Sub,
    Mul,
    DivS,
    DivU,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ICompOp {
    Eq,
    NEq,
    LeU,
    LeS,
    GeU,
    GeS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Width {
    W32,
    W64,
}

#[derive(Clone, Debug, PartialEq)]
enum Const {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Sym(u32);

#[derive(Debug, Clone, PartialEq)]
struct Global {
    ty: wasm::GlobalType,
    idx: u32,
}

#[derive(Debug, Clone)]
enum Base {
    Nop,
    Const(Const),
    Load(wasm::ValueType, Box<Base>),
    /// Store(ptr, val)
    Store(Box<Base>, Box<Base>),
    INumOp(Width, INumOp, Box<Base>, Box<Base>),
    ICompOp(Width, ICompOp, Box<Base>, Box<Base>),
    SetLocal(u32, Box<Base>),
    GetLocal(u32),
    GetGlobal(Global),
    /// A left-associative block
    Seq(Box<Base>, Box<Base>),
    If {
        cond: Box<Base>,
        t: Box<Base>,
        f: Box<Base>,
    },
}

#[derive(Debug, Clone)]
enum Direct {
    Nop,
    Const(Const),
    Load(wasm::ValueType, Box<Direct>),
    /// Store(ptr, val)
    Store(Box<Direct>, Box<Direct>),
    INumOp(Width, INumOp, Box<Direct>, Box<Direct>),
    ICompOp(Width, ICompOp, Box<Direct>, Box<Direct>),
    SetLocal(u32, Box<Direct>),
    GetLocal(u32),
    GetGlobal(Global),
    Seq(Box<Direct>, Box<Direct>),
    Label(Box<Direct>),
    Br(u32),
    // Block(Vec<Direct>),
    If {
        cond: Box<Direct>,
        t: Box<Direct>,
        f: Box<Direct>,
    },
}

impl Direct {
    fn map(self, f: impl Fn(Self) -> Self) -> Self {
        match self {
            Direct::INumOp(w, op, a, b) => f(Direct::INumOp(w, op, Box::new(f(*a)), Box::new(f(*b)))),
            Direct::ICompOp(w, op, a, b) => f(Direct::ICompOp(w, op, Box::new(f(*a)), Box::new(f(*b)))),
            Direct::Seq(a, b) => f(Direct::Seq(Box::new(f(*a)), Box::new(f(*b)))),
            Direct::Store(a, b) => f(Direct::Store(Box::new(f(*a)), Box::new(f(*b)))),
            Direct::SetLocal(u, x) => f(Direct::SetLocal(u, Box::new(f(*x)))),
            Direct::Load(t, p) => f(Direct::Load(t, Box::new(f(*p)))),
            Direct::Label(a) => f(Direct::Label(Box::new(f(*a)))),
            x => f(x),
        }
    }
    fn map_no_lbl(self, f: impl Fn(Self) -> Self) -> Self {
        match self {
            Direct::INumOp(w, op, a, b) => f(Direct::INumOp(w, op, Box::new(f(*a)), Box::new(f(*b)))),
            Direct::ICompOp(w, op, a, b) => f(Direct::ICompOp(w, op, Box::new(f(*a)), Box::new(f(*b)))),
            Direct::Seq(a, b) => f(Direct::Seq(Box::new(f(*a)), Box::new(f(*b)))),
            Direct::Store(a, b) => f(Direct::Store(Box::new(f(*a)), Box::new(f(*b)))),
            Direct::SetLocal(u, x) => f(Direct::SetLocal(u, Box::new(f(*x)))),
            Direct::Load(t, p) => f(Direct::Load(t, Box::new(f(*p)))),
            x => f(x),
        }
    }
    fn fold_leaves<T>(&self, start: T, f: &impl Fn(T, &Self) -> T) -> T {
        match self {
            Direct::Seq(a, b) | Direct::INumOp(_, _, a, b) | Direct::ICompOp(_, _, a, b) | Direct::If { t: a, f: b, .. } | Direct::Store(a, b) => b.fold_leaves(a.fold_leaves(start, f), f),
            Direct::SetLocal(_, x) | Direct::Load(_, x) => x.fold_leaves(start, f),
            x => f(start, x),
        }
    }

    fn nest(self, max: u32) -> Self {
        self.map_no_lbl(|x| match x {
            Direct::Br(i) if i >= max => Direct::Br(i + 1),
            Direct::Label(a) => Direct::Label(Box::new(a.nest(max + 1))),
            x => x,
        })
    }
    fn lift(self, max: u32) -> Self {
        self.map_no_lbl(|x| match x {
            Direct::Br(i) if i >= max => Direct::Br(i - 1),
            Direct::Label(a) => Direct::Label(Box::new(a.lift(max + 1))),
            x => x,
        })
    }

    /// Does this code have any branches, and if so what's the maximum number
    /// of `Seq`s they can branch out of, above this code?
    /// In `Seq(a, b)`, if `a.br().is_some()`, then `a` might branch out of the `Seq`.
    ///
    /// Example:
    /// ```
    /// use playground::*;
    /// let d = parse("{ br 2 }");
    /// // 1, not 2, because one of the `Seqs` has "already" been branched out of
    /// assert_eq!(d.br(), Some(1));
    /// ```
    pub fn br(&self) -> Option<u32> {
        self.fold_leaves(None, &|acc, x| {println!("br of {:?}", x); match (acc, x) {
            (None, Direct::Br(i)) => Some(*i),
            (Some(a), Direct::Br(b)) => Some(a.max(*b)),
            (None, Direct::Label(a)) => a.br().and_then(|x| x.checked_sub(1)),
            (Some(q), Direct::Label(a)) => Some(a.br().and_then(|x| x.checked_sub(1)).map_or(q, |x| x.max(q))),
            (acc, _) => acc,
        }})
    }

    fn replace_br(self, with: Self, offset: u32) -> Self {
        self.map_no_lbl(|x| match x {
            Direct::Br(i) if i >= offset => with.clone(),
            Direct::Label(a) => Direct::Label(Box::new(a.replace_br(with.clone(), offset + 1))),
            x => x,
        })
    }

    /// We know this code might branch, so here's something that comes after it
    /// and how many `Seqs` up it is.
    /// Put it somewhere where it won't be run after a branch.
    ///
    /// This function may clone `x`.
    fn insert(self, x: Self, offset: u32) -> Self {
        match &self {
            Direct::INumOp(_,_,_,_) | Direct::ICompOp(_,_,_,_) | Direct::SetLocal(_,_) | Direct::Load(_, _) | Direct::Store(_, _) if self.br().is_some() => panic!("Branches are currently not supported in arguments to expressions"),
            _ => (),
        }
        match self {
            Direct::Nop => x,
            Direct::Br(i) if i < offset => x,
            Direct::Br(i) if i >= offset => Direct::Br(i),
            Direct::Label(a) => Direct::Label(Box::new(a.insert(x.nest(0), offset + 1))),
            Direct::Seq(a, b) => {
                if a.br().is_none() && b.br().is_none() {
                    Direct::Seq(Box::new(Direct::Seq(a, b)), Box::new(x))
                } else if a.br().is_some() {
                    a.insert(*b, 0).insert(x, offset)
                } else {
                    Direct::Seq(a, Box::new(b.insert(x, offset)))
                }
            }
            Direct::If { cond, t, f } => {
                assert_eq!(cond.br(), None, "br not allowed in expressions");
                if t.br().is_some() || f.br().is_some() {
                    Direct::If {
                        cond,
                        t: Box::new(t.insert(x.clone(), offset)),
                        f: Box::new(f.insert(x, offset)),
                    }
                } else {
                    Direct::Seq(Box::new(Direct::If{ cond, t, f }), Box::new(x))
                }
            }
            op => Direct::Seq(Box::new(op), Box::new(x)),
        }
    }

    fn base(self) -> Base {
        match self {
            Direct::Nop => Base::Nop,
            Direct::INumOp(w, op, a, b) => Base::INumOp(w, op, Box::new(a.base()), Box::new(b.base())),
            Direct::ICompOp(w, op, a, b) => Base::ICompOp(w, op, Box::new(a.base()), Box::new(b.base())),
            Direct::Const(c) => Base::Const(c),
            Direct::If { cond, t, f } => Base::If { cond: Box::new(cond.base()), t: Box::new(t.base()), f: Box::new(f.base()) },
            Direct::Br(_) => Base::Nop,
            Direct::GetLocal(l) => Base::GetLocal(l),
            Direct::GetGlobal(g) => Base::GetGlobal(g),
            Direct::Label(a) => a.base(),
            Direct::Seq(a, b) => {
                if a.br().is_some() {
                    a.insert(*b, 0).base()
                } else {
                    Base::Seq(Box::new(a.base()), Box::new(b.base()))
                }
            }
            Direct::Store(a, b) => Base::Store(Box::new(a.base()), Box::new(b.base())),
            Direct::SetLocal(l, v) => Base::SetLocal(l, Box::new(v.base())),
            Direct::Load(t, p) => Base::Load(t, Box::new(p.base())),
        }
    }
}

#[derive(Debug)]
struct Fun<T> {
    params: Vec<wasm::ValueType>,
    body: T,
}

pub fn test(w: &wasm::Module) {
    let d = direct(w);
    println!("{:?}", d);
    println!("Base: {:#?}", d.into_iter().map(|Fun {params, body}| Fun { params, body: body.base() }).collect::<Vec<_>>());
}

fn direct(w: &wasm::Module) -> Vec<Fun<Direct>> {
    let imports = w.import_section().map_or_else(Vec::new, |x| x.entries().to_vec());
    let mut globals: Vec<_> = imports.iter().map(|x| x.external()).filter_map(|x| if let wasm::External::Global(g) = x { Some(*g) } else { None }).collect();
    globals.append(&mut w.global_section().map_or_else(Vec::new, |x| x.entries().iter().map(|x| *x.global_type()).collect()));

    let mut funs = Vec::new();
    for (fun, body) in w.function_section().map(|x| x.entries().to_vec()).unwrap_or_else(Vec::new).into_iter().zip(w.code_section().map(|x| x.bodies().to_vec()).unwrap_or_else(Vec::new)) {
        let mut stack = Vec::new();

        enum BlockTy {
            Block(Vec<Direct>),
            If(Box<Direct>, Vec<Direct>),
            Else(Box<Direct>, Vec<Direct>, Vec<Direct>),
        }
        impl BlockTy {
            fn push(&mut self, op: Direct) {
                match self {
                    BlockTy::Block(v)
                    | BlockTy::If(_, v)
                    | BlockTy::Else(_, _, v) => {
                        v.push(op);
                    }
                }
            }

            fn op(self) -> Direct {
                fn fold(v: Vec<Direct>) -> Direct {
                    Direct::Label(Box::new(v.into_iter().fold(Direct::Nop, |acc, x| Direct::Seq(Box::new(acc), Box::new(x)))))
                }

                match self {
                    BlockTy::If(cond, v) => Direct::If {
                        cond,
                        t: Box::new(fold(v)),
                        f: Box::new(Direct::Nop),
                    },
                    BlockTy::Else(cond, t, f) => Direct::If {
                        cond,
                        t: Box::new(fold(t)),
                        f: Box::new(fold(f)),
                    },
                    BlockTy::Block(v) => fold(v),
                }
            }
        }

        let mut blocks = vec![BlockTy::Block(Vec::new())];

        let ty = fun.type_ref();
        let wasm::Type::Function(ty) = &w.type_section().unwrap().types()[ty as usize];
        let params = ty.params().to_vec();

        let code = body.code();

        macro_rules! numop {
            ($w:ident, $op:ident) => {{
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(Direct::INumOp(Width::$w, INumOp::$op, Box::new(a), Box::new(b)));
            }}
        }
        macro_rules! compop {
            ($w:ident, $op:ident) => {{
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(Direct::ICompOp(Width::$w, ICompOp::$op, Box::new(a), Box::new(b)));
            }}
        }

        for op in code.elements() {
            use wasm::Instruction::*;
            match op {
                Br(i) => blocks.last_mut().unwrap().push(Direct::Br(*i)),
                Nop => (),
                SetLocal(u) => {
                    let val = stack.pop().unwrap();
                    blocks.last_mut().unwrap().push(Direct::SetLocal(*u, Box::new(val)));
                }
                I32Load(_, _) => {
                    let val = stack.pop().unwrap();
                    stack.push(Direct::Load(wasm::ValueType::I32, Box::new(val)))
                }
                I32Store(_, _) => {
                    let val = stack.pop().unwrap();
                    let ptr = stack.pop().unwrap();
                    blocks.last_mut().unwrap().push(Direct::Store(Box::new(ptr), Box::new(val)))
                }
                GetGlobal(idx) => stack.push(Direct::GetGlobal(Global { ty: globals[*idx as usize], idx: *idx })),
                GetLocal(u) => stack.push(Direct::GetLocal(*u)),
                I32Const(i) => stack.push(Direct::Const(Const::I32(*i))),
                I64Const(i) => stack.push(Direct::Const(Const::I64(*i))),
                F32Const(i) => stack.push(Direct::Const(Const::F32(f32::from_bits(*i)))),
                F64Const(i) => stack.push(Direct::Const(Const::F64(f64::from_bits(*i)))),
                I32Add => numop!(W32, Add),
                I32Mul => numop!(W32, Mul),
                I32Sub => numop!(W32, Sub),
                I32DivS => numop!(W32, DivS),
                I32DivU => numop!(W32, DivU),
                I32Eq => compop!(W32, Eq),
                I32Ne => compop!(W32, NEq),
                If(_ty) => {
                    let cond = stack.pop().unwrap();
                    blocks.push(BlockTy::If(Box::new(cond), Vec::new()));
                },
                Else => match blocks.pop().unwrap() {
                    BlockTy::If(cond, v) => blocks.push(BlockTy::Else(cond, v, Vec::new())),
                    _ => panic!("Else without if"),
                },
                End => if blocks.len() <= 1 {
                    break
                } else {
                    let b = blocks.pop().unwrap();
                    blocks.last_mut().unwrap().push(b.op());
                },
                x => panic!("Instruction {} not supported", x),
            }
        }

        assert_eq!(blocks.len(), 1);
        assert_eq!(stack.len(), 0, "Stuff left on stack");

        funs.push(Fun {
            params,
            body: blocks.pop().unwrap().op(),
        });
    }
    funs
}

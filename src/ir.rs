use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum INumOp {
    Add,
    Sub,
    Mul,
    DivS,
    DivU,
    Shl,
    ShrU,
    ShrS,
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICompOp {
    Eq,
    NEq,
    LeU,
    LeS,
    GeU,
    GeS,
    LtU,
    LtS,
    GtU,
    GtS,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CvtOp {
    F32toI32S,
    F32toI32U,
    I32toF32S,
    I32toF32U,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FUnOp {
    Sqrt,
    Abs,
    Neg,
    Ceil,
    Floor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FNumOp {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FCompOp {
    Eq,
    NEq,
    Le,
    Ge,
    Lt,
    Gt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    W32,
    W64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Const {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Sym(u32);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Local {
    pub ty: wasm::ValueType,
    pub idx: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Global {
    pub ty: wasm::GlobalType,
    pub idx: u32,
}

#[derive(Debug, Clone)]
pub enum Base {
    Nop,
    Const(Const),
    Load(wasm::ValueType, Box<Base>),
    /// Store(ptr, val)
    Store(wasm::ValueType, Box<Base>, Box<Base>),
    INumOp(Width, INumOp, Box<Base>, Box<Base>),
    ICompOp(Width, ICompOp, Box<Base>, Box<Base>),
    FCompOp(Width, FCompOp, Box<Base>, Box<Base>),
    FNumOp(Width, FNumOp, Box<Base>, Box<Base>),
    CvtOp(CvtOp, Box<Base>),
    FUnOp(Width, FUnOp, Box<Base>),
    SetLocal(Local, Box<Base>),
    GetLocal(Local),
    GetGlobal(Global),
    Loop(Box<Base>),
    Break,
    Continue,
    Return,
    Call(u32, Vec<Base>),
    /// A left-associative block
    Seq(Box<Base>, Box<Base>),
    If {
        cond: Box<Base>,
        t: Box<Base>,
        f: Box<Base>,
    },
}

impl Base {
    fn map(self, f: impl Copy + Fn(Self) -> Self) -> Self {
        match self {
            Base::INumOp(w, op, a, b) => {
                f(Base::INumOp(w, op, Box::new(a.map(f)), Box::new(b.map(f))))
            }
            Base::ICompOp(w, op, a, b) => {
                f(Base::ICompOp(w, op, Box::new(a.map(f)), Box::new(b.map(f))))
            }
            Base::FCompOp(w, op, a, b) => {
                f(Base::FCompOp(w, op, Box::new(a.map(f)), Box::new(b.map(f))))
            }
            Base::FNumOp(w, op, a, b) => {
                f(Base::FNumOp(w, op, Box::new(a.map(f)), Box::new(b.map(f))))
            }
            Base::FUnOp(w, op, a) => f(Base::FUnOp(w, op, Box::new(a.map(f)))),
            Base::CvtOp(op, a) => f(Base::CvtOp(op, Box::new(a.map(f)))),
            Base::Seq(a, b) => f(Base::Seq(Box::new(a.map(f)), Box::new(b.map(f)))),
            Base::Store(t, a, b) => f(Base::Store(t, Box::new(a.map(f)), Box::new(b.map(f)))),
            Base::SetLocal(u, x) => f(Base::SetLocal(u, Box::new(x.map(f)))),
            Base::Load(t, p) => f(Base::Load(t, Box::new(p.map(f)))),
            Base::Loop(a) => f(Base::Loop(Box::new(a.map(f)))),
            Base::If { cond, t, f: fa } => f(Base::If {
                cond: Box::new(cond.map(f)),
                t: Box::new(t.map(f)),
                f: Box::new(fa.map(f)),
            }),
            Base::Call(i, params) => f(Base::Call(
                i,
                params.into_iter().map(|x| x.map(f)).collect(),
            )),
            x => f(x),
        }
    }

    fn fold_leaves<T>(&self, start: T, f: &impl Fn(T, &Self) -> T) -> T {
        match self {
            Base::Seq(a, b)
            | Base::INumOp(_, _, a, b)
            | Base::ICompOp(_, _, a, b)
            | Base::FCompOp(_, _, a, b)
            | Base::FNumOp(_, _, a, b)
            | Base::If { t: a, f: b, .. }
            | Base::Store(_, a, b) => b.fold_leaves(a.fold_leaves(start, f), f),
            Base::Loop(x)
            | Base::SetLocal(_, x)
            | Base::Load(_, x)
            | Base::CvtOp(_, x)
            | Base::FUnOp(_, _, x) => x.fold_leaves(start, f),
            Base::Call(_, params) => params.iter().fold(start, |acc, x| x.fold_leaves(acc, f)),
            x => f(start, x),
        }
    }
    fn fold<T>(&self, start: T, f: &impl Fn(T, &Self) -> T) -> T {
        let n = f(start, self);
        match self {
            Base::Seq(a, b)
            | Base::INumOp(_, _, a, b)
            | Base::FNumOp(_, _, a, b)
            | Base::ICompOp(_, _, a, b)
            | Base::FCompOp(_, _, a, b)
            | Base::If { t: a, f: b, .. }
            | Base::Store(_, a, b) => b.fold(a.fold(n, f), f),
            Base::Loop(x)
            | Base::SetLocal(_, x)
            | Base::Load(_, x)
            | Base::CvtOp(_, x)
            | Base::FUnOp(_, _, x) => x.fold(n, f),
            Base::Call(_, params) => params.iter().fold(n, |acc, x| x.fold_leaves(acc, f)),
            _ => n,
        }
    }

    pub fn locals(&self) -> Vec<Local> {
        self.fold(Vec::new(), &|mut acc, x| match x {
            Base::SetLocal(l, _) | Base::GetLocal(l) => {
                acc.push(*l);
                acc
            }
            _ => acc,
        })
    }
}

#[derive(Debug, Clone)]
enum Direct {
    Nop,
    Const(Const),
    Load(wasm::ValueType, Box<Direct>),
    /// Store(ptr, val)
    Store(wasm::ValueType, Box<Direct>, Box<Direct>),
    INumOp(Width, INumOp, Box<Direct>, Box<Direct>),
    ICompOp(Width, ICompOp, Box<Direct>, Box<Direct>),
    FCompOp(Width, FCompOp, Box<Direct>, Box<Direct>),
    FNumOp(Width, FNumOp, Box<Direct>, Box<Direct>),
    CvtOp(CvtOp, Box<Direct>),
    FUnOp(Width, FUnOp, Box<Direct>),
    SetLocal(Local, Box<Direct>),
    GetLocal(Local),
    GetGlobal(Global),
    Seq(Box<Direct>, Box<Direct>),
    Label(Box<Direct>),
    Loop(Box<Direct>),
    Break,
    Continue,
    Return,
    Call(u32, Vec<Direct>),
    Br(u32),
    // Block(Vec<Direct>),
    If {
        cond: Box<Direct>,
        t: Box<Direct>,
        f: Box<Direct>,
    },
}

impl Direct {
    fn map(self, f: impl Copy + Fn(Self) -> Self) -> Self {
        match self {
            Direct::INumOp(w, op, a, b) => f(Direct::INumOp(
                w,
                op,
                Box::new(a.map(f)),
                Box::new(b.map(f)),
            )),
            Direct::ICompOp(w, op, a, b) => f(Direct::ICompOp(
                w,
                op,
                Box::new(a.map(f)),
                Box::new(b.map(f)),
            )),
            Direct::FCompOp(w, op, a, b) => f(Direct::FCompOp(
                w,
                op,
                Box::new(a.map(f)),
                Box::new(b.map(f)),
            )),
            Direct::FNumOp(w, op, a, b) => f(Direct::FNumOp(
                w,
                op,
                Box::new(a.map(f)),
                Box::new(b.map(f)),
            )),
            Direct::FUnOp(w, op, a) => f(Direct::FUnOp(w, op, Box::new(a.map(f)))),
            Direct::CvtOp(op, a) => f(Direct::CvtOp(op, Box::new(a.map(f)))),
            Direct::Seq(a, b) => f(Direct::Seq(Box::new(a.map(f)), Box::new(b.map(f)))),
            Direct::Store(t, a, b) => f(Direct::Store(t, Box::new(a.map(f)), Box::new(b.map(f)))),
            Direct::SetLocal(u, x) => f(Direct::SetLocal(u, Box::new(x.map(f)))),
            Direct::Load(t, p) => f(Direct::Load(t, Box::new(p.map(f)))),
            Direct::Label(a) => f(Direct::Label(Box::new(a.map(f)))),
            Direct::Loop(a) => f(Direct::Loop(Box::new(a.map(f)))),
            Direct::If { cond, t, f: fa } => f(Direct::If {
                cond: Box::new(cond.map(f)),
                t: Box::new(t.map(f)),
                f: Box::new(fa.map(f)),
            }),
            Direct::Call(i, params) => f(Direct::Call(
                i,
                params.into_iter().map(|x| x.map(f)).collect(),
            )),
            x => f(x),
        }
    }

    fn map_no_lbl(self, f: impl Copy + Fn(Self) -> Self) -> Self {
        match self {
            Direct::INumOp(w, op, a, b) => f(Direct::INumOp(
                w,
                op,
                Box::new(a.map_no_lbl(f)),
                Box::new(b.map_no_lbl(f)),
            )),
            Direct::ICompOp(w, op, a, b) => f(Direct::ICompOp(
                w,
                op,
                Box::new(a.map_no_lbl(f)),
                Box::new(b.map_no_lbl(f)),
            )),
            Direct::FCompOp(w, op, a, b) => f(Direct::FCompOp(
                w,
                op,
                Box::new(a.map_no_lbl(f)),
                Box::new(b.map_no_lbl(f)),
            )),
            Direct::FNumOp(w, op, a, b) => f(Direct::FNumOp(
                w,
                op,
                Box::new(a.map_no_lbl(f)),
                Box::new(b.map_no_lbl(f)),
            )),
            Direct::FUnOp(w, op, a) => f(Direct::FUnOp(w, op, Box::new(a.map_no_lbl(f)))),
            Direct::CvtOp(op, a) => f(Direct::CvtOp(op, Box::new(a.map_no_lbl(f)))),
            Direct::Seq(a, b) => f(Direct::Seq(
                Box::new(a.map_no_lbl(f)),
                Box::new(b.map_no_lbl(f)),
            )),
            Direct::Store(t, a, b) => f(Direct::Store(
                t,
                Box::new(a.map_no_lbl(f)),
                Box::new(b.map_no_lbl(f)),
            )),
            Direct::SetLocal(u, x) => f(Direct::SetLocal(u, Box::new(x.map_no_lbl(f)))),
            Direct::Load(t, p) => f(Direct::Load(t, Box::new(p.map_no_lbl(f)))),
            Direct::If { cond, t, f: fa } => f(Direct::If {
                cond: Box::new(cond.map_no_lbl(f)),
                t: Box::new(t.map_no_lbl(f)),
                f: Box::new(fa.map_no_lbl(f)),
            }),
            Direct::Call(i, params) => f(Direct::Call(
                i,
                params.into_iter().map(|x| x.map_no_lbl(f)).collect(),
            )),
            x => f(x),
        }
    }

    fn fold_leaves<T>(&self, start: T, f: &impl Fn(T, &Self) -> T) -> T {
        match self {
            Direct::Seq(a, b)
            | Direct::INumOp(_, _, a, b)
            | Direct::ICompOp(_, _, a, b)
            | Direct::FCompOp(_, _, a, b)
            | Direct::FNumOp(_, _, a, b)
            | Direct::If { t: a, f: b, .. }
            | Direct::Store(_, a, b) => b.fold_leaves(a.fold_leaves(start, f), f),
            Direct::SetLocal(_, x)
            | Direct::Load(_, x)
            | Direct::FUnOp(_, _, x)
            | Direct::CvtOp(_, x) => x.fold_leaves(start, f),
            Direct::Call(_, params) => params.iter().fold(start, |acc, x| x.fold_leaves(acc, f)),
            x => f(start, x),
        }
    }
}

use std::sync::RwLock;
lazy_static::lazy_static! {
    static ref NLOCALS: RwLock<u32> = RwLock::new(0);
}

impl Direct {
    fn nest(self, max: u32) -> Self {
        self.map_no_lbl(|x| match x {
            Direct::Br(i) if i >= max => Direct::Br(i + 1),
            Direct::Label(a) => Direct::Label(Box::new(a.nest(max + 1))),
            Direct::Loop(a) => Direct::Loop(Box::new(a.nest(max + 1))),
            x => x,
        })
    }
    fn lift(self, max: u32) -> Self {
        self.map_no_lbl(|x| match x {
            Direct::Br(i) if i >= max => Direct::Br(i - 1),
            Direct::Label(a) => Direct::Label(Box::new(a.lift(max + 1))),
            Direct::Loop(a) => Direct::Loop(Box::new(a.lift(max + 1))),
            x => x,
        })
    }

    /// Does this code have any branches, and if so what's the maximum number
    /// of `labels`s (or `Loop`s) they can branch out of, above this code?
    /// In `Label(a)`, if `a.br().is_some()`, then `a` might branch out of the `Label`.
    ///
    /// For example, `<(block (br 2))>.br() == Some(1)`
    pub fn br(&self) -> Option<u32> {
        self.fold_leaves(None, &|acc, x| match (acc, x) {
            (None, Direct::Br(i)) => Some(*i),
            (Some(a), Direct::Br(b)) => Some(a.max(*b)),
            (None, Direct::Label(a)) => a.br().and_then(|x| x.checked_sub(1)),
            (Some(q), Direct::Label(a)) => Some(
                a.br()
                    .and_then(|x| x.checked_sub(1))
                    .map_or(q, |x| x.max(q)),
            ),
            (None, Direct::Loop(a)) => a.br().and_then(|x| x.checked_sub(1)),
            (Some(q), Direct::Loop(a)) => Some(
                a.br()
                    .and_then(|x| x.checked_sub(1))
                    .map_or(q, |x| x.max(q)),
            ),
            (acc, _) => acc,
        })
    }

    /// Replaces any `Br(i)`s with `with`, if `i >= offset`.
    /// If `exact` is true, only matches `Br(i)` where `i == offset`.
    fn replace_br(self, with: Self, offset: u32, exact: bool) -> Self {
        self.map_no_lbl(|x| match x {
            Direct::Br(i) if (!exact && i >= offset) || i == offset => with.clone(),
            Direct::Label(a) => {
                Direct::Label(Box::new(a.replace_br(with.clone(), offset + 1, exact)))
            }
            Direct::Loop(a) => {
                if a.br().map_or(false, |x| x > offset) {
                    // Break out of this loop, then run "with"
                    // let l = false
                    // loop (a.replace_br(Seq(l = true, Op("break")), offset + 1))
                    // if l { with } else {}
                    let mut lk = NLOCALS.write().unwrap();
                    *lk += 1;
                    let l = *lk - 1;
                    drop(lk);
                    let l = Local {
                        ty: wasm::ValueType::I32,
                        idx: l,
                    };

                    Direct::Seq(
                        Box::new(Direct::Seq(
                            Box::new(Direct::SetLocal(l, Box::new(Direct::Const(Const::I32(0))))),
                            Box::new(Direct::Loop(Box::new(a.replace_br(
                                Direct::Seq(
                                    Box::new(Direct::SetLocal(
                                        l,
                                        Box::new(Direct::Const(Const::I32(1))),
                                    )),
                                    Box::new(Direct::Break),
                                ),
                                offset + 1,
                                exact,
                            )))),
                        )),
                        Box::new(Direct::If {
                            cond: Box::new(Direct::GetLocal(l)),
                            t: Box::new(with.clone()),
                            f: Box::new(Direct::Nop),
                        }),
                    )
                } else {
                    Direct::Loop(Box::new(a.replace_br(with.clone(), offset + 1, exact)))
                }
            }
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
            Direct::INumOp(_, _, _, _)
            | Direct::FNumOp(_, _, _, _)
            | Direct::FUnOp(_, _, _)
            | Direct::CvtOp(_, _)
            | Direct::ICompOp(_, _, _, _)
            | Direct::FCompOp(_, _, _, _)
            | Direct::SetLocal(_, _)
            | Direct::Load(_, _)
            | Direct::Store(_, _, _)
                if self.br().is_some() =>
            {
                panic!("Branches are currently not supported in arguments to expressions")
            }
            _ => (),
        }
        match self {
            Direct::Loop(a) => {
                if a.br().map_or(true, |x| x == 0) {
                    Direct::Seq(Box::new(Direct::Loop(a)), Box::new(x))
                } else {
                    let mut lk = NLOCALS.write().unwrap();
                    *lk += 1;
                    let l = *lk - 1;
                    drop(lk);
                    let l = Local {
                        ty: wasm::ValueType::I32,
                        idx: l,
                    };

                    Direct::Seq(
                        Box::new(Direct::Seq(
                            Box::new(Direct::SetLocal(l, Box::new(Direct::Const(Const::I32(1))))),
                            Box::new(Direct::Loop(Box::new(a.replace_br(
                                Direct::Seq(
                                    Box::new(Direct::SetLocal(
                                        l,
                                        Box::new(Direct::Const(Const::I32(0))),
                                    )),
                                    Box::new(Direct::Break),
                                ),
                                1,
                                false,
                            )))),
                        )),
                        Box::new(Direct::If {
                            cond: Box::new(Direct::GetLocal(l)),
                            t: Box::new(x),
                            f: Box::new(Direct::Nop),
                        }),
                    )
                }
            }
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
                    Direct::Seq(Box::new(Direct::If { cond, t, f }), Box::new(x))
                }
            }
            op => Direct::Seq(Box::new(op), Box::new(x)),
        }
    }

    fn base(self) -> Base {
        match self {
            Direct::Nop => Base::Nop,
            Direct::INumOp(w, op, a, b) => {
                Base::INumOp(w, op, Box::new(a.base()), Box::new(b.base()))
            }
            Direct::FNumOp(w, op, a, b) => {
                Base::FNumOp(w, op, Box::new(a.base()), Box::new(b.base()))
            }
            Direct::ICompOp(w, op, a, b) => {
                Base::ICompOp(w, op, Box::new(a.base()), Box::new(b.base()))
            }
            Direct::FCompOp(w, op, a, b) => {
                Base::FCompOp(w, op, Box::new(a.base()), Box::new(b.base()))
            }
            Direct::CvtOp(op, a) => Base::CvtOp(op, Box::new(a.base())),
            Direct::FUnOp(w, op, a) => Base::FUnOp(w, op, Box::new(a.base())),
            Direct::Const(c) => Base::Const(c),
            Direct::If { cond, t, f } => Base::If {
                cond: Box::new(cond.base()),
                t: Box::new(t.base()),
                f: Box::new(f.base()),
            },
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
            Direct::Store(t, a, b) => Base::Store(t, Box::new(a.base()), Box::new(b.base())),
            Direct::SetLocal(l, v) => Base::SetLocal(l, Box::new(v.base())),
            Direct::Load(t, p) => Base::Load(t, Box::new(p.base())),
            Direct::Break => Base::Break,
            Direct::Continue => Base::Continue,
            Direct::Return => Base::Return,
            Direct::Call(i, params) => {
                Base::Call(i, params.into_iter().map(|x| x.base()).collect())
            }
            // TODO - do we add Break at the end?
            Direct::Loop(a) => Base::Loop(Box::new(a.replace_br(Direct::Continue, 0, true).base())),
        }
    }
}

#[derive(Debug)]
pub struct Fun<T> {
    pub params: Vec<wasm::ValueType>,
    pub body: T,
    /// Return value
    /// Return type
    pub ty: Option<wasm::ValueType>,
}

pub fn test(w: &wasm::Module) {
    let d = direct(w);
    println!("{:?}", d);
    println!(
        "Base: {:#?}",
        d.into_iter()
            .map(|Fun { params, body, ty }| Fun {
                params,
                body: body.base(),
                ty,
            })
            .collect::<Vec<_>>()
    );
}

pub fn to_base(w: &wasm::Module) -> Vec<Fun<Base>> {
    let d = direct(w);
    // println!("Direct: {:#?}", d);
    // let b = d
    //     .into_iter()
    //     .map(|Fun { params, body }| Fun {
    //         params,
    //         body: body.base(),
    //     })
    //     .collect();
    // println!("Base: {:#?}", b);
    d.into_iter()
        .map(|Fun { params, body, ty }| Fun {
            params,
            body: body.base(),
            ty,
        })
        .collect()
}

fn direct(w: &wasm::Module) -> Vec<Fun<Direct>> {
    let imports = w
        .import_section()
        .map_or_else(Vec::new, |x| x.entries().to_vec());
    let mut globals: Vec<_> = imports
        .iter()
        .map(|x| x.external())
        .filter_map(|x| {
            if let wasm::External::Global(g) = x {
                Some(*g)
            } else {
                None
            }
        })
        .collect();
    globals.append(&mut w.global_section().map_or_else(Vec::new, |x| {
        x.entries().iter().map(|x| *x.global_type()).collect()
    }));

    let mut funs = Vec::new();
    for (fun, body) in w
        .function_section()
        .map(|x| x.entries().to_vec())
        .unwrap_or_else(Vec::new)
        .into_iter()
        .zip(
            w.code_section()
                .map(|x| x.bodies().to_vec())
                .unwrap_or_else(Vec::new),
        )
    {
        let mut stack = Vec::new();

        enum BlockTy {
            Block(Vec<Direct>),
            Loop(Vec<Direct>),
            If(Box<Direct>, Vec<Direct>),
            Else(Box<Direct>, Vec<Direct>, Vec<Direct>),
        }
        impl BlockTy {
            fn push(&mut self, op: Direct) {
                match self {
                    BlockTy::Block(v)
                    | BlockTy::Loop(v)
                    | BlockTy::If(_, v)
                    | BlockTy::Else(_, _, v) => {
                        v.push(op);
                    }
                }
            }

            fn op(self) -> Direct {
                fn fold(v: Vec<Direct>) -> Direct {
                    v.into_iter().fold(Direct::Nop, |acc, x| {
                        Direct::Seq(Box::new(acc), Box::new(x))
                    })
                }

                match self {
                    BlockTy::If(cond, v) => Direct::If {
                        cond,
                        t: Box::new(Direct::Label(Box::new(fold(v)))),
                        f: Box::new(Direct::Nop),
                    },
                    BlockTy::Else(cond, t, f) => Direct::If {
                        cond,
                        t: Box::new(Direct::Label(Box::new(fold(t)))),
                        f: Box::new(Direct::Label(Box::new(fold(f)))),
                    },
                    BlockTy::Loop(v) => Direct::Loop(Box::new(fold(v))),
                    BlockTy::Block(v) => Direct::Label(Box::new(fold(v))),
                }
            }
        }

        let mut blocks = vec![BlockTy::Block(Vec::new())];

        let ty = fun.type_ref();
        let wasm::Type::Function(ty) = &w.type_section().unwrap().types()[ty as usize];
        let params = ty.params().to_vec();
        let ret = ty.return_type();

        let code = body.code();

        let locals: Vec<_> = body
            .locals()
            .iter()
            .flat_map(|x| (0..x.count()).map(move |_| x.value_type()))
            .collect();
        let locals: Vec<_> = params.iter().cloned().chain(locals).collect();

        macro_rules! numop {
            ($w:ident, $op:ident) => {{
                // They're on the stack as [a, b], so pop b and then a
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(Direct::INumOp(
                    Width::$w,
                    INumOp::$op,
                    Box::new(a),
                    Box::new(b),
                ));
            }};
        }
        macro_rules! fnumop {
            ($w:ident, $op:ident) => {{
                // They're on the stack as [a, b], so pop b and then a
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(Direct::FNumOp(
                    Width::$w,
                    FNumOp::$op,
                    Box::new(a),
                    Box::new(b),
                ));
            }};
        }
        macro_rules! funop {
            ($w:ident, $op:ident) => {{
                let a = stack.pop().unwrap();
                stack.push(Direct::FUnOp(Width::$w, FUnOp::$op, Box::new(a)));
            }};
        }
        macro_rules! cvtop {
            ($op:ident) => {{
                let a = stack.pop().unwrap();
                stack.push(Direct::CvtOp(CvtOp::$op, Box::new(a)));
            }};
        }
        macro_rules! compop {
            ($w:ident, $op:ident) => {{
                // They're on the stack as [a, b], so pop b and then a
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(Direct::ICompOp(
                    Width::$w,
                    ICompOp::$op,
                    Box::new(a),
                    Box::new(b),
                ));
            }};
        }
        macro_rules! fcompop {
            ($w:ident, $op:ident) => {{
                // They're on the stack as [a, b], so pop b and then a
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(Direct::FCompOp(
                    Width::$w,
                    FCompOp::$op,
                    Box::new(a),
                    Box::new(b),
                ));
            }};
        }

        for op in code.elements() {
            use wasm::Instruction::*;
            match op {
                Call(i) => {
                    let f = &w.function_section().unwrap().entries()[*i as usize];
                    let wasm::Type::Function(f) =
                        &w.type_section().unwrap().types()[f.type_ref() as usize];
                    let mut params: Vec<_> =
                        f.params().iter().map(|_x| stack.pop().unwrap()).collect();
                    // The arguments are stored on the stack in reverse order
                    params.reverse();

                    // It only goes on the stack if it returned something
                    if f.return_type().is_some() {
                        stack.push(Direct::Call(*i, params))
                    } else {
                        blocks.last_mut().unwrap().push(Direct::Call(*i, params))
                    }
                }
                Br(i) => blocks.last_mut().unwrap().push(Direct::Br(*i)),
                BrIf(i) => {
                    let cond = stack.pop().unwrap();
                    blocks.last_mut().unwrap().push(Direct::If {
                        cond: Box::new(cond),
                        t: Box::new(Direct::Br(*i)),
                        f: Box::new(Direct::Nop),
                    })
                }
                Nop => (),
                SetLocal(u) => {
                    let val = stack.pop().unwrap();
                    let ty = locals[*u as usize];
                    blocks
                        .last_mut()
                        .unwrap()
                        .push(Direct::SetLocal(Local { ty, idx: *u }, Box::new(val)));
                }
                I32Load(_, _) => {
                    let val = stack.pop().unwrap();
                    stack.push(Direct::Load(wasm::ValueType::I32, Box::new(val)))
                }
                I32Store(_, _) => {
                    let val = stack.pop().unwrap();
                    let ptr = stack.pop().unwrap();
                    blocks.last_mut().unwrap().push(Direct::Store(
                        wasm::ValueType::I32,
                        Box::new(ptr),
                        Box::new(val),
                    ))
                }
                GetGlobal(idx) => stack.push(Direct::GetGlobal(Global {
                    ty: globals[*idx as usize],
                    idx: *idx,
                })),
                GetLocal(u) => {
                    let ty = locals[*u as usize];
                    stack.push(Direct::GetLocal(Local { ty, idx: *u }))
                }
                TeeLocal(u) => {
                    let val = stack.pop().unwrap().clone();
                    let ty = locals[*u as usize];
                    // This has a side effect but also returns something, and we need the side effect to get executed at the right time
                    // So we use a Seq instead of pushing the Set to blocks.last_mut()
                    stack.push(Direct::Seq(
                        Box::new(Direct::SetLocal(Local { ty, idx: *u }, Box::new(val))),
                        Box::new(Direct::GetLocal(Local { ty, idx: *u })),
                    ))
                }
                I32Const(i) => stack.push(Direct::Const(Const::I32(*i))),
                I64Const(i) => stack.push(Direct::Const(Const::I64(*i))),
                F32Const(i) => stack.push(Direct::Const(Const::F32(f32::from_bits(*i)))),
                F64Const(i) => stack.push(Direct::Const(Const::F64(f64::from_bits(*i)))),
                I32Add => numop!(W32, Add),
                I32Mul => numop!(W32, Mul),
                I32Sub => numop!(W32, Sub),
                I32DivS => numop!(W32, DivS),
                I32DivU => numop!(W32, DivU),
                I32Shl => numop!(W32, Shl),
                I32ShrS => numop!(W32, ShrS),
                I32ShrU => numop!(W32, ShrU),
                I32And => numop!(W32, And),
                I32Or => numop!(W32, Or),
                I32Xor => numop!(W32, Xor),
                I32Eq => compop!(W32, Eq),
                I32Ne => compop!(W32, NEq),
                I32LeU => compop!(W32, LeU),
                I32LeS => compop!(W32, LeS),
                I32GeU => compop!(W32, GeU),
                I32GeS => compop!(W32, GeS),
                I32LtU => compop!(W32, LtU),
                I32LtS => compop!(W32, LtS),
                I32GtU => compop!(W32, GtU),
                I32GtS => compop!(W32, GtS),

                I32Eqz => {
                    let a = stack.pop().unwrap();
                    let b = Direct::Const(Const::I32(0));
                    stack.push(Direct::ICompOp(
                        Width::W32,
                        ICompOp::Eq,
                        Box::new(a),
                        Box::new(b),
                    ))
                }

                F32Min => fnumop!(W32, Min),
                F32Max => fnumop!(W32, Max),
                F32Add => fnumop!(W32, Add),
                F32Sub => fnumop!(W32, Sub),
                F32Mul => fnumop!(W32, Mul),
                F32Div => fnumop!(W32, Div),
                F32Abs => funop!(W32, Abs),
                F32Neg => funop!(W32, Neg),
                F32Sqrt => funop!(W32, Sqrt),
                F32Ceil => funop!(W32, Ceil),
                F32Floor => funop!(W32, Floor),
                F32Gt => fcompop!(W32, Gt),
                F32Lt => fcompop!(W32, Lt),
                F32Ge => fcompop!(W32, Ge),
                F32Le => fcompop!(W32, Le),
                F32Eq => fcompop!(W32, Eq),
                F32Ne => fcompop!(W32, NEq),

                I32TruncSF32 => cvtop!(F32toI32S),
                I32TruncUF32 => cvtop!(F32toI32U),
                F32ConvertSI32 => cvtop!(I32toF32S),
                F32ConvertUI32 => cvtop!(I32toF32U),

                Loop(_ty) => blocks.push(BlockTy::Loop(Vec::new())),
                Block(_ty) => blocks.push(BlockTy::Block(Vec::new())),
                If(_ty) => {
                    let cond = stack.pop().unwrap();
                    blocks.push(BlockTy::If(Box::new(cond), Vec::new()));
                }
                Else => match blocks.pop().unwrap() {
                    BlockTy::If(cond, v) => blocks.push(BlockTy::Else(cond, v, Vec::new())),
                    _ => panic!("Else without if"),
                },
                End => {
                    if blocks.len() <= 1 {
                        break;
                    } else {
                        let b = blocks.pop().unwrap();
                        blocks.last_mut().unwrap().push(b.op());
                    }
                }
                Return => blocks.last_mut().unwrap().push(Direct::Return),
                x => panic!("Instruction {} not supported", x),
            }
        }

        assert_eq!(blocks.len(), 1);

        let mut body = blocks.pop().unwrap().op();

        if ret.is_none() {
            assert_eq!(stack.len(), 0, "Stuff left on stack: {:?}", stack);
        } else {
            assert_eq!(
                stack.len(),
                1,
                "Wrong number of things on stack: {:?}",
                stack
            );
            body = Direct::Seq(Box::new(body), Box::new(stack.pop().unwrap()));
        }

        funs.push(Fun {
            params,
            body,
            ty: ret,
        });
    }
    funs
}

use crate::*;
use rspirv::dr;
use spirv_headers as spvh;

trait S {
    type Value;
    type Ctx;
    fn spv(self, ctx: &mut Self::Ctx) -> Self::Value;
}

#[derive(Default)]
struct Types {
    b: Option<u32>,
    i_32: Option<u32>,
    s_32: Option<u32>,
    i_64: Option<u32>,
    s_64: Option<u32>,
    f_32: Option<u32>,
    f_64: Option<u32>,
}

struct Ctx {
    tys: Types,
    ptrs: Types,
    buffer: u32,
    thread_id: u32,
    locals: Vec<u32>,
    b: dr::Builder,
}
impl Ctx {
    fn new(b: dr::Builder) -> Self {
        Ctx {
            tys: Default::default(),
            ptrs: Default::default(),
            buffer: 0,
            thread_id: 0,
            locals: Vec::new(),
            b,
        }
    }

    fn fun(&mut self, f: ir::Fun<ir::Base>) {
        let ir::Fun {params, body} = f;
        let mut locals = body.locals();
        locals.sort_by_key(|x| x.idx);

        // TODO return type
        // TODO parameters
        let void = self.type_void();
        let t = self.type_function(void, []);
        self.begin_function(void, None, spvh::FunctionControl::NONE, t).unwrap();

        let locals = locals.into_iter().map(|x| {
            let ty = x.ty;
            let ty = self.get(ty);
            self.variable(ty, None, spvh::StorageClass::Function, None)
        }).collect();

        self.locals = locals;

        // TODO buffer
        // TODO thread_id
    }

    fn bool(&mut self) -> u32 {
        if let Some(i) = self.tys.b {
            i
        } else {
            let i = self.type_bool();
            self.tys.b = Some(i);
            i
        }
    }

    fn signed(&mut self, width: ir::Width) -> u32 {
        match width {
            ir::Width::W32 => if let Some(i) = self.tys.s_32 {
                i
            } else {
                let i = self.type_int(32, 1);
                self.tys.s_32 = Some(i);
                i
            }
            ir::Width::W64 => if let Some(i) = self.tys.s_64 {
                i
            } else {
                let i = self.type_int(64, 1);
                self.tys.s_64 = Some(i);
                i
            }
        }
    }

    fn int(&mut self, width: ir::Width) -> u32 {
        match width {
            ir::Width::W32 => self.get(wasm::ValueType::I32),
            ir::Width::W64 => self.get(wasm::ValueType::I64),
        }
    }

    fn ptr(&mut self, t: wasm::ValueType) -> u32 {
        match t {
            wasm::ValueType::I32 => if let Some(i) = self.ptrs.i_32 {
                i
            } else {
                let i = self.type_int(32, 0);
                self.ptrs.i_32 = Some(i);
                i
            }
            wasm::ValueType::I64 => if let Some(i) = self.ptrs.i_64 {
                i
            } else {
                let i = self.type_int(64, 0);
                self.ptrs.i_64 = Some(i);
                i
            }
            wasm::ValueType::F32 => if let Some(i) = self.ptrs.f_32 {
                i
            } else {
                let i = self.type_float(32);
                self.ptrs.f_32 = Some(i);
                i
            }
            wasm::ValueType::F64 => if let Some(i) = self.ptrs.f_64 {
                i
            } else {
                let i = self.type_float(64);
                self.ptrs.f_64 = Some(i);
                i
            }
        }
    }

    fn get(&mut self, t: wasm::ValueType) -> u32 {
        match t {
            wasm::ValueType::I32 => if let Some(i) = self.tys.i_32 {
                i
            } else {
                let i = self.type_int(32, 0);
                self.tys.i_32 = Some(i);
                i
            }
            wasm::ValueType::I64 => if let Some(i) = self.tys.i_64 {
                i
            } else {
                let i = self.type_int(64, 0);
                self.tys.i_64 = Some(i);
                i
            }
            wasm::ValueType::F32 => if let Some(i) = self.tys.f_32 {
                i
            } else {
                let i = self.type_float(32);
                self.tys.f_32 = Some(i);
                i
            }
            wasm::ValueType::F64 => if let Some(i) = self.tys.f_64 {
                i
            } else {
                let i = self.type_float(64);
                self.tys.f_64 = Some(i);
                i
            }
        }
    }
}
use std::ops::{Deref, DerefMut};
impl Deref for Ctx {
    type Target = dr::Builder;
    fn deref(&self) -> &dr::Builder {
        &self.b
    }
}
impl DerefMut for Ctx {
    fn deref_mut(&mut self) -> &mut dr::Builder {
        &mut self.b
    }
}

impl S for ir::Base {
    type Ctx = Ctx;
    type Value = u32;
    fn spv(self, ctx: &mut Ctx) -> u32 {
        match self {
            ir::Base::Nop => 0,
            ir::Base::INumOp(w, op, a, b) => {
                let a = a.spv(ctx);
                let b = b.spv(ctx);
                let ty = ctx.int(w);
                match op {
                    ir::INumOp::Mul => ctx.i_mul(ty, None, a, b).unwrap(),
                    ir::INumOp::Add => ctx.i_add(ty, None, a, b).unwrap(),
                    ir::INumOp::Sub => ctx.i_sub(ty, None, a, b).unwrap(),
                    ir::INumOp::DivU => ctx.u_div(ty, None, a, b).unwrap(),
                    ir::INumOp::DivS => {
                        let sty = ctx.signed(w);
                        let a = ctx.bitcast(sty, None, a).unwrap();
                        let b = ctx.bitcast(sty, None, b).unwrap();
                        ctx.s_div(sty, None, a, b).unwrap()
                    }
                }
            }
            ir::Base::ICompOp(w, op, a, b) => {
                let a = a.spv(ctx);
                let b = b.spv(ctx);
                let ty = ctx.int(w);
                // Unlike WASM, SPIR-V has booleans
                // So we convert them to integers immediately
                let t_bool = ctx.bool();

                let b = match op {
                    ir::ICompOp::Eq => ctx.i_equal(t_bool, None, a, b).unwrap(),
                    ir::ICompOp::NEq => ctx.i_not_equal(t_bool, None, a, b).unwrap(),
                    ir::ICompOp::LeU => ctx.u_less_than_equal(t_bool, None, a, b).unwrap(),
                    ir::ICompOp::GeU => ctx.u_greater_than_equal(t_bool, None, a, b).unwrap(),
                    ir::ICompOp::LeS => {
                        let sty = ctx.signed(w);
                        let a = ctx.bitcast(sty, None, a).unwrap();
                        let b = ctx.bitcast(sty, None, b).unwrap();
                        ctx.s_less_than_equal(sty, None, a, b).unwrap()
                    }
                    ir::ICompOp::GeS => {
                        let sty = ctx.signed(w);
                        let a = ctx.bitcast(sty, None, a).unwrap();
                        let b = ctx.bitcast(sty, None, b).unwrap();
                        ctx.s_greater_than_equal(sty, None, a, b).unwrap()
                    }
                };

                let zero = ctx.constant_u32(ty, 0);
                let one = ctx.constant_u32(ty, 1);
                ctx.select(ty, None, b, one, zero).unwrap()
            }
            ir::Base::Const(ir::Const::I32(i)) => {
                let ty = ctx.get(wasm::ValueType::I32);
                ctx.constant_u32(ty, unsafe { std::mem::transmute(i) })
            }
            ir::Base::Const(ir::Const::F32(i)) => {
                let ty = ctx.get(wasm::ValueType::F32);
                ctx.constant_f32(ty, i)
            }
            ir::Base::Const(_) => panic!("We currently don't support 64-bit constants"),
            ir::Base::Seq(a, b) => {
                a.spv(ctx);
                b.spv(ctx)
            }
            ir::Base::GetLocal(l) => {
                let ty = ctx.get(l.ty);
                let l = ctx.locals[l.idx as usize];
                ctx.load(ty, None, l, None, []).unwrap()
            }
            ir::Base::SetLocal(l, val) => {
                let l = ctx.locals[l.idx as usize];
                let val = val.spv(ctx);
                ctx.store(l, val, None, []).unwrap();
                0
            }
            ir::Base::GetGlobal(g) => {
                assert_eq!(g, ir::Global { ty: wasm::GlobalType::new(wasm::ValueType::I32, false), idx: 0 });
                ctx.thread_id
            }
            ir::Base::Load(ty, ptr) => {
                let ptr_ty = ctx.ptr(ty);
                let ty = ctx.get(ty);
                let ptr = ptr.spv(ctx);
                let buf = ctx.buffer;
                let ptr = ctx.access_chain(ptr_ty, None, buf, [ptr]).unwrap();
                ctx.load(ty, None, ptr, None, []).unwrap()
            }
            ir::Base::Store(ty, ptr, val) => {
                let val = val.spv(ctx);
                let ptr_ty = ctx.ptr(ty);
                let ptr = ptr.spv(ctx);
                let buf = ctx.buffer;
                let ptr = ctx.access_chain(ptr_ty, None, buf, [ptr]).unwrap();
                ctx.store(ptr, val, None, []).unwrap();
                0
            }
            ir::Base::If { cond, t, f } => {
                let l_t = ctx.id();
                let l_f = ctx.id();
                let l_m = ctx.id();

                let cond = cond.spv(ctx);

                ctx.selection_merge(l_m, spvh::SelectionControl::NONE).unwrap();
                ctx.branch_conditional(cond, l_t, l_f, []).unwrap();
                ctx.begin_basic_block(Some(l_t)).unwrap();
                t.spv(ctx);
                ctx.branch(l_m).unwrap();
                ctx.begin_basic_block(Some(l_f)).unwrap();
                f.spv(ctx);
                ctx.branch(l_m).unwrap();
                0
            },
        }
    }
}

fn test(f: ir::Fun<ir::Base>) {
    let params = f.params;
    let body = f.body;
}

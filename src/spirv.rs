use rspirv::binary::*;
use rspirv::dr::*;
use spirv_headers as spvh;

use std::collections::HashMap;
struct SBuilder {
    b: Builder,
    locals: Vec<spvh::Word>,
    table: HashMap<SType, spvh::Word>,
    buffer: spvh::Word,
    idx: spvh::Word,
}
impl SBuilder {
    fn new(b: Builder) -> SBuilder {
        SBuilder {
            b,
            locals: Vec::new(),
            table: HashMap::new(),
            buffer: 0,
            idx: 0,
        }
    }
    fn ty(&mut self, t: SType) -> spvh::Word {
        t.spirv(self)
    }
}
impl std::ops::Deref for SBuilder {
    type Target = Builder;
    fn deref(&self) -> &Builder {
        &self.b
    }
}
impl std::ops::DerefMut for SBuilder {
    fn deref_mut(&mut self) -> &mut Builder {
        &mut self.b
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum SType {
    Bool,
    Uint,
    Float,
    Ptr(Box<SType>, spvh::StorageClass),
    Struct(Vec<SType>),
    RuntimeArray(Box<SType>),
    Vector(Box<SType>, u32),
    None,
}
impl SType {
    fn spirv(&self, b: &mut SBuilder) -> spvh::Word {
        if let Some(t) = b.table.get(self) {
            return *t;
        }
        let t = match self {
            SType::Bool => b.type_bool(),
            SType::Ptr(x, c) => {
                let x = x.spirv(b);
                b.type_pointer(None, *c, x)
            }
            SType::Uint => b.type_int(32, 0),
            SType::Float => b.type_float(32),
            SType::Struct(v) => {
                let mut v2 = Vec::with_capacity(v.len());
                for i in v {
                    v2.push(i.spirv(b));
                }
                b.type_struct(v2)
            }
            SType::RuntimeArray(x) => {
                let x = x.spirv(b);
                b.type_runtime_array(x)
            }
            SType::Vector(x, n) => {
                let x = x.spirv(b);
                b.type_vector(x, *n)
            }
            SType::None => 0,
        };
        b.table.insert(self.clone(), t);
        t
    }
}

#[derive(Clone, Debug)]
pub struct Value {
    val: spvh::Word,
    ty: SType,
}
impl std::ops::Deref for Value {
    type Target = spvh::Word;
    fn deref(&self) -> &spvh::Word {
        &self.val
    }
}

impl Into<SType> for WasmTy {
    fn into(self) -> SType {
        match self {
            // We don't actually support 64-bit ints or floats
            WasmTy::I32 => SType::Uint,
            WasmTy::I64 => SType::Uint,
            WasmTy::F32 => SType::Float,
            WasmTy::F64 => SType::Float,
        }
    }
}

use crate::*;

#[derive(Debug)]
enum BlockData {
    Loop {
        head: u32,
        cont: u32,
        end: u32,
    },
    If {
        t: u32,
        f: u32,
        end: u32,
    },
    /// A block that must branch to 'end' after completing 'block'
    Shim {
        block: Box<BlockData>,
        end: u32,
    },
}

impl Visitor for SBuilder {
    type Output = Value;
    type BlockData = BlockData;

    fn br_continue(&mut self, blocks: &mut [Self::BlockData]) {
        match blocks.last().unwrap() {
            BlockData::Loop { cont, .. } => {
                self.branch(*cont).unwrap();
                // Useless basic block that will never be executed
                self.begin_basic_block(None).unwrap();
            }
            _ => panic!("Non loop continue"),
        }
    }

    fn br_break(&mut self, blocks: &mut [BlockData]) {
        let mut last_end = None;
        for i in 0..blocks.len() {
            let end = match &mut blocks[i] {
                BlockData::Loop { end, .. } => end,
                BlockData::If { end, .. } => end,
                BlockData::Shim { end, .. } => end,
            };
            let end2 = *end;

            if let Some(last_end) = last_end {
                // This block has a child, so we change the parent's end block to the child's old end block
                //  and redirect it with a shim to our old end block.
                //
                // When the child block finishes, it will branch to our end block, which is now
                //  the block the child declared as its merge block. Then, it will branch to the
                //  block that the parent declared as its merge block before continuing. So,
                //  both blocks fulfill their promises
                let block = &mut blocks[i];
                let mut block2 = BlockData::If { t: 0, f: 0, end: 0 };
                std::mem::swap(block, &mut block2);
                *block = match block2 {
                    BlockData::Loop { end, head, cont } => BlockData::Shim {
                        block: Box::new(BlockData::Loop {
                            head,
                            cont,
                            end: last_end,
                        }),
                        end,
                    },
                    BlockData::If { t, f, end } => BlockData::Shim {
                        block: Box::new(BlockData::If {
                            t,
                            f,
                            end: last_end,
                        }),
                        end,
                    },
                    BlockData::Shim { block, end } => BlockData::Shim {
                        block: Box::new(BlockData::Shim {
                            block,
                            end: last_end,
                        }),
                        end,
                    },
                };
            } else {
                // This is the innermost block, so branch to the old 'end' and set the new end to a new block
                self.branch(*end).unwrap();
                *end = self.id();
                self.begin_basic_block(None).unwrap();
            }
            last_end = Some(end2);
        }
    }

    fn start_block(&mut self, op: BlockOp<Value>) -> BlockData {
        match op {
            BlockOp::Loop => {
                let head = self.id();
                let cont = self.id();
                let end = self.id();
                let body = self.id();

                self.branch(head).unwrap();
                self.begin_basic_block(Some(head)).unwrap();
                self.loop_merge(end, cont, spvh::LoopControl::NONE, [])
                    .unwrap();
                self.branch(body).unwrap();
                self.begin_basic_block(Some(body)).unwrap();

                BlockData::Loop { head, end, cont }
            }
            BlockOp::If(cond) => {
                let uint = self.ty(SType::Uint);
                let t_bool = self.ty(SType::Bool);
                // `cond` is an i32, so we branch if it isn't zero
                let zero = self.constant_u32(uint, 0);
                let b = self.i_equal(t_bool, None, **cond, zero).unwrap();

                let t_lbl = self.id();
                let f_lbl = self.id();
                let end_lbl = self.id();
                self.selection_merge(end_lbl, spvh::SelectionControl::NONE)
                    .unwrap();
                // b is `cond == 0`, so we swap true and false here
                self.branch_conditional(b, f_lbl, t_lbl, []).unwrap();
                self.begin_basic_block(Some(t_lbl)).unwrap();
                BlockData::If {
                    t: t_lbl,
                    f: f_lbl,
                    end: end_lbl,
                }
            }
        }
    }

    fn else_block(&mut self, data: BlockData) -> BlockData {
        match data {
            BlockData::If { end, f, t } => {
                self.branch(end).unwrap();
                self.begin_basic_block(Some(f)).unwrap();
                data
            }
            BlockData::Shim { block, end } => {
                let block = self.else_block(*block);
                BlockData::Shim {
                    block: Box::new(block),
                    end,
                }
            }
            BlockData::Loop { .. } => panic!("Else blocks not allowed for loops!"),
        }
    }

    fn end_block(&mut self, data: BlockData) {
        match data {
            BlockData::If { end, .. } => {
                self.branch(end).unwrap();
                self.begin_basic_block(Some(end)).unwrap();
            }
            BlockData::Shim { block, end } => {
                self.end_block(*block);
                self.branch(end).unwrap();
                self.begin_basic_block(Some(end)).unwrap();
            }
            BlockData::Loop { end, cont, head } => {
                // For WASM loops, the default behaviour is to break out of a loop at the end
                self.branch(end).unwrap();

                // SPIR-V requires the continue block to be after the rest of the loop
                self.begin_basic_block(Some(cont)).unwrap();
                self.branch(head).unwrap();

                self.begin_basic_block(Some(end)).unwrap();
            }
        }
    }

    fn add_local(&mut self, ty: WasmTy, val: Option<Value>) {
        let var_ty = SType::Ptr(Box::new(ty.into()), spvh::StorageClass::Function);
        let var_ty = self.ty(var_ty);
        let var = self.variable(var_ty, None, spvh::StorageClass::Function, None);
        if let Some(val) = val {
            self.store(var, *val, None, []).unwrap();
        }
        self.locals.push(var);
    }

    fn init(&mut self) {
        let uint = self.ty(SType::Uint);
        let ptr_uint_i = self.ty(SType::Ptr(Box::new(SType::Uint), spvh::StorageClass::Input));
        let const_0 = self.constant_u32(uint, 0);
        let idx = self.idx;
        let idx = self.access_chain(ptr_uint_i, None, idx, [const_0]).unwrap();
        self.idx = self.load(uint, None, idx, None, []).unwrap();
    }

    fn visit(&mut self, op: AOp<Value>) -> Value {
        use AOp::*;

        let uint = SType::Uint.spirv(self);

        let (v, t) = match op {
            Return => {
                self.ret().unwrap();
                // Unreachable block
                self.begin_basic_block(None).unwrap();
                (0, SType::None)
            }
            GetGlobalImport(module, field) => {
                if module != "spv" {
                    panic!("Unknown namespace {}", module);
                }
                match &*field {
                    "id" => (self.idx, SType::Uint),
                    _ => panic!("Unknown global {}", field),
                }
            }
            GetLocal(l) => {
                let l = self.locals[l as usize];
                let r = self.load(uint, None, l, None, []).unwrap();
                (r, SType::Uint)
            }
            SetLocal(l, v) => {
                let l = self.locals[l as usize];
                self.store(l, *v.val, None, []).unwrap();
                (0, SType::None)
            }
            LeU(a, b) => {
                // Unlike WASM, SPIR-V has booleans
                // So we convert them to integers immediately
                let t_bool = self.ty(SType::Bool);
                let b = self.u_less_than_equal(t_bool, None, **a, **b).unwrap();
                let zero = self.constant_u32(uint, 0);
                let one = self.constant_u32(uint, 1);
                let r = self.select(uint, None, b, one, zero).unwrap();
                (r, SType::Uint)
            }
            Eq(a, b) => {
                // Unlike WASM, SPIR-V has booleans
                // So we convert them to integers immediately
                let t_bool = self.ty(SType::Bool);
                let b = self.i_equal(t_bool, None, **a, **b).unwrap();
                let zero = self.constant_u32(uint, 0);
                let one = self.constant_u32(uint, 1);
                let r = self.select(uint, None, b, one, zero).unwrap();
                (r, SType::Uint)
            }
            Mul(a, b) => (self.i_mul(uint, None, **a, **b).unwrap(), SType::Uint),
            Add(a, b) => (self.i_add(uint, None, **a, **b).unwrap(), SType::Uint),
            I32Const(x) => (self.constant_u32(uint, x), SType::Uint),
            Load(x) => {
                let ptr = self.buffer;
                let p_uint_u = self.ty(SType::Ptr(
                    Box::new(SType::Uint),
                    spvh::StorageClass::Uniform,
                ));
                let c0 = self.constant_u32(uint, 0);
                let c4 = self.constant_u32(uint, 4);
                let x = self.u_div(uint, None, *x.val, c4).unwrap();
                let ptr = self.access_chain(p_uint_u, None, ptr, [c0, x]).unwrap();
                let r = self.load(uint, None, ptr, None, []).unwrap();
                (r, SType::Uint)
            }
            Store(p, x) => {
                let ptr = self.buffer;
                let p_uint_u = self.ty(SType::Ptr(
                    Box::new(SType::Uint),
                    spvh::StorageClass::Uniform,
                ));
                let c0 = self.constant_u32(uint, 0);
                let c4 = self.constant_u32(uint, 4);
                let p = self.u_div(uint, None, *p.val, c4).unwrap();
                let ptr = self.access_chain(p_uint_u, None, ptr, [c0, p]).unwrap();
                self.store(ptr, *x.val, None, []).unwrap();
                (0, SType::None)
            }
        };
        Value { val: v, ty: t }
    }
}

pub fn to_spirv(w: wasm::Module) -> Vec<u8> {
    let main_idx = w.start_section().expect("No 'start' function in module!");

    let mut b = Builder::new();
    b.set_version(1, 0);
    b.capability(spvh::Capability::Shader);
    b.ext_inst_import("GLSL.std.450");
    b.memory_model(spvh::AddressingModel::Logical, spvh::MemoryModel::GLSL450);

    let mut b = SBuilder::new(b);

    let uint3 = b.ty(SType::Vector(Box::new(SType::Uint), 3));
    let p_i_v3 = b.type_pointer(None, spvh::StorageClass::Input, uint3);
    let array = b.ty(SType::RuntimeArray(Box::new(SType::Uint)));
    b.decorate(
        array,
        spvh::Decoration::ArrayStride,
        [Operand::LiteralInt32(4)],
    );
    let data_t = b.ty(SType::Struct(vec![SType::RuntimeArray(Box::new(
        SType::Uint,
    ))]));
    b.member_decorate(
        data_t,
        0,
        spvh::Decoration::Offset,
        [Operand::LiteralInt32(0)],
    );
    let ptr_data_t = b.type_pointer(None, spvh::StorageClass::Uniform, data_t);
    let data = b.variable(ptr_data_t, None, spvh::StorageClass::Uniform, None);

    b.buffer = data;

    b.decorate(data_t, spvh::Decoration::BufferBlock, []);
    b.decorate(
        data,
        spvh::Decoration::DescriptorSet,
        [Operand::LiteralInt32(0)],
    );
    b.decorate(data, spvh::Decoration::Binding, [Operand::LiteralInt32(0)]);

    let id = b.variable(p_i_v3, None, spvh::StorageClass::Input, None);
    b.idx = id;
    b.decorate(
        id,
        spvh::Decoration::BuiltIn,
        [Operand::BuiltIn(spvh::BuiltIn::GlobalInvocationId)],
    );

    // let const_64 = b.constant_u32(uint, 64);
    // let const_1 = b.constant_u32(uint, 1);
    // let workgroup = b.constant_composite(uint3, [const_64, const_1, const_1]);
    // b.decorate(workgroup, spvh::Decoration::BuiltIn,b.load(uint, None, id2, None, []).unwrap(); [Operand::BuiltIn(spvh::BuiltIn::WorkgroupSize)]);

    let void = b.type_void();
    let voidf = b.type_function(void, vec![]);
    let main = b
        .begin_function(void, None, spvh::FunctionControl::NONE, voidf)
        .unwrap();
    b.begin_basic_block(None).unwrap();

    // let uint = b.ty(SType::Uint);
    // let ptr_uint_i = b.ty(SType::Ptr(Box::new(SType::Uint), spvh::StorageClass::Input));
    // let ptr_uint_f = b.ty(SType::Ptr(
    //     Box::new(SType::Uint),
    //     spvh::StorageClass::Function,
    // ));
    // let ptr_uint_u = b.ty(SType::Ptr(
    //     Box::new(SType::Uint),
    //     spvh::StorageClass::Uniform,
    // ));
    // let const_0 = b.constant_u32(uint, 0);
    // let id2 = b.load(uint, None, id2, None, []).unwrap();

    // let slot = b
    //     .access_chain(ptr_uint_u, None, data, [const_0, id2])
    //     .unwrap();
    // let slot_val = b.load(uint, None, slot, None, []).unwrap();
    //
    // // Process for using locals:
    // // Declaration of a local (at function start):
    // let v = b.variable(ptr_uint_f, None, spvh::StorageClass::Function, None);
    // // local.set:
    // b.store(v, slot_val, None, []).unwrap();
    // // local.get
    // let val = b.load(uint, None, v, None, []).unwrap();
    //
    // let const_12 = b.constant_u32(uint, 12);
    // let val = b.i_mul(uint, None, val, const_12).unwrap();
    // let const_3 = b.constant_u32(uint, 3);
    // let val = b.i_add(uint, None, val, const_3).unwrap();
    // b.store(v, val, None, []).unwrap();
    // let val = b.load(uint, None, v, None, []).unwrap();
    // b.store(slot, val, None, []).unwrap();

    let mut am = AM::from_move(w);
    am.visit(
        main_idx,
        Vec::new(),
        // vec![TVal {
        //     ty: WasmTy::I32,
        //     val: Value {
        //         ty: SType::Uint,
        //         val: id2,
        //     },
        // }],
        &mut b,
    )
    .unwrap();

    b.ret().unwrap();
    b.end_function().unwrap();

    b.entry_point(spvh::ExecutionModel::GLCompute, main, "main", [id]);
    b.execution_mode(main, spvh::ExecutionMode::LocalSize, [64, 1, 1]);

    let m = b.b.module();
    // println!("{}", m.disassemble());
    let mut spv = m.assemble();
    // TODO: test this on a big-endian system
    for i in spv.iter_mut() {
        *i = i.to_le()
    }
    let spv: &[u8] = unsafe { spv.align_to().1 };
    spv.to_vec()
}

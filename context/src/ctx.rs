use std::{marker::PhantomData, num::NonZeroU16};

use init::Ctor;

mod alloc_ctx;
pub use alloc_ctx::AllocContext;

mod types_ctx;
pub use types_ctx::TypeContext;

mod value_ctx;
pub use value_ctx::ValueContext;

use crate::{types, value};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Invariant<'a>(PhantomData<*mut &'a ()>);

pub(crate) struct ContextInfo<'ctx> {
    alloc: alloc_ctx::AllocContextInfo<'ctx>,
    ty: types_ctx::TypeContextInfo<'ctx>,
    value: value_ctx::ValueContextInfo<'ctx>,
    target: Target,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Context<'ctx> {
    pub(crate) info: &'ctx ContextInfo<'ctx>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ContextRef<'a>(Invariant<'a>);

impl core::fmt::Debug for ContextRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContextRef")
    }
}

impl<'ctx> Context<'ctx> {
    pub fn with<R>(target: Target, f: impl FnOnce(Context<'_>) -> R) -> R {
        init::stack_init(ContextBuilder { target }, |x| {
            f(Context {
                info: unsafe { &*x.as_ptr() },
            })
        })
    }

    #[inline]
    #[must_use]
    pub fn alloc(&self) -> AllocContext<'ctx> {
        AllocContext {
            info: &self.info.alloc,
        }
    }

    #[inline]
    #[must_use]
    pub fn ty(&self) -> TypeContext<'ctx> {
        TypeContext {
            info: &self.info.ty,
        }
    }

    #[inline]
    #[must_use]
    pub fn value(&self) -> ValueContext<'ctx> {
        ValueContext {
            info: &self.info.value,
        }
    }
}

macro_rules! getters {
    (
        $($name:ident: $ty:ident)*
    ) => {$(
        #[inline]
        #[must_use]
        pub fn $name(self) -> crate::types::$ty<'ctx> {
            self.ty().$name()
        }
    )*};
}

impl<'ctx> Context<'ctx> {
    getters! {
        unit: UnitTy
        i1: IntegerTy
        i8: IntegerTy
        i16: IntegerTy
        i32: IntegerTy
        i64: IntegerTy
        i128: IntegerTy
        isize: IntegerTy
        iptr: IntegerTy

        f16: FloatTy
        f32: FloatTy
        f64: FloatTy

        ptr_ty: PointerTy
    }

    #[inline]
    pub fn int_ty(self, bits: NonZeroU16) -> types::IntegerTy<'ctx> {
        self.ty().int(self.alloc(), bits)
    }

    #[inline]
    pub fn int_ty_lit(self, bits: u16) -> types::IntegerTy<'ctx> {
        self.int_ty(NonZeroU16::new(bits).unwrap())
    }

    #[inline]
    pub fn ptr_ty_at(self, address_space: types::AddressSpace) -> types::PointerTy<'ctx> {
        self.ty().ptr_at(self.alloc(), address_space)
    }

    #[inline]
    pub fn function_ty(
        self,
        output_ty: impl Into<types::Type<'ctx>>,
        arguments: &[types::Type<'ctx>],
    ) -> types::FunctionTy<'ctx> {
        self.ty()
            .function(self.alloc(), output_ty.into(), arguments)
    }

    #[inline]
    pub fn array_ty(self, len: u64, item_ty: impl Into<types::Type<'ctx>>) -> types::ArrayTy<'ctx> {
        self.ty().array(self.alloc(), len, item_ty.into())
    }

    #[inline]
    pub fn struct_ty<I: IntoIterator>(
        self,
        name: impl crate::name::Name,
        flags: types::StructFlags,
        field_tys: &[types::Type<'ctx>],
    ) -> types::StructTy<'ctx>
    where
        I::Item: Into<types::Type<'ctx>>,
        I::IntoIter: ExactSizeIterator,
    {
        self.ty()
            .struct_ty(self.alloc(), name.to_name(), flags, field_tys)
    }
}

impl<'ctx> Context<'ctx> {
    pub fn const_int(
        self,
        ty: types::IntegerTy<'ctx>,
        value: &'ctx rug::Integer,
        signed: bool,
    ) -> Option<value::ConstInt<'ctx>> {
        self.value().const_int(self.alloc(), ty, value, signed)
    }
}

#[derive(Debug, Clone)]
pub struct Target {
    pub ptr_diff_bits: PtrBits,
    pub ptr_size_bits: PtrBits,
}

#[repr(u16)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PtrBits {
    _8 = 8,
    _16 = 16,
    _32 = 32,
    _64 = 64,
    _128 = 128,
}

struct ContextBuilder {
    target: Target,
}

impl<'ctx> Ctor<ContextBuilder> for ContextInfo<'ctx> {
    fn init(uninit: init::Uninit<'_, Self>, builder: ContextBuilder) -> init::Init<'_, Self> {
        init::init_struct! {
            uninit => Self {
                alloc: (),
                ty: types_ctx::TypeContextBuilder {
                    alloc: AllocContext {
                        info: unsafe{ &*alloc.as_ptr() },
                    },
                    target: &builder.target,
                },
                value: (),
                target: init::ctor(|uninit| uninit.write(builder.target))
            }
        }
    }
}

#[test]
fn test() {
    let target = Target {
        ptr_diff_bits: PtrBits::_32,
        ptr_size_bits: PtrBits::_32,
    };

    Context::with(target, |ctx| {
        let _ = ctx.ty().unit();
        assert_eq!(ctx.ptr_ty(), ctx.ptr_ty());
        assert_eq!(ctx.int_ty_lit(9), ctx.int_ty_lit(9));
        assert_ne!(ctx.int_ty_lit(9), ctx.int_ty_lit(10));

        assert_eq!(
            ctx.function_ty(ctx.iptr(), &[ctx.unit().erase()]),
            ctx.function_ty(ctx.i32(), &[ctx.unit().erase()]),
        );

        let four = ctx.value().intern_i32(4);
        ctx.const_int(ctx.int_ty_lit(3), four, true).unwrap();
    });
}

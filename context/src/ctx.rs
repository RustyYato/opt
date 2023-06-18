use std::{marker::PhantomData, num::NonZeroU16};

use init::Ctor;

mod alloc_ctx;
pub use alloc_ctx::AllocContext;

mod types_ctx;
pub use types_ctx::TypeContext;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Invariant<'a>(PhantomData<fn() -> *mut &'a ()>);

pub(crate) struct ContextInfo<'ctx> {
    alloc: alloc_ctx::AllocContextInfo<'ctx>,
    ty: types_ctx::TypeContextInfo<'ctx>,
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
        unit: Unit
        i1: Integer
        i8: Integer
        i16: Integer
        i32: Integer
        i64: Integer
        i128: Integer
        isize: Integer
        iptr: Integer
    }

    pub fn int(self, bits: NonZeroU16) -> crate::types::Integer<'ctx> {
        self.ty().int(self.alloc(), bits)
    }

    pub fn ptr(
        self,
        target_ty: impl Into<crate::types::Type<'ctx>>,
    ) -> crate::types::Pointer<'ctx> {
        self.ty().ptr(self.alloc(), target_ty.into())
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
        assert_eq!(ctx.ptr(ctx.i8()), ctx.ptr(ctx.i8()));
    });
}

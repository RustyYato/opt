use std::alloc::Layout;

use init::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    Ctor,
};

use crate::ctx::AllocContext;

use super::{
    raw_type::{TypeInfo, TypeTag},
    Ty, Type,
};

#[repr(C)]
#[non_exhaustive]
#[derive(PartialEq, Eq, Hash)]
pub struct FunctionInfo<'ctx> {
    output_ty: Type<'ctx>,
    arguments_tys: [Type<'ctx>],
}

impl core::fmt::Debug for FunctionInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct FmtArguments<'a, 'ctx>(&'a [Type<'ctx>]);

        impl core::fmt::Debug for FmtArguments<'_, '_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "fn(")?;
                for (i, arg) in self.0.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?
                    }
                    write!(f, "{:?}", arg)?
                }
                write!(f, ")")
            }
        }

        if self.output_ty.tag() == TypeTag::Unit {
            write!(f, "{:?}", FmtArguments(&self.arguments_tys))
        } else {
            write!(
                f,
                "{:?} -> {:?}",
                FmtArguments(&self.arguments_tys),
                self.output_ty
            )
        }
    }
}

pub type FunctionTy<'ctx> = Ty<'ctx, FunctionInfo<'ctx>>;

unsafe impl<'ctx> TypeInfo<'ctx> for FunctionInfo<'ctx> {
    const TAG: TypeTag = TypeTag::Function;
    type Flags = ();

    type Key<'a> = FunctionInit<'ctx, 'a> where 'ctx: 'a;

    #[inline]
    fn key<'a>(&'ctx self, (): Self::Flags) -> Self::Key<'a>
    where
        'ctx: 'a,
    {
        FunctionInit {
            output_ty: self.output_ty,
            arguments: &self.arguments_tys,
        }
    }

    fn create_from_key<'a>(alloc: AllocContext<'ctx>, key: Self::Key<'a>) -> Ty<'ctx, Self>
    where
        'ctx: 'a,
    {
        Ty::create_in_place(alloc, key, ())
    }
}

impl<'ctx> FunctionTy<'ctx> {
    #[inline]
    pub fn output_ty(self) -> Type<'ctx> {
        self.info().output_ty
    }

    #[inline]
    pub fn arguments_tys(self) -> &'ctx [Type<'ctx>] {
        &self.info().arguments_tys
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionInit<'ctx, 'a> {
    pub(crate) output_ty: Type<'ctx>,
    pub(crate) arguments: &'a [Type<'ctx>],
}

impl<'ctx> Ctor<FunctionInit<'ctx, '_>> for FunctionInfo<'ctx> {
    fn init<'a>(
        uninit: init::Uninit<'a, Self>,
        args: FunctionInit<'ctx, '_>,
    ) -> init::Init<'a, Self> {
        init::init_struct! {
            uninit => Self {
                output_ty: args.output_ty,
                arguments_tys: args.arguments,
            }
        }
    }
}

impl<'ctx> HasLayoutProvider<FunctionInit<'ctx, '_>> for FunctionInfo<'ctx> {
    type LayoutProvider = FunctionInfoLayoutProvider;
}

pub struct FunctionInfoLayoutProvider;

unsafe impl<'ctx> LayoutProvider<FunctionInfo<'ctx>, FunctionInit<'ctx, '_>>
    for FunctionInfoLayoutProvider
{
    fn layout_of(args: &FunctionInit<'ctx, '_>) -> Option<std::alloc::Layout> {
        Some(
            Layout::new::<Type>()
                .extend(Layout::array::<Type>(args.arguments.len()).ok()?)
                .ok()?
                .0,
        )
    }

    unsafe fn cast(
        ptr: std::ptr::NonNull<u8>,
        args: &FunctionInit<'ctx, '_>,
    ) -> std::ptr::NonNull<FunctionInfo<'ctx>> {
        std::ptr::NonNull::from_raw_parts(ptr.cast(), args.arguments.len())
    }
}

use std::alloc::Layout;

use init::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    try_ctor::{of_ctor, of_ctor_any_err},
    TryCtor,
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

pub type Function<'ctx> = Ty<'ctx, FunctionInfo<'ctx>>;

unsafe impl TypeInfo for FunctionInfo<'_> {
    const TAG: TypeTag = TypeTag::Function;
}

impl<'ctx> Function<'ctx> {
    #[must_use]
    pub(crate) fn create<I: ExactSizeIterator<Item = Type<'ctx>>>(
        ctx: AllocContext<'ctx>,
        output_ty: Type<'ctx>,
        arguments: I,
    ) -> Self {
        let args_len = arguments.len();
        match Ty::try_create_in_place(
            ctx,
            FunctionInit {
                output_ty,
                args_len,
                arguments,
            },
        ) {
            Ok(ty) => ty,
            Err(init::try_slice::IterInitError::NotEnoughItems) => {
                fn not_enough_items(args_len: usize) -> ! {
                    panic!("Arguments iterator didn't produce {args_len} items")
                }

                not_enough_items(args_len)
            }
            Err(init::try_slice::IterInitError::InitError(inf)) => match inf {},
        }
    }

    #[inline]
    pub fn output_ty(self) -> Type<'ctx> {
        self.info().output_ty
    }

    #[inline]
    pub fn arguments_tys(self) -> &'ctx [Type<'ctx>] {
        &self.info().arguments_tys
    }
}

struct FunctionInit<'ctx, I> {
    output_ty: Type<'ctx>,
    args_len: usize,
    arguments: I,
}

impl<'ctx, I: Iterator<Item = Type<'ctx>>> TryCtor<FunctionInit<'ctx, I>> for FunctionInfo<'ctx> {
    type Error = init::try_slice::IterInitError<core::convert::Infallible>;

    fn try_init<'a>(
        uninit: init::Uninit<'a, Self>,
        args: FunctionInit<'ctx, I>,
    ) -> Result<init::Init<'a, Self>, Self::Error> {
        Ok(init::try_init_struct! {
            uninit => Self {
                output_ty: of_ctor_any_err(args.output_ty),
                arguments_tys: init::try_slice::IterInit(args.arguments.map(of_ctor))
            }
        })
    }
}

impl<'ctx, I> HasLayoutProvider<FunctionInit<'ctx, I>> for FunctionInfo<'ctx> {
    type LayoutProvider = FunctionInfoLayoutProvider;
}

struct FunctionInfoLayoutProvider;

unsafe impl<'ctx, I> LayoutProvider<FunctionInfo<'ctx>, FunctionInit<'ctx, I>>
    for FunctionInfoLayoutProvider
{
    fn layout_of(args: &FunctionInit<'ctx, I>) -> Option<std::alloc::Layout> {
        Some(
            Layout::new::<Type>()
                .extend(Layout::array::<Type>(args.args_len).ok()?)
                .ok()?
                .0,
        )
    }

    unsafe fn cast(
        ptr: std::ptr::NonNull<u8>,
        args: &FunctionInit<'ctx, I>,
    ) -> std::ptr::NonNull<FunctionInfo<'ctx>> {
        std::ptr::NonNull::from_raw_parts(ptr.cast(), args.args_len)
    }
}

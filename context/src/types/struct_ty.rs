use std::{
    alloc::Layout,
    ops::{BitAnd, BitOr},
};

use init::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    try_ctor::of_ctor,
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
pub struct StructInfo<'ctx> {
    name: Option<istr::IStr>,
    field_tys: [Type<'ctx>],
}

impl core::fmt::Debug for StructInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct FmtFields<'a, 'ctx>(&'a [Type<'ctx>]);

        impl core::fmt::Debug for FmtFields<'_, '_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "[")?;
                for (i, arg) in self.0.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?
                    }
                    write!(f, "{:?}", arg)?
                }
                write!(f, "]")
            }
        }

        if let Some(name) = self.name {
            write!(f, ".{name} ")?
        }

        write!(f, "{:?}", FmtFields(&self.field_tys))
    }
}

pub type Struct<'ctx> = Ty<'ctx, StructInfo<'ctx>>;

unsafe impl TypeInfo for StructInfo<'_> {
    const TAG: TypeTag = TypeTag::Struct;
    type Flags = StructFlags;
}

#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructFlags(u16);

impl BitOr for StructFlags {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for StructFlags {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl StructFlags {
    #[inline(always)]
    pub fn any(self) -> bool {
        self.0 != 0
    }

    pub fn packed(self) -> bool {
        (self & Struct::PACKED).any()
    }

    pub fn opaque(self) -> bool {
        (self & Struct::OPAQUE).any()
    }

    pub fn literal(self) -> bool {
        (self & Struct::LITERAL).any()
    }

    pub fn sized(self) -> bool {
        (self & Struct::SIZED).any()
    }
}

impl<'ctx> Struct<'ctx> {
    pub const PACKED: StructFlags = StructFlags(1 << 0);
    pub const OPAQUE: StructFlags = StructFlags(1 << 1);
    pub const LITERAL: StructFlags = StructFlags(1 << 2);
    pub const SIZED: StructFlags = StructFlags(1 << 3);

    #[must_use]
    pub(crate) fn create<I: ExactSizeIterator<Item = Type<'ctx>>>(
        ctx: AllocContext<'ctx>,
        name: Option<istr::IStr>,
        flags: StructFlags,
        arguments: I,
    ) -> Self {
        let args_len = arguments.len();
        match Ty::try_create_in_place(
            ctx,
            StructInit {
                name,
                args_len,
                arguments,
            },
            flags,
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
    pub fn name(self) -> Option<istr::IStr> {
        self.info().name
    }

    #[inline]
    pub fn field_tys(self) -> &'ctx [Type<'ctx>] {
        &self.info().field_tys
    }
}

struct StructInit<I> {
    name: Option<istr::IStr>,
    args_len: usize,
    arguments: I,
}

impl<'ctx, I: Iterator<Item = Type<'ctx>>> TryCtor<StructInit<I>> for StructInfo<'ctx> {
    type Error = init::try_slice::IterInitError<core::convert::Infallible>;

    fn try_init(
        uninit: init::Uninit<'_, Self>,
        args: StructInit<I>,
    ) -> Result<init::Init<'_, Self>, Self::Error> {
        Ok(init::try_init_struct! {
            uninit => Self {
                name: init::try_ctor(|uninit| Ok(uninit.write(args.name))),
                field_tys: init::try_slice::IterInit(args.arguments.map(of_ctor))
            }
        })
    }
}

impl<'ctx, I> HasLayoutProvider<StructInit<I>> for StructInfo<'ctx> {
    type LayoutProvider = FunctionInfoLayoutProvider;
}

struct FunctionInfoLayoutProvider;

unsafe impl<'ctx, I> LayoutProvider<StructInfo<'ctx>, StructInit<I>>
    for FunctionInfoLayoutProvider
{
    fn layout_of(args: &StructInit<I>) -> Option<std::alloc::Layout> {
        Some(
            Layout::new::<Option<istr::IStr>>()
                .extend(Layout::array::<Type>(args.args_len).ok()?)
                .ok()?
                .0,
        )
    }

    unsafe fn cast(
        ptr: std::ptr::NonNull<u8>,
        args: &StructInit<I>,
    ) -> std::ptr::NonNull<StructInfo<'ctx>> {
        std::ptr::NonNull::from_raw_parts(ptr.cast(), args.args_len)
    }
}

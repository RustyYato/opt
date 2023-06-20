use std::num::NonZeroU16;

use init::Ctor;

use crate::{
    types::{self, TypeCache},
    AllocContext,
};

use super::Target;

pub(crate) struct TypeContextInfo<'ctx> {
    unit: types::UnitTy<'ctx>,

    i1: types::IntegerTy<'ctx>,
    i8: types::IntegerTy<'ctx>,
    i16: types::IntegerTy<'ctx>,
    i32: types::IntegerTy<'ctx>,
    i64: types::IntegerTy<'ctx>,
    i128: types::IntegerTy<'ctx>,
    isize: types::IntegerTy<'ctx>,
    iptr: types::IntegerTy<'ctx>,

    f16: types::FloatTy<'ctx>,
    f32: types::FloatTy<'ctx>,
    f64: types::FloatTy<'ctx>,

    ptr_ty: types::PointerTy<'ctx>,

    int_cache: TypeCache<'ctx, types::IntegerInfo>,
    ptr_cache: TypeCache<'ctx, types::PointerInfo>,
    function_cache: TypeCache<'ctx, types::FunctionInfo<'ctx>>,
    struct_cache: TypeCache<'ctx, types::StructInfo<'ctx>>,
    array_cache: TypeCache<'ctx, types::ArrayInfo<'ctx>>,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TypeContext<'ctx> {
    pub(super) info: &'ctx TypeContextInfo<'ctx>,
}

pub(super) struct TypeContextBuilder<'ctx, 't> {
    pub alloc: AllocContext<'ctx>,
    pub target: &'t Target,
}

macro_rules! nz {
    ($x:literal) => {{
        const NZ: NonZeroU16 = match NonZeroU16::new($x) {
            Some(x) => x,
            None => panic!("Cannot make 0 non-zero"),
        };
        NZ
    }};
}

impl<'ctx> Ctor<TypeContextBuilder<'ctx, '_>> for TypeContextInfo<'ctx> {
    fn init<'a>(
        uninit: init::Uninit<'a, Self>,
        builder: TypeContextBuilder<'ctx, '_>,
    ) -> init::Init<'a, Self> {
        let alloc = builder.alloc;
        let target = builder.target;

        let i1 = types::IntegerTy::create(alloc, nz!(1));
        let i8 = types::IntegerTy::create(alloc, nz!(8));
        let i16 = types::IntegerTy::create(alloc, nz!(16));
        let i32 = types::IntegerTy::create(alloc, nz!(32));
        let i64 = types::IntegerTy::create(alloc, nz!(64));
        let i128 = types::IntegerTy::create(alloc, nz!(128));

        let get = |bits: super::PtrBits| match bits {
            super::PtrBits::_8 => i8,
            super::PtrBits::_16 => i16,
            super::PtrBits::_32 => i32,
            super::PtrBits::_64 => i64,
            super::PtrBits::_128 => i128,
        };

        uninit.write(TypeContextInfo {
            unit: types::UnitTy::create(alloc),

            i1,
            i8,
            i16,
            i32,
            i64,
            i128,
            isize: get(target.ptr_diff_bits),
            iptr: get(target.ptr_size_bits),

            f16: types::FloatTy::create(alloc, types::FloatKind::Ieee16Bit),
            f32: types::FloatTy::create(alloc, types::FloatKind::Ieee32Bit),
            f64: types::FloatTy::create(alloc, types::FloatKind::Ieee64Bit),

            ptr_ty: types::PointerTy::create(alloc, types::AddressSpace::DEFAULT),

            int_cache: TypeCache::new(),
            ptr_cache: TypeCache::new(),
            function_cache: TypeCache::new(),
            struct_cache: TypeCache::new(),
            array_cache: TypeCache::new(),
        })
    }
}

macro_rules! getters {
    (
        $($name:ident: $ty:ident)*
    ) => {$(
        #[inline]
        #[must_use]
        pub fn $name(self) -> types::$ty<'ctx> {
            self.info.$name
        }
    )*};
}

impl<'ctx> TypeContext<'ctx> {
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
    pub fn int(self, alloc: AllocContext<'ctx>, bits: NonZeroU16) -> types::IntegerTy<'ctx> {
        match bits.get() {
            1 => return self.info.i1,
            8 => return self.info.i8,
            16 => return self.info.i16,
            32 => return self.info.i32,
            64 => return self.info.i64,
            128 => return self.info.i128,
            _ => (),
        }

        self.info.int_cache.get_or_create(alloc, bits)
    }

    #[inline]
    pub fn ptr_at(
        self,
        alloc: AllocContext<'ctx>,
        address_space: types::AddressSpace,
    ) -> types::PointerTy<'ctx> {
        if address_space.is_default() {
            self.info.ptr_ty
        } else {
            self.info.ptr_cache.get_or_create(alloc, address_space)
        }
    }

    pub fn function(
        self,
        alloc: AllocContext<'ctx>,
        output_ty: types::Type<'ctx>,
        arguments: &[types::Type<'ctx>],
    ) -> types::FunctionTy<'ctx> {
        self.info.function_cache.get_or_create(
            alloc,
            types::FunctionInit {
                output_ty,
                arguments,
            },
        )
    }

    pub fn array(
        self,
        alloc: AllocContext<'ctx>,
        len: u64,
        item_ty: types::Type<'ctx>,
    ) -> types::ArrayTy<'ctx> {
        self.info
            .array_cache
            .get_or_create(alloc, types::ArrayInit { len, item_ty })
    }

    pub fn struct_ty(
        self,
        alloc: AllocContext<'ctx>,
        name: Option<istr::IStr>,
        flags: types::StructFlags,
        field_tys: &[types::Type<'ctx>],
    ) -> types::StructTy<'ctx> {
        self.info.struct_cache.get_or_create(
            alloc,
            (
                flags,
                types::StructInit {
                    name,
                    fields: field_tys,
                },
            ),
        )
    }
}

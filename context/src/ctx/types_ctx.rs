use std::{cell::RefCell, num::NonZeroU16};

use hashbrown::{hash_map::VacantEntry, HashMap};
use init::Ctor;

use crate::{types, AllocContext};

use super::Target;

type FxHashMap<K, V> = HashMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

pub(crate) struct TypeContextInfo<'ctx> {
    unit: types::Unit<'ctx>,

    i1: types::Integer<'ctx>,
    i8: types::Integer<'ctx>,
    i16: types::Integer<'ctx>,
    i32: types::Integer<'ctx>,
    i64: types::Integer<'ctx>,
    i128: types::Integer<'ctx>,
    isize: types::Integer<'ctx>,
    iptr: types::Integer<'ctx>,

    int_cache: RefCell<FxHashMap<NonZeroU16, types::Integer<'ctx>>>,
    ptr_cache: RefCell<FxHashMap<types::Type<'ctx>, types::Pointer<'ctx>>>,
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

        let i1 = types::Integer::create(alloc, nz!(1));
        let i8 = types::Integer::create(alloc, nz!(8));
        let i16 = types::Integer::create(alloc, nz!(16));
        let i32 = types::Integer::create(alloc, nz!(32));
        let i64 = types::Integer::create(alloc, nz!(64));
        let i128 = types::Integer::create(alloc, nz!(128));

        let get = |bits: super::PtrBits| match bits {
            super::PtrBits::_8 => i8,
            super::PtrBits::_16 => i16,
            super::PtrBits::_32 => i32,
            super::PtrBits::_64 => i64,
            super::PtrBits::_128 => i128,
        };

        uninit.write(TypeContextInfo {
            unit: types::Unit::create(alloc),

            i1,
            i8,
            i16,
            i32,
            i64,
            i128,
            isize: get(target.ptr_diff_bits),
            iptr: get(target.ptr_size_bits),

            int_cache: RefCell::new(FxHashMap::default()),
            ptr_cache: RefCell::new(FxHashMap::default()),
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

    #[inline]
    pub fn int(self, alloc: AllocContext<'ctx>, bits: NonZeroU16) -> types::Integer<'ctx> {
        match bits.get() {
            1 => return self.info.i1,
            8 => return self.info.i8,
            16 => return self.info.i16,
            32 => return self.info.i32,
            64 => return self.info.i64,
            128 => return self.info.i128,
            _ => (),
        }

        match self.info.int_cache.borrow_mut().entry(bits) {
            hashbrown::hash_map::Entry::Occupied(entry) => *entry.get(),
            hashbrown::hash_map::Entry::Vacant(entry) => self.create_int(alloc, entry, bits),
        }
    }

    #[cold]
    #[inline(never)]
    fn create_int<S: std::hash::BuildHasher>(
        self,
        alloc: AllocContext<'ctx>,
        entry: VacantEntry<NonZeroU16, types::Integer<'ctx>, S>,
        bits: NonZeroU16,
    ) -> types::Integer<'ctx> {
        *entry.insert(types::Integer::create(alloc, bits))
    }

    #[inline]
    pub fn ptr(
        self,
        alloc: AllocContext<'ctx>,
        target_ty: types::Type<'ctx>,
    ) -> types::Pointer<'ctx> {
        match self.info.ptr_cache.borrow_mut().entry(target_ty) {
            hashbrown::hash_map::Entry::Occupied(entry) => *entry.get(),
            hashbrown::hash_map::Entry::Vacant(entry) => self.create_ptr(alloc, entry, target_ty),
        }
    }

    #[cold]
    #[inline(never)]
    fn create_ptr<S: std::hash::BuildHasher>(
        self,
        alloc: AllocContext<'ctx>,
        entry: VacantEntry<types::Type<'ctx>, types::Pointer<'ctx>, S>,
        target_ty: types::Type<'ctx>,
    ) -> types::Pointer<'ctx> {
        *entry.insert(types::Pointer::create(alloc, target_ty))
    }

    pub fn function<I: ExactSizeIterator<Item = types::Type<'ctx>>>(
        self,
        alloc: AllocContext<'ctx>,
        output_ty: types::Type<'ctx>,
        arguments: I,
    ) -> types::Function<'ctx> {
        types::Function::create(alloc, output_ty, arguments)
    }

    pub fn array(
        self,
        alloc: AllocContext<'ctx>,
        len: u64,
        item_ty: types::Type<'ctx>,
    ) -> types::Array<'ctx> {
        types::Array::create(alloc, item_ty, len)
    }

    pub fn struct_ty<I: ExactSizeIterator<Item = types::Type<'ctx>>>(
        self,
        alloc: AllocContext<'ctx>,
        name: Option<istr::IStr>,
        flags: types::StructFlags,
        field_tys: I,
    ) -> types::Struct<'ctx> {
        types::Struct::create(alloc, name, flags, field_tys)
    }
}

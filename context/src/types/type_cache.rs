use std::{cell::RefCell, hash::Hasher};

use hashbrown::raw;

use crate::AllocContext;

use super::{raw_type::TypeInfo, Ty};

fn hash_one<T: core::hash::Hash>(value: T) -> u64 {
    let mut hasher = rustc_hash::FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

pub struct TypeCache<'ctx, T: ?Sized + TypeInfo<'ctx>> {
    table: RefCell<raw::RawTable<Ty<'ctx, T>>>,
}

impl<'ctx, T: ?Sized + TypeInfo<'ctx>> TypeCache<'ctx, T> {
    pub fn new() -> Self {
        Self {
            table: RefCell::new(raw::RawTable::new()),
        }
    }

    pub fn get_or_create(&self, alloc: AllocContext<'ctx>, key: T::Key<'_>) -> Ty<'ctx, T> {
        let mut table = self.table.borrow_mut();

        let hash = hash_one(key);

        if let Some(&ty) = table.get(hash, |ptr| ptr.info().key(ptr.flags()) == key) {
            return ty;
        }

        self.insert(alloc, &mut table, hash, key)
    }

    #[cold]
    #[inline(never)]
    fn insert(
        &self,
        alloc: AllocContext<'ctx>,
        table: &mut raw::RawTable<Ty<'ctx, T>>,
        hash: u64,
        key: T::Key<'_>,
    ) -> Ty<'ctx, T> {
        let ty = T::create_from_key(alloc, key);
        table.insert(hash, ty, |ty| hash_one(ty.info().key(ty.flags())));
        ty
    }
}

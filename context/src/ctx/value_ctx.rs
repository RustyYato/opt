use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
};

use hashbrown::{raw, HashSet};
use init::Ctor;

use crate::{types, value, AllocContext};

pub(crate) struct ValueContextInfo<'ctx> {
    int_one: rug::Integer,
    int_table: RefCell<HashSet<rug::Integer>>,
    const_integers: RefCell<raw::RawTable<value::ConstInt<'ctx>>>,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ValueContext<'ctx> {
    pub(super) info: &'ctx ValueContextInfo<'ctx>,
}

fn borrow(x: &rug::Integer) -> rug::integer::BorrowInteger<'_> {
    let raw = x.as_raw();
    // create a shallow copy of the integer
    unsafe { rug::integer::BorrowInteger::from_raw(*raw) }
}

fn hash_one<T: Hash>(value: T) -> u64 {
    let mut hasher = rustc_hash::FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

impl<'ctx> ValueContext<'ctx> {
    pub fn zero_value(self) -> &'ctx rug::Integer {
        static ZERO: rug::Integer = rug::Integer::ZERO;
        &ZERO
    }

    pub fn one_value(self) -> &'ctx rug::Integer {
        &self.info.int_one
    }

    pub fn intern_u32(self, x: u32) -> &'ctx rug::Integer {
        match x {
            0 => self.zero_value(),
            1 => self.one_value(),
            x => {
                let x = rug::Integer::from_f64(x.into()).unwrap();
                self.intern_integer_value(x)
            }
        }
    }

    pub fn intern_i32(self, x: i32) -> &'ctx rug::Integer {
        match x {
            0 => self.zero_value(),
            1 => self.one_value(),
            x => {
                let x = rug::Integer::from_f64(x.into()).unwrap();
                self.intern_integer_value(x)
            }
        }
    }

    pub fn intern_integer_value(self, x: rug::Integer) -> &'ctx rug::Integer {
        let mut table;
        if x == *self.zero_value() {
            self.zero_value()
        } else if x == *self.one_value() {
            self.one_value()
        } else {
            table = self.info.int_table.borrow_mut();
            // no integers in the table get mutated after insertion
            unsafe { &*(table.get_or_insert(x) as *const _) }
        }
    }

    pub(crate) fn const_int(
        self,
        alloc: AllocContext<'ctx>,
        ty: types::IntegerTy<'ctx>,
        value: &'ctx rug::Integer,
        signed: bool,
    ) -> Option<value::ConstInt<'ctx>> {
        if !signed && value < self.zero_value() {
            return None;
        }

        let bits = if signed {
            value.signed_bits()
        } else {
            value.significant_bits()
        };

        if bits > u32::from(ty.bits().get()) {
            return None;
        }

        let table = &mut *self.info.const_integers.borrow_mut();

        let hash = hash_one((ty, signed, value));

        if let Some(&value) = table.get(hash, |x| {
            x.ty() == ty.erase() && x.is_signed() == signed && *x.value() == *value
        }) {
            return Some(value);
        }

        let value = value::ConstInt::new(alloc, ty, borrow(value), signed);

        table.insert(hash, value, |x| hash_one((x.ty(), signed, &*x.value())));

        Some(value)
    }
}

impl<'ctx> Ctor for ValueContextInfo<'ctx> {
    fn init(uninit: init::Uninit<'_, Self>, (): ()) -> init::Init<'_, Self> {
        uninit.write(Self {
            int_one: rug::Integer::from_f32(1.0).expect("One is a value integer"),
            int_table: RefCell::new(HashSet::new()),
            const_integers: RefCell::new(raw::RawTable::new()),
        })
    }
}

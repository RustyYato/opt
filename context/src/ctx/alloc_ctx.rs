use std::marker::PhantomData;

use init::{layout_provider::HasLayoutProvider, Ctor, TryCtor};

use super::{ContextRef, Invariant};

pub(crate) struct AllocContextInfo<'ctx> {
    alloc: bumpalo::Bump,
    pub ctx_ref: ContextRef<'ctx>,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct AllocContext<'ctx> {
    pub(super) info: &'ctx AllocContextInfo<'ctx>,
}

impl<'ctx> AllocContext<'ctx> {
    pub(crate) fn try_create_in_place<T, Args>(self, args: Args) -> Result<&'ctx T, T::Error>
    where
        T: ?Sized + TryCtor<Args> + HasLayoutProvider<Args>,
    {
        let layout = init::layout_provider::layout_of::<T, Args>(&args).unwrap();
        let ptr = self.info.alloc.alloc_layout(layout);

        let ptr = unsafe { init::layout_provider::cast::<T, Args>(ptr, &args) };

        // SAFETY: all pointers from bumpalo are valid for reads/writes, and this one fits `T`
        let ptr = unsafe { init::Uninit::from_raw(ptr.as_ptr()) };
        let init = ptr.try_init(args)?;
        Ok(init.into_mut())
    }

    pub(crate) fn ctx_ref(self) -> ContextRef<'ctx> {
        self.info.ctx_ref
    }
}

impl Ctor for AllocContextInfo<'_> {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, (): ()) -> init::Init<'_, Self> {
        uninit.write(AllocContextInfo {
            alloc: bumpalo::Bump::new(),
            ctx_ref: ContextRef(Invariant(PhantomData)),
        })
    }
}

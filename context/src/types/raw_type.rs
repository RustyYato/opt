use std::{
    alloc::Layout,
    hash::Hash,
    ptr::{NonNull, Pointee},
};

use init::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    try_ctor::{of_ctor, OfCtor},
    Ctor, TryCtor,
};

use crate::ctx::{AllocContext, ContextRef};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq)]
pub struct Type<'ctx> {
    data: NonNull<RawTypeInfoData<'ctx>>,
}

impl<'ctx> PartialEq for Type<'ctx> {
    fn eq(&self, other: &Self) -> bool {
        self.unpack() == other.unpack()
    }
}

impl<'ctx> Hash for Type<'ctx> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self.unpack() {
            UnpackedType::Unit(x) => x.hash(state),
            UnpackedType::Integer(x) => x.hash(state),
            UnpackedType::Pointer(x) => x.hash(state),
        }
    }
}

impl<'ctx, T: ?Sized + TypeInfo + PartialEq> PartialEq<Ty<'ctx, T>> for Type<'ctx> {
    fn eq(&self, other: &Ty<'ctx, T>) -> bool {
        self.try_cast() == Some(*other)
    }
}

unsafe impl Send for Type<'_> {}
unsafe impl Sync for Type<'_> {}

#[repr(transparent)]
#[derive(Debug)]
pub struct Ty<'ctx, T: ?Sized> {
    data: &'ctx TypeInfoData<'ctx, T>,
}

impl<T: ?Sized> Copy for Ty<'_, T> {}
impl<T: ?Sized> Clone for Ty<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'ctx, T: ?Sized + TypeInfo + Eq> Eq for Ty<'ctx, T> {}
impl<'ctx, T: ?Sized + TypeInfo + PartialEq> PartialEq for Ty<'ctx, T> {
    fn eq(&self, other: &Self) -> bool {
        match T::TAG {
            TypeTag::Unit => true,
            TypeTag::Integer => core::ptr::eq(self.data, other.data),
            TypeTag::Pointer => self.data == other.data,
        }
    }
}

impl<'ctx, T: ?Sized + TypeInfo + Hash> Hash for Ty<'ctx, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match T::TAG {
            TypeTag::Unit => (),
            TypeTag::Integer => core::ptr::hash(self.data, state),
            TypeTag::Pointer => self.data.hash(state),
        }
    }
}

impl<'ctx, T: ?Sized> From<Ty<'ctx, T>> for Type<'ctx> {
    #[inline]
    fn from(value: Ty<'ctx, T>) -> Self {
        value.erase()
    }
}

#[repr(C)]
pub struct RawTypeInfoData<'ctx> {
    _ctx: ContextRef<'ctx>,
    type_tag: TypeTag,
}

#[repr(C)]
pub struct TypeInfoDataHeader<'ctx, T: ?Sized> {
    _ctx: ContextRef<'ctx>,
    type_tag: TypeTag,
    meta: <T as Pointee>::Metadata,
}

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct TypeInfoData<'ctx, T: ?Sized> {
    _ctx: ContextRef<'ctx>,
    type_tag: TypeTag,
    meta: <Self as Pointee>::Metadata,
    info: T,
}

impl<T: ?Sized + core::fmt::Debug> core::fmt::Debug for TypeInfoData<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeInfoData")
            .field("_ctx", &self._ctx)
            .field("type_tag", &self.type_tag)
            // .field("meta", &self.meta)
            .field("info", &&self.info)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeTag {
    Unit,
    Integer,
    Pointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnpackedType<'ctx> {
    Unit(super::Unit<'ctx>),
    Integer(super::Integer<'ctx>),
    Pointer(super::Pointer<'ctx>),
}

/// # Safety
///
/// This must be the only type with the given tag
pub unsafe trait TypeInfo {
    const TAG: TypeTag;
}

impl<'ctx, T: ?Sized> Ty<'ctx, T> {
    pub(crate) fn create_in_place<Args>(ctx: AllocContext<'ctx>, args: Args) -> Self
    where
        T: TypeInfo + Ctor<Args> + HasLayoutProvider<Args>,
    {
        match Self::try_create_in_place::<OfCtor<Args>>(ctx, of_ctor(args)) {
            Ok(ty) => ty,
            Err(inf) => match inf {},
        }
    }

    pub(crate) fn try_create_in_place<Args>(
        ctx: AllocContext<'ctx>,
        args: Args,
    ) -> Result<Self, T::Error>
    where
        T: TypeInfo + TryCtor<Args> + HasLayoutProvider<Args>,
    {
        Ok(Self {
            data: ctx.try_create_in_place(BuildTypeInfo(ctx.ctx_ref(), args))?,
        })
    }

    pub fn erase(self) -> Type<'ctx> {
        Type {
            data: NonNull::from(self.data).cast(),
        }
    }

    #[inline]
    pub fn tag(&self) -> TypeTag {
        self.data.type_tag
    }

    #[inline]
    pub fn info(&self) -> &T {
        &self.data.info
    }
}

impl<'ctx> Type<'ctx> {
    unsafe fn metadata<T: ?Sized>(self) -> <TypeInfoData<'ctx, T> as Pointee>::Metadata {
        let header = &*self
            .data
            .as_ptr()
            .cast::<TypeInfoDataHeader<'ctx, TypeInfoData<T>>>();
        header.meta
    }

    #[inline]
    pub fn tag(&self) -> TypeTag {
        unsafe { self.data.as_ref().type_tag }
    }

    pub fn cast<T: ?Sized + TypeInfo>(self) -> Ty<'ctx, T> {
        #[cold]
        #[inline(never)]
        fn failed_cast(found: TypeTag, expected: TypeTag) -> ! {
            panic!("Could not cast `Type` to `{found:?}` because it has type {expected:?}")
        }

        match self.try_cast() {
            Some(ty) => ty,
            None => failed_cast(T::TAG, self.tag()),
        }
    }

    /// # Safety
    ///
    /// This type must have the tag `T::TAG`
    pub unsafe fn cast_unchecked<T: ?Sized + TypeInfo>(self) -> Ty<'ctx, T> {
        debug_assert_eq!(self.tag(), T::TAG);

        let metadata = unsafe { self.metadata::<T>() };
        let ptr = core::ptr::NonNull::<TypeInfoData<T>>::from_raw_parts(self.data.cast(), metadata);

        Ty {
            data: unsafe { &*ptr.as_ptr() },
        }
    }

    pub fn try_cast<T: ?Sized + TypeInfo>(self) -> Option<Ty<'ctx, T>> {
        if self.tag() == T::TAG {
            Some(unsafe { self.cast_unchecked() })
        } else {
            None
        }
    }

    pub fn unpack(self) -> UnpackedType<'ctx> {
        match self.tag() {
            TypeTag::Unit => UnpackedType::Unit(unsafe { self.cast_unchecked() }),
            TypeTag::Integer => UnpackedType::Integer(unsafe { self.cast_unchecked() }),
            TypeTag::Pointer => UnpackedType::Pointer(unsafe { self.cast_unchecked() }),
        }
    }
}

struct BuildTypeInfo<'ctx, Args>(ContextRef<'ctx>, Args);

struct BuildTypeInfoLayoutProvider;

impl<'ctx, T: ?Sized + HasLayoutProvider<Args>, Args> HasLayoutProvider<BuildTypeInfo<'ctx, Args>>
    for TypeInfoData<'ctx, T>
{
    type LayoutProvider = BuildTypeInfoLayoutProvider;
}

unsafe impl<'ctx, T: ?Sized + HasLayoutProvider<Args>, Args>
    LayoutProvider<TypeInfoData<'ctx, T>, BuildTypeInfo<'ctx, Args>>
    for BuildTypeInfoLayoutProvider
{
    fn layout_of(args: &BuildTypeInfo<'ctx, Args>) -> Option<std::alloc::Layout> {
        let prefix = Layout::new::<RawTypeInfoData>();
        let meta = Layout::new::<<T as Pointee>::Metadata>();
        let layout = init::layout_provider::layout_of::<T, Args>(&args.1)?;
        Some(prefix.extend(meta).ok()?.0.extend(layout).ok()?.0)
    }

    unsafe fn cast(
        ptr: NonNull<u8>,
        args: &BuildTypeInfo<'ctx, Args>,
    ) -> NonNull<TypeInfoData<'ctx, T>> {
        let args = init::layout_provider::cast::<T, Args>(ptr, &args.1);
        NonNull::new_unchecked(args.as_ptr() as _)
    }
}

impl<'ctx, T, Args> TryCtor<BuildTypeInfo<'ctx, Args>> for TypeInfoData<'ctx, T>
where
    T: ?Sized + TypeInfo + TryCtor<Args>,
{
    type Error = T::Error;

    #[inline]
    fn try_init<'a>(
        uninit: init::Uninit<'a, Self>,
        BuildTypeInfo(ctx, args): BuildTypeInfo<'ctx, Args>,
    ) -> Result<init::Init<'a, Self>, Self::Error> {
        let meta = core::ptr::metadata(uninit.as_ptr());
        Ok(init::try_init_struct! {
            uninit => Self {
                _ctx: init::try_ctor(|uninit| Ok(uninit.write(ctx))),
                type_tag: init::try_ctor(|uninit| Ok(uninit.write(T::TAG))),
                info: args,
                meta: init::try_ctor(|uninit| {
                    Ok(uninit.write(meta))
                }),
            }
        })
    }
}

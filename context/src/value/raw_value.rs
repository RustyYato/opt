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

use crate::{ctx::AllocContext, types::Type};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Value<'ctx, Tag = ValueTag> {
    data: NonNull<RawValueInfoData<'ctx, Tag>>,
}

impl<'ctx> Eq for Value<'ctx> {}
impl<'ctx> PartialEq for Value<'ctx> {
    fn eq(&self, other: &Self) -> bool {
        self.unpack() == other.unpack()
    }
}

impl<'ctx> Hash for Value<'ctx> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self.unpack() {
            UnpackedValue::ConstAggrZero(x) => x.hash(state),
        }
    }
}

impl<'ctx> core::fmt::Debug for Value<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = self.unpack();
        let x: &dyn core::fmt::Debug = match &x {
            UnpackedValue::ConstAggrZero(x) => x,
        };

        core::fmt::Debug::fmt(x, f)
    }
}

impl<'ctx, T: ?Sized + ValueInfo + PartialEq> PartialEq<Val<'ctx, T>> for Value<'ctx> {
    fn eq(&self, other: &Val<'ctx, T>) -> bool {
        self.try_cast() == Some(*other)
    }
}

unsafe impl Send for Value<'_> {}
unsafe impl Sync for Value<'_> {}

#[repr(transparent)]
pub struct Val<'ctx, T: ?Sized + ValueInfo> {
    data: &'ctx ValueInfoData<'ctx, T>,
}

impl<T: ?Sized + ValueInfo + core::fmt::Debug> core::fmt::Debug for Val<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.info().fmt(f)
    }
}

impl<T: ?Sized + ValueInfo> Copy for Val<'_, T> {}
impl<T: ?Sized + ValueInfo> Clone for Val<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'ctx, T: ?Sized + ValueInfo + Eq> Eq for Val<'ctx, T> {}
impl<'ctx, T: ?Sized + ValueInfo + PartialEq> PartialEq for Val<'ctx, T> {
    fn eq(&self, other: &Self) -> bool {
        self.ty() == other.ty()
            && match T::TAG {
                ValueTag::ConstAggrZero => true,
            }
    }
}

impl<'ctx, T: ?Sized + ValueInfo + Hash> Hash for Val<'ctx, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ty().hash(state);
        match T::TAG {
            ValueTag::ConstAggrZero => (),
        }
    }
}

impl<'ctx, T: ?Sized + ValueInfo> From<Val<'ctx, T>> for Value<'ctx> {
    #[inline]
    fn from(value: Val<'ctx, T>) -> Self {
        value.erase()
    }
}

impl<'ctx, T: ?Sized + ValueInfo> From<&Val<'ctx, T>> for Value<'ctx> {
    #[inline]
    fn from(value: &Val<'ctx, T>) -> Self {
        value.erase()
    }
}

impl<'ctx> From<&Value<'ctx>> for Value<'ctx> {
    #[inline]
    fn from(value: &Value<'ctx>) -> Self {
        *value
    }
}

#[repr(C)]
pub struct RawValueInfoData<'ctx, Tag = ValueTag> {
    ty: Type<'ctx>,
    value_tag: Tag,
}

#[repr(C)]
pub struct ValueInfoDataHeader<'ctx, T: ?Sized, F = <T as ValueInfo>::Flags> {
    ty: Type<'ctx>,
    value_tag: ValueTag,
    flags: F,
    meta: <T as Pointee>::Metadata,
}

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct ValueInfoData<'ctx, T: ?Sized + ValueInfo> {
    ty: Type<'ctx>,
    value_tag: ValueTag,
    flags: T::Flags,
    meta: <Self as Pointee>::Metadata,
    info: T,
}

impl<T: ?Sized + core::fmt::Debug + ValueInfo> core::fmt::Debug for ValueInfoData<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueInfoData")
            .field("ty", &self.ty)
            .field("value_tag", &self.value_tag)
            // .field("meta", &self.meta)
            .field("info", &&self.info)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueTag {
    ConstAggrZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnpackedValue<'ctx> {
    ConstAggrZero(super::ConstAggrZero<'ctx>),
}

/// # Safety
///
/// This must be the only value with the given tag
pub unsafe trait ValueInfo {
    const TAG: ValueTag;
    type Flags: Copy + PartialEq + Hash;
}

impl<'ctx, T: ?Sized + ValueInfo> Val<'ctx, T> {
    pub(crate) fn create_in_place<Args>(
        ctx: AllocContext<'ctx>,
        ty: Type<'ctx>,
        args: Args,
        flags: T::Flags,
    ) -> Self
    where
        T: Ctor<Args> + HasLayoutProvider<Args>,
    {
        match Self::try_create_in_place::<OfCtor<Args>>(ctx, ty, of_ctor(args), flags) {
            Ok(ty) => ty,
            Err(inf) => match inf {},
        }
    }

    pub(crate) fn try_create_in_place<Args>(
        ctx: AllocContext<'ctx>,
        ty: Type<'ctx>,
        args: Args,
        flags: T::Flags,
    ) -> Result<Self, T::Error>
    where
        T: TryCtor<Args> + HasLayoutProvider<Args>,
    {
        Ok(Self {
            data: ctx.try_create_in_place(BuildValueInfo(ty, args, flags))?,
        })
    }

    pub fn erase(self) -> Value<'ctx> {
        Value {
            data: NonNull::from(self.data).cast(),
        }
    }

    #[inline]
    pub fn ty(self) -> Type<'ctx> {
        self.data.ty
    }

    #[inline]
    pub fn tag(self) -> ValueTag {
        self.data.value_tag
    }

    #[inline]
    pub fn flags(self) -> T::Flags {
        self.data.flags
    }

    #[inline]
    pub fn info(self) -> &'ctx T {
        &self.data.info
    }
}

impl<'ctx> Value<'ctx> {
    unsafe fn metadata<T: ?Sized + ValueInfo>(
        self,
    ) -> <ValueInfoData<'ctx, T> as Pointee>::Metadata {
        let header = &*self
            .data
            .as_ptr()
            .cast::<ValueInfoDataHeader<'ctx, ValueInfoData<T>, T::Flags>>();
        header.meta
    }

    #[inline]
    pub fn ty(self) -> Type<'ctx> {
        unsafe { self.data.as_ref().ty }
    }

    #[inline]
    pub fn tag(&self) -> ValueTag {
        unsafe { self.data.as_ref().value_tag }
    }

    pub fn cast<T: ?Sized + ValueInfo>(self) -> Val<'ctx, T> {
        #[cold]
        #[inline(never)]
        fn failed_cast(found: ValueTag, expected: ValueTag) -> ! {
            panic!("Could not cast `Value` to `{found:?}` because it has value {expected:?}")
        }

        match self.try_cast() {
            Some(ty) => ty,
            None => failed_cast(T::TAG, self.tag()),
        }
    }

    /// # Safety
    ///
    /// This value must have the tag `T::TAG`
    pub unsafe fn cast_unchecked<T: ?Sized + ValueInfo>(self) -> Val<'ctx, T> {
        debug_assert_eq!(self.tag(), T::TAG);

        let metadata = unsafe { self.metadata::<T>() };
        let ptr =
            core::ptr::NonNull::<ValueInfoData<T>>::from_raw_parts(self.data.cast(), metadata);

        Val {
            data: unsafe { &*ptr.as_ptr() },
        }
    }

    pub fn try_cast<T: ?Sized + ValueInfo>(self) -> Option<Val<'ctx, T>> {
        if self.tag() == T::TAG {
            Some(unsafe { self.cast_unchecked() })
        } else {
            None
        }
    }

    pub fn unpack(self) -> UnpackedValue<'ctx> {
        match self.tag() {
            ValueTag::ConstAggrZero => {
                UnpackedValue::ConstAggrZero(unsafe { self.cast_unchecked() })
            }
        }
    }
}

struct BuildValueInfo<'ctx, Args, F>(Type<'ctx>, Args, F);

struct BuildValueInfoLayoutProvider;

impl<'ctx, T: ?Sized + ValueInfo + HasLayoutProvider<Args>, Args, F>
    HasLayoutProvider<BuildValueInfo<'ctx, Args, F>> for ValueInfoData<'ctx, T>
{
    type LayoutProvider = BuildValueInfoLayoutProvider;
}

unsafe impl<'ctx, T: ?Sized + ValueInfo + HasLayoutProvider<Args>, Args, F>
    LayoutProvider<ValueInfoData<'ctx, T>, BuildValueInfo<'ctx, Args, F>>
    for BuildValueInfoLayoutProvider
{
    fn layout_of(args: &BuildValueInfo<'ctx, Args, F>) -> Option<std::alloc::Layout> {
        let ty = Layout::new::<Type<'ctx>>();
        let meta = Layout::new::<<T as Pointee>::Metadata>();
        let tag = Layout::new::<ValueTag>();
        let flags = Layout::new::<RawValueInfoData>();
        let info_layout = init::layout_provider::layout_of::<T, Args>(&args.1)?;

        let layout = ty.extend(tag).ok()?.0;
        let layout = layout.extend(flags).ok()?.0;
        let layout = layout.extend(meta).ok()?.0;
        let layout = layout.extend(info_layout).ok()?.0;

        Some(layout)
    }

    unsafe fn cast(
        ptr: NonNull<u8>,
        args: &BuildValueInfo<'ctx, Args, F>,
    ) -> NonNull<ValueInfoData<'ctx, T>> {
        let args = init::layout_provider::cast::<T, Args>(ptr, &args.1);
        NonNull::new_unchecked(args.as_ptr() as _)
    }
}

impl<'ctx, T, Args> TryCtor<BuildValueInfo<'ctx, Args, T::Flags>> for ValueInfoData<'ctx, T>
where
    T: ?Sized + ValueInfo + TryCtor<Args>,
{
    type Error = T::Error;

    #[inline]
    fn try_init<'a>(
        uninit: init::Uninit<'a, Self>,
        BuildValueInfo(ty, args, flags): BuildValueInfo<'ctx, Args, T::Flags>,
    ) -> Result<init::Init<'a, Self>, Self::Error> {
        let meta = core::ptr::metadata(uninit.as_ptr());
        Ok(init::try_init_struct! {
            uninit => Self {
                ty: init::try_ctor(|uninit| Ok(uninit.write(ty))),
                value_tag: init::try_ctor(|uninit| Ok(uninit.write(T::TAG))),
                info: args,
                flags: init::try_ctor(|uninit| Ok(uninit.write(flags))),
                meta: init::try_ctor(|uninit| {
                    Ok(uninit.write(meta))
                }),
            }
        })
    }
}

impl<'ctx, Tag> Ctor<Self> for Value<'ctx, Tag> {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, args: Self) -> init::Init<'_, Self> {
        uninit.write(args)
    }
}

use core::fmt;
use std::{
    alloc::Layout,
    hash::Hash,
    ptr::{NonNull, Pointee},
};

use init::{
    ctor::{CloneCtor, MoveCtor, TakeCtor},
    layout_provider::{HasLayoutProvider, LayoutProvider},
    try_ctor::{of_ctor, OfCtor},
    Ctor, TryCtor,
};

use crate::ctx::{AllocContext, ContextRef};

#[repr(transparent)]
pub struct Type<'ctx, Tag = TypeTag> {
    data: NonNull<RawTypeInfoData<'ctx, Tag>>,
}

impl<'ctx, Tag> Copy for Type<'ctx, Tag> {}
impl<'ctx, Tag> Clone for Type<'ctx, Tag> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'ctx> Eq for Type<'ctx> {}
impl<'ctx> PartialEq for Type<'ctx> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<'ctx> Hash for Type<'ctx> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::ptr::hash(self.data.as_ptr(), state)
    }
}

impl<'ctx> fmt::Debug for Type<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = self.unpack();
        let x: &dyn fmt::Debug = match &x {
            UnpackedType::Unit(x) => x,
            UnpackedType::Integer(x) => x,
            UnpackedType::Pointer(x) => x,
            UnpackedType::Function(x) => x,
            UnpackedType::Array(x) => x,
            UnpackedType::Struct(x) => x,
        };

        fmt::Debug::fmt(x, f)
    }
}

impl<'ctx, T: ?Sized + TypeInfo<'ctx> + PartialEq> PartialEq<Ty<'ctx, T>> for Type<'ctx> {
    fn eq(&self, other: &Ty<'ctx, T>) -> bool {
        *self == other.erase()
    }
}

unsafe impl Send for Type<'_> {}
unsafe impl Sync for Type<'_> {}

#[repr(transparent)]
pub struct Ty<'ctx, T: ?Sized + TypeInfo<'ctx>> {
    data: &'ctx TypeInfoData<'ctx, T>,
}

impl<'ctx, T> fmt::Debug for Ty<'ctx, T>
where
    T: ?Sized + TypeInfo<'ctx> + fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.info().fmt(f)
    }
}

impl<'ctx, T> Copy for Ty<'ctx, T> where T: ?Sized + TypeInfo<'ctx> {}
impl<'ctx, T> Clone for Ty<'ctx, T>
where
    T: ?Sized + TypeInfo<'ctx>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'ctx, T: ?Sized + TypeInfo<'ctx> + Eq> Eq for Ty<'ctx, T> {}
impl<'ctx, T: ?Sized + TypeInfo<'ctx> + PartialEq> PartialEq for Ty<'ctx, T> {
    fn eq(&self, other: &Self) -> bool {
        match T::TAG {
            TypeTag::Unit => true,
            TypeTag::Integer => core::ptr::eq(self.data, other.data),
            TypeTag::Pointer | TypeTag::Function | TypeTag::Array | TypeTag::Struct => {
                self.data == other.data
            }
        }
    }
}

impl<'ctx, T: ?Sized + TypeInfo<'ctx> + Hash> Hash for Ty<'ctx, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match T::TAG {
            TypeTag::Unit => (),
            TypeTag::Integer => core::ptr::hash(self.data, state),
            TypeTag::Pointer | TypeTag::Function | TypeTag::Array | TypeTag::Struct => {
                self.data.hash(state)
            }
        }
    }
}

impl<'ctx, T: ?Sized + TypeInfo<'ctx>> From<Ty<'ctx, T>> for Type<'ctx> {
    #[inline]
    fn from(value: Ty<'ctx, T>) -> Self {
        value.erase()
    }
}

impl<'ctx, T: ?Sized + TypeInfo<'ctx>> From<&Ty<'ctx, T>> for Type<'ctx> {
    #[inline]
    fn from(value: &Ty<'ctx, T>) -> Self {
        value.erase()
    }
}

impl<'ctx> From<&Type<'ctx>> for Type<'ctx> {
    #[inline]
    fn from(value: &Type<'ctx>) -> Self {
        *value
    }
}

#[repr(C)]
pub struct RawTypeInfoData<'ctx, Tag = TypeTag> {
    _ctx: ContextRef<'ctx>,
    type_tag: Tag,
}

#[repr(C)]
pub struct TypeInfoDataHeader<'ctx, T: ?Sized, F = <T as TypeInfo<'ctx>>::Flags> {
    _ctx: ContextRef<'ctx>,
    type_tag: TypeTag,
    flags: F,
    meta: <T as Pointee>::Metadata,
}

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct TypeInfoData<'ctx, T: ?Sized + TypeInfo<'ctx>> {
    _ctx: ContextRef<'ctx>,
    type_tag: TypeTag,
    flags: T::Flags,
    meta: <Self as Pointee>::Metadata,
    info: T,
}

impl<'ctx, T> fmt::Debug for TypeInfoData<'ctx, T>
where
    T: ?Sized + fmt::Debug + TypeInfo<'ctx>,
{
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
    Function,
    Array,
    Struct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnpackedType<'ctx> {
    Unit(super::UnitTy<'ctx>),
    Integer(super::IntegerTy<'ctx>),
    Pointer(super::PointerTy<'ctx>),
    Function(super::FunctionTy<'ctx>),
    Array(super::ArrayTy<'ctx>),
    Struct(super::StructTy<'ctx>),
}

/// # Safety
///
/// This must be the only type with the given tag
pub unsafe trait TypeInfo<'ctx> {
    const TAG: TypeTag;
    type Flags: Copy + PartialEq + Hash;

    type Key<'a>: Copy + Hash + Eq
    where
        'ctx: 'a;

    fn key<'a>(&'ctx self, flags: Self::Flags) -> Self::Key<'a>
    where
        'ctx: 'a;

    fn create_from_key<'a>(alloc: AllocContext<'ctx>, key: Self::Key<'a>) -> Ty<'ctx, Self>
    where
        'ctx: 'a;
}

impl<'ctx, T: ?Sized + TypeInfo<'ctx>> Ty<'ctx, T> {
    pub(crate) fn create_in_place<Args>(
        ctx: AllocContext<'ctx>,
        args: Args,
        flags: T::Flags,
    ) -> Self
    where
        T: Ctor<Args> + HasLayoutProvider<Args>,
    {
        match Self::try_create_in_place::<OfCtor<Args>>(ctx, of_ctor(args), flags) {
            Ok(ty) => ty,
            Err(inf) => match inf {},
        }
    }

    pub(crate) fn try_create_in_place<Args>(
        ctx: AllocContext<'ctx>,
        args: Args,
        flags: T::Flags,
    ) -> Result<Self, T::Error>
    where
        T: TryCtor<Args> + HasLayoutProvider<Args>,
    {
        Ok(Self {
            data: ctx.try_create_in_place(BuildTypeInfo(ctx.ctx_ref(), args, flags))?,
        })
    }

    pub fn erase(self) -> Type<'ctx> {
        Type {
            data: NonNull::from(self.data).cast(),
        }
    }

    #[inline]
    pub fn tag(self) -> TypeTag {
        self.data.type_tag
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

impl<'ctx> Type<'ctx> {
    unsafe fn metadata<T: ?Sized + TypeInfo<'ctx>>(
        self,
    ) -> <TypeInfoData<'ctx, T> as Pointee>::Metadata {
        let header = &*self
            .data
            .as_ptr()
            .cast::<TypeInfoDataHeader<'ctx, TypeInfoData<T>, T::Flags>>();
        header.meta
    }

    #[inline]
    pub fn tag(&self) -> TypeTag {
        unsafe { self.data.as_ref().type_tag }
    }

    pub fn cast<T: ?Sized + TypeInfo<'ctx>>(self) -> Ty<'ctx, T> {
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
    pub unsafe fn cast_unchecked<T: ?Sized + TypeInfo<'ctx>>(self) -> Ty<'ctx, T> {
        debug_assert_eq!(self.tag(), T::TAG);

        let metadata = unsafe { self.metadata::<T>() };
        let ptr = core::ptr::NonNull::<TypeInfoData<T>>::from_raw_parts(self.data.cast(), metadata);

        Ty {
            data: unsafe { &*ptr.as_ptr() },
        }
    }

    pub fn try_cast<T: ?Sized + TypeInfo<'ctx>>(self) -> Option<Ty<'ctx, T>> {
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
            TypeTag::Function => UnpackedType::Function(unsafe { self.cast_unchecked() }),
            TypeTag::Array => UnpackedType::Array(unsafe { self.cast_unchecked() }),
            TypeTag::Struct => UnpackedType::Struct(unsafe { self.cast_unchecked() }),
        }
    }
}

struct BuildTypeInfo<'ctx, Args, F>(ContextRef<'ctx>, Args, F);

struct BuildTypeInfoLayoutProvider;

impl<'ctx, T: ?Sized + TypeInfo<'ctx> + HasLayoutProvider<Args>, Args, F>
    HasLayoutProvider<BuildTypeInfo<'ctx, Args, F>> for TypeInfoData<'ctx, T>
{
    type LayoutProvider = BuildTypeInfoLayoutProvider;
}

unsafe impl<'ctx, T: ?Sized + TypeInfo<'ctx> + HasLayoutProvider<Args>, Args, F>
    LayoutProvider<TypeInfoData<'ctx, T>, BuildTypeInfo<'ctx, Args, F>>
    for BuildTypeInfoLayoutProvider
{
    fn layout_of(args: &BuildTypeInfo<'ctx, Args, F>) -> Option<std::alloc::Layout> {
        let meta = Layout::new::<<T as Pointee>::Metadata>();
        let tag = Layout::new::<TypeTag>();
        let flags = Layout::new::<RawTypeInfoData>();
        let info_layout = init::layout_provider::layout_of::<T, Args>(&args.1)?;

        let layout = tag.extend(flags).ok()?.0;
        let layout = layout.extend(meta).ok()?.0;
        let layout = layout.extend(info_layout).ok()?.0;

        Some(layout)
    }

    unsafe fn cast(
        ptr: NonNull<u8>,
        args: &BuildTypeInfo<'ctx, Args, F>,
    ) -> NonNull<TypeInfoData<'ctx, T>> {
        let args = init::layout_provider::cast::<T, Args>(ptr, &args.1);
        NonNull::new_unchecked(args.as_ptr() as _)
    }
}

impl<'ctx, T, Args> TryCtor<BuildTypeInfo<'ctx, Args, T::Flags>> for TypeInfoData<'ctx, T>
where
    T: ?Sized + TypeInfo<'ctx> + TryCtor<Args>,
{
    type Error = T::Error;

    #[inline]
    fn try_init<'a>(
        uninit: init::Uninit<'a, Self>,
        BuildTypeInfo(ctx, args, flags): BuildTypeInfo<'ctx, Args, T::Flags>,
    ) -> Result<init::Init<'a, Self>, Self::Error> {
        let meta = core::ptr::metadata(uninit.as_ptr());
        Ok(init::try_init_struct! {
            uninit => Self {
                _ctx: init::try_ctor(|uninit| Ok(uninit.write(ctx))),
                type_tag: init::try_ctor(|uninit| Ok(uninit.write(T::TAG))),
                info: args,
                flags: init::try_ctor(|uninit| Ok(uninit.write(flags))),
                meta: init::try_ctor(|uninit| {
                    Ok(uninit.write(meta))
                }),
            }
        })
    }
}

impl<'ctx, Tag> Ctor<Self> for Type<'ctx, Tag> {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, args: Self) -> init::Init<'_, Self> {
        uninit.write(args)
    }
}

impl<'ctx, Tag> MoveCtor for Type<'ctx, Tag> {
    #[inline]
    fn move_ctor<'this>(
        uninit: init::Uninit<'this, Self>,
        p: init::Init<Self>,
    ) -> init::Init<'this, Self> {
        uninit.write(p.into_inner())
    }
}

impl<'ctx, Tag> TakeCtor for Type<'ctx, Tag> {
    #[inline]
    fn take_ctor<'this>(
        uninit: init::Uninit<'this, Self>,
        p: &mut Self,
    ) -> init::Init<'this, Self> {
        uninit.write(*p)
    }
}

impl<'ctx, Tag> CloneCtor for Type<'ctx, Tag> {
    #[inline]
    fn clone_ctor<'this>(uninit: init::Uninit<'this, Self>, p: &Self) -> init::Init<'this, Self> {
        uninit.write(*p)
    }
}

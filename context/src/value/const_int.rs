use init::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Ctor,
};

use crate::{types::Type, AllocContext};

use super::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstIntInfo {}

pub type ConstInt<'ctx> = super::Val<'ctx, ConstIntInfo>;

unsafe impl super::ValueInfo for ConstIntInfo {
    const TAG: super::ValueTag = super::ValueTag::ConstInt;
    type Flags = ();
}

impl<'ctx> ConstInt<'ctx> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: AllocContext<'ctx>, ty: Type<'ctx>) -> Value<'ctx> {
        Self::create_in_place(ctx, ty, (), ()).erase()
    }
}

impl Ctor for ConstIntInfo {
    fn init(uninit: init::Uninit<'_, Self>, (): ()) -> init::Init<'_, Self> {
        uninit.write(Self {})
    }
}

impl HasLayoutProvider for ConstIntInfo {
    type LayoutProvider = SizedLayoutProvider;
}

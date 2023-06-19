use init::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Ctor,
};

use crate::{types::Type, AllocContext};

use super::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstAggrZeroInfo {}

pub type ConstAggrZero<'ctx> = super::Val<'ctx, ConstAggrZeroInfo>;

unsafe impl super::ValueInfo for ConstAggrZeroInfo {
    const TAG: super::ValueTag = super::ValueTag::ConstAggrZero;
    type Flags = ();
}

impl<'ctx> ConstAggrZero<'ctx> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: AllocContext<'ctx>, ty: Type<'ctx>) -> Value<'ctx> {
        Self::create_in_place(ctx, ty, (), ()).erase()
    }
}

impl Ctor for ConstAggrZeroInfo {
    fn init(uninit: init::Uninit<'_, Self>, (): ()) -> init::Init<'_, Self> {
        uninit.write(Self {})
    }
}

impl HasLayoutProvider for ConstAggrZeroInfo {
    type LayoutProvider = SizedLayoutProvider;
}

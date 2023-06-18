use std::hash::Hash;

use init::Ctor;

#[derive(Clone, Copy, Eq)]
#[allow(non_camel_case_types)]
pub struct AddressSpace(u8, u8, u8);

impl AddressSpace {
    pub const DEFAULT: Self = Self(0, 0, 0);

    pub fn get(self) -> u32 {
        u32::from_ne_bytes([0, self.0, self.1, self.2])
    }

    pub fn is_default(self) -> bool {
        self.get() == 0
    }

    pub fn new(address_space: u32) -> Self {
        assert_eq!(address_space & 0xff000000, 0);
        let [a, b, c, _] = u32::to_le_bytes(address_space);
        Self(a, b, c)
    }
}

impl PartialEq for AddressSpace {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Hash for AddressSpace {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state)
    }
}

impl Ctor for AddressSpace {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, (): ()) -> init::Init<'_, Self> {
        uninit.write(Self::DEFAULT)
    }
}

impl Ctor<AddressSpace> for AddressSpace {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, addr_space: AddressSpace) -> init::Init<'_, Self> {
        uninit.write(addr_space)
    }
}

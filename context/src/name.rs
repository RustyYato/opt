pub trait Name {
    fn to_name(self) -> Option<istr::IStr>;
}

impl Name for () {
    #[inline]
    fn to_name(self) -> Option<istr::IStr> {
        None
    }
}

impl Name for &str {
    #[inline]
    fn to_name(self) -> Option<istr::IStr> {
        Some(istr::IStr::new(self))
    }
}

impl Name for Option<&str> {
    #[inline]
    fn to_name(self) -> Option<istr::IStr> {
        self.map(istr::IStr::new)
    }
}

impl Name for istr::IStr {
    #[inline]
    fn to_name(self) -> Option<istr::IStr> {
        Some(self)
    }
}

impl Name for Option<istr::IStr> {
    #[inline]
    fn to_name(self) -> Option<istr::IStr> {
        self
    }
}

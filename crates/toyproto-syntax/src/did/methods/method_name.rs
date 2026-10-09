use smol_str::SmolStr;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MethodName {
    Defined(SmolStr),

    #[default]
    Undefined,
}

impl Into<MethodName> for &String {
    #[inline(always)]
    fn into(self) -> MethodName {
        MethodName::Defined(self.into())
    }
}

impl Into<MethodName> for &str {
    #[inline(always)]
    fn into(self) -> MethodName {
        MethodName::Defined(self.into())
    }
}

impl Into<MethodName> for String {
    #[inline(always)]
    fn into(self) -> MethodName {
        MethodName::Defined(self.into())
    }
}

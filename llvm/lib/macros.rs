#[macro_export]
macro_rules! impl_trait {
    ($trait:ident ( $fn:ident -> $ret_ty:ty ) for { $( $ty:ident ),+ $(,)? }) => {
        $(
            impl $trait for $ty {
                unsafe fn $fn(&self) -> $ret_ty {
                    $ty::as_ptr(self)
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! not_null {
    ($ptr:expr) => {{
        let ptr = $ptr;
        assert!(!ptr.is_null(), "Got a null pointer from LLVM");
        ptr
    }};
}

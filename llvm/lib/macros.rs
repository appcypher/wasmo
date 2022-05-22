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

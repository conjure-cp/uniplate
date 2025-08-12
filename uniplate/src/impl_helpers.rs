//! Helper functions for manually implementing Uniplate and Biplate instances.

/// If `T` and `U` are the same type, turns a `&T` into a `&U`. Otherwise, returns `None`.
#[inline(always)]
pub fn transmute_if_same_type<T: 'static, U: 'static>(src: &T) -> Option<&U> {
    if std::any::TypeId::of::<T>() == std::any::TypeId::of::<U>() {
        unsafe {
            // SAFETY: already checked that T and U are the same type
            Some(std::mem::transmute::<&T, &U>(src))
        }
    } else {
        None
    }
}

/// If `T` and `U` are the same type, turns a `&T` into a `&U`.
///
/// # Panics
///
/// If `T` and `U` are not the same type.
#[inline(always)]
pub fn try_transmute_if_same_type<T: 'static, U: 'static>(src: &T) -> &U {
    if std::any::TypeId::of::<T>() == std::any::TypeId::of::<U>() {
        unsafe {
            // SAFETY: already checked that T and U are the same type
            std::mem::transmute::<&T, &U>(src)
        }
    } else {
        panic!("T and U are not the same type");
    }
}

//! Validation implementations and helper types.

use crate::{ArchivedDynMetadata, RegisteredImpl, IMPL_REGISTRY};
use bytecheck::CheckBytes;
#[cfg(feature = "vtable_cache")]
use core::sync::atomic::Ordering;
use core::{
    alloc::Layout,
    any::{Any, TypeId},
    convert::Infallible,
    fmt,
    marker::PhantomData,
    ptr,
};
use rkyv::{
    from_archived,
    validation::{ArchiveContext, SharedContext},
    Archived, Fallible,
};
use rkyv_typename::TypeName;
use std::{collections::HashMap, error::Error};

/// A context that's object safe and suitable for checking most types.
pub trait DynContext {
    /// Checks that a relative pointer points to an address within the archive.
    ///
    /// See [`bounds_check_ptr`] for more information.
    ///
    /// # Safety
    ///
    /// - `base` must be inside the archive this valiator was created for.
    ///
    /// [`bounds_check_ptr`]: rkyv::validation::ArchiveContext::bounds_check_ptr
    unsafe fn bounds_check_ptr_dyn(
        &mut self,
        base: *const u8,
        offset: isize,
    ) -> Result<*const u8, Box<dyn Error>>;

    /// Checks that a given pointer can be dereferenced.
    ///
    /// See [`bounds_check_layout`] for more information.
    ///
    /// # Safety
    ///
    /// - `data_address` must be inside the archive this validator was created for.
    /// - `layout` must be the layout for the given pointer.
    ///
    /// [`bounds_check_layout`]: rkyv::validation::ArchiveContext::bounds_check_layout
    unsafe fn bounds_check_layout_dyn(
        &mut self,
        data_address: *const u8,
        layout: &Layout,
    ) -> Result<(), Box<dyn Error>>;

    /// Checks that the given data address and layout is located completely within the subtree
    /// range.
    ///
    /// See [`bounds_check_subtree_ptr_layout`] for more information.
    ///
    /// # Safety
    ///
    /// - `data_address` must be inside the archive this validator was created for.
    ///
    /// [`bounds_check_subtree_ptr_layout`]: rkyv::validation::ArchiveContext::bounds_check_subtree_ptr_layout
    unsafe fn bounds_check_subtree_ptr_layout_dyn(
        &mut self,
        data_address: *const u8,
        layout: &Layout,
    ) -> Result<(), Box<dyn Error>>;

    /// Pushes a new subtree range onto the validator and starts validating it.
    ///
    /// See [`push_prefix_subtree_range`] for more information.
    ///
    /// # Safety
    ///
    /// `root` and `end` must be located inside the archive.
    ///
    /// [`push_prefix_subtree_range`]: rkyv::validation::ArchiveContext::push_prefix_subtree_range
    unsafe fn push_prefix_subtree_range_dyn(
        &mut self,
        root: *const u8,
        end: *const u8,
    ) -> Result<Box<dyn Any>, Box<dyn Error>>;

    /// Pops the given range, restoring the original state with the pushed range removed.
    ///
    /// See [`pop_prefix_range`] for more information.
    ///
    /// [`pop_prefix_range`]: rkyv::validation::ArchiveContext::pop_prefix_range
    fn pop_prefix_range_dyn(&mut self, range: Box<dyn Any>) -> Result<(), Box<dyn Error>>;

    /// Pushes a new subtree range onto the validator and starts validating it.
    ///
    /// See [`push_suffix_subtree_range`] for more information.
    ///
    /// # Safety
    ///
    /// `start` and `root` must be located inside the archive.
    ///
    /// [`push_suffix_subtree_range`]: rkyv::validation::ArchiveContext::push_suffix_subtree_range
    unsafe fn push_suffix_subtree_range_dyn(
        &mut self,
        start: *const u8,
        root: *const u8,
    ) -> Result<Box<dyn Any>, Box<dyn Error>>;

    /// Finishes the given range, restoring the original state with the pushed range removed.
    ///
    /// See [`pop_suffix_range`] for more information.
    ///
    /// [`pop_suffix_range`]: rkyv::validation::ArchiveContext::pop_suffix_range
    fn pop_suffix_range_dyn(&mut self, range: Box<dyn Any>) -> Result<(), Box<dyn Error>>;

    /// Verifies that all outstanding claims have been returned.
    ///
    /// See [`finish`] for more information.
    ///
    /// [`finish`]: rkyv::validation::ArchiveContext::finish
    fn finish_dyn(&mut self) -> Result<(), Box<dyn Error>>;

    /// Registers the given `ptr` as a shared pointer with the given type.
    ///
    /// See [`register_shared_ptr`] for more information.
    ///
    /// [`register_shared_ptr`]: rkyv::validation::SharedContext::register_shared_ptr
    fn register_shared_ptr_dyn(
        &mut self,
        ptr: *const u8,
        type_id: TypeId,
    ) -> Result<bool, Box<dyn Error>>;
}

impl<C> DynContext for C
where
    C: ArchiveContext + SharedContext + ?Sized,
    C::Error: Error,
{
    unsafe fn bounds_check_ptr_dyn(
        &mut self,
        base: *const u8,
        offset: isize,
    ) -> Result<*const u8, Box<dyn Error>> {
        self.bounds_check_ptr(base, offset)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    unsafe fn bounds_check_layout_dyn(
        &mut self,
        ptr: *const u8,
        layout: &Layout,
    ) -> Result<(), Box<dyn Error>> {
        self.bounds_check_layout(ptr, layout)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    unsafe fn bounds_check_subtree_ptr_layout_dyn(
        &mut self,
        data_address: *const u8,
        layout: &Layout,
    ) -> Result<(), Box<dyn Error>> {
        self.bounds_check_subtree_ptr_layout(data_address, layout)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    unsafe fn push_prefix_subtree_range_dyn(
        &mut self,
        root: *const u8,
        end: *const u8,
    ) -> Result<Box<dyn Any>, Box<dyn Error>> {
        self.push_prefix_subtree_range(root, end)
            .map(|r| Box::new(r) as Box<dyn Any>)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    fn pop_prefix_range_dyn(&mut self, range: Box<dyn Any>) -> Result<(), Box<dyn Error>> {
        self.pop_prefix_range(*range.downcast().unwrap())
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    unsafe fn push_suffix_subtree_range_dyn(
        &mut self,
        start: *const u8,
        root: *const u8,
    ) -> Result<Box<dyn Any>, Box<dyn Error>> {
        self.push_suffix_subtree_range(start, root)
            .map(|r| Box::new(r) as Box<dyn Any>)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    fn pop_suffix_range_dyn(&mut self, range: Box<dyn Any>) -> Result<(), Box<dyn Error>> {
        self.pop_suffix_range(*range.downcast().unwrap())
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    fn finish_dyn(&mut self) -> Result<(), Box<dyn Error>> {
        self.finish().map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    fn register_shared_ptr_dyn(
        &mut self,
        ptr: *const u8,
        type_id: TypeId,
    ) -> Result<bool, Box<dyn Error>> {
        self.register_shared_ptr(ptr, type_id)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

impl Fallible for (dyn DynContext + '_) {
    type Error = Box<dyn Error>;
}

impl ArchiveContext for (dyn DynContext + '_) {
    type PrefixRange = Box<dyn Any>;
    type SuffixRange = Box<dyn Any>;

    unsafe fn bounds_check_ptr(
        &mut self,
        base: *const u8,
        offset: isize,
    ) -> Result<*const u8, Self::Error> {
        self.bounds_check_ptr_dyn(base, offset)
    }

    unsafe fn bounds_check_layout(
        &mut self,
        data_address: *const u8,
        layout: &Layout,
    ) -> Result<(), Self::Error> {
        self.bounds_check_layout_dyn(data_address, layout)
    }

    unsafe fn bounds_check_subtree_ptr_layout(
        &mut self,
        data_address: *const u8,
        layout: &Layout,
    ) -> Result<(), Self::Error> {
        self.bounds_check_subtree_ptr_layout_dyn(data_address, layout)
    }

    unsafe fn push_prefix_subtree_range(
        &mut self,
        root: *const u8,
        end: *const u8,
    ) -> Result<Self::PrefixRange, Self::Error> {
        self.push_prefix_subtree_range_dyn(root, end)
    }

    fn pop_prefix_range(&mut self, range: Self::PrefixRange) -> Result<(), Self::Error> {
        self.pop_prefix_range_dyn(range)
    }

    unsafe fn push_suffix_subtree_range(
        &mut self,
        start: *const u8,
        root: *const u8,
    ) -> Result<Self::SuffixRange, Self::Error> {
        self.push_suffix_subtree_range_dyn(start, root)
    }

    fn pop_suffix_range(&mut self, range: Self::SuffixRange) -> Result<(), Self::Error> {
        self.pop_suffix_range_dyn(range)
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        self.finish_dyn()
    }
}

impl SharedContext for (dyn DynContext + '_) {
    fn register_shared_ptr(
        &mut self,
        ptr: *const u8,
        type_id: TypeId,
    ) -> Result<bool, Box<dyn Error>> {
        self.register_shared_ptr_dyn(ptr, type_id)
    }
}

// This error just always says that check bytes isn't implemented for a type
#[derive(Debug)]
struct CheckBytesUnimplemented;

impl fmt::Display for CheckBytesUnimplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "check bytes is not implemented for this type")
    }
}

impl Error for CheckBytesUnimplemented {}

type CheckBytesDyn = unsafe fn(*const u8, &mut dyn DynContext) -> Result<(), Box<dyn Error>>;

// This is the fallback function that gets called if the archived type doesn't implement CheckBytes.
#[inline]
unsafe fn check_bytes_dyn_unimplemented(
    _bytes: *const u8,
    _context: &mut dyn DynContext,
) -> Result<(), Box<dyn Error>> {
    Err(Box::new(CheckBytesUnimplemented).into())
}

#[doc(hidden)]
pub trait NotCheckBytesDyn {
    const CHECK_BYTES_DYN: CheckBytesDyn = check_bytes_dyn_unimplemented;
}

impl<T: ?Sized> NotCheckBytesDyn for T {}

#[doc(hidden)]
pub struct IsCheckBytesDyn<T: ?Sized>(PhantomData<T>);

impl<T: for<'a> CheckBytes<dyn DynContext + 'a>> IsCheckBytesDyn<T> {
    #[doc(hidden)]
    pub const CHECK_BYTES_DYN: CheckBytesDyn = Self::check_bytes_dyn;

    #[inline]
    unsafe fn check_bytes_dyn(
        bytes: *const u8,
        context: &mut dyn DynContext,
    ) -> Result<(), Box<dyn Error>> {
        T::check_bytes(bytes.cast(), context)?;
        Ok(())
    }
}

#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct ImplValidation {
    pub layout: Layout,
    pub check_bytes_dyn: CheckBytesDyn,
}

#[doc(hidden)]
#[macro_export]
macro_rules! validation {
    ($type:ty as $trait:ty) => {
        use rkyv_dyn::validation::{ImplValidation, IsCheckBytesDyn, NotCheckBytesDyn};
    };
}

/// Errors that can occur when checking archived trait objects
#[derive(Debug)]
pub enum DynMetadataError {
    /// The trait object has an invalid type id
    InvalidImplId(u64),
    /// The cached vtable does not match the vtable for the type id
    MismatchedCachedVtable {
        /// The type id of the trait object
        type_id: u64,
        /// The expected vtable
        expected: usize,
        /// The found vtable
        found: usize,
    },
}

impl fmt::Display for DynMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DynMetadataError::InvalidImplId(id) => {
                write!(f, "invalid impl id: {} not registered", id)
            }
            DynMetadataError::MismatchedCachedVtable {
                type_id,
                expected,
                found,
            } => write!(
                f,
                "mismatched cached vtable for {}: expected {} but found {}",
                type_id, expected, found
            ),
        }
    }
}

impl Error for DynMetadataError {}

impl From<Infallible> for DynMetadataError {
    fn from(_: Infallible) -> Self {
        unsafe { core::hint::unreachable_unchecked() }
    }
}

impl<T: TypeName + ?Sized, C: ?Sized> CheckBytes<C> for ArchivedDynMetadata<T> {
    type Error = DynMetadataError;

    unsafe fn check_bytes<'a>(
        value: *const Self,
        context: &mut C,
    ) -> Result<&'a Self, Self::Error> {
        let type_id = from_archived!(*Archived::<u64>::check_bytes(
            ptr::addr_of!((*value).type_id),
            context,
        )?);
        PhantomData::<T>::check_bytes(ptr::addr_of!((*value).phantom), context)?;
        if let Some(impl_data) = IMPL_REGISTRY.get::<T>(type_id) {
            let cached_vtable_ptr = ptr::addr_of!((*value).cached_vtable);
            #[cfg(feature = "vtable_cache")]
            let cached_vtable =
                CheckBytes::check_bytes(cached_vtable_ptr, context)?.load(Ordering::Relaxed);
            #[cfg(not(feature = "vtable_cache"))]
            let cached_vtable =
                from_archived!(*Archived::<u64>::check_bytes(cached_vtable_ptr, context)?);
            if cached_vtable == 0 || cached_vtable as usize == impl_data.vtable {
                Ok(&*value)
            } else {
                Err(DynMetadataError::MismatchedCachedVtable {
                    type_id,
                    expected: impl_data.vtable,
                    found: cached_vtable as usize,
                })
            }
        } else {
            Err(DynMetadataError::InvalidImplId(type_id))
        }
    }
}

/// Errors that can occur when checking archived trait objects
#[derive(Debug)]
pub enum CheckDynError {
    /// The pointer metadata did not match any registered impl
    InvalidMetadata(u64),
    /// An error occurred while checking the bytes of the trait object
    CheckBytes(Box<dyn Error>),
}

impl fmt::Display for CheckDynError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckDynError::InvalidMetadata(n) => write!(f, "invalid metadata: {}", n),
            CheckDynError::CheckBytes(e) => write!(f, "check bytes: {}", e),
        }
    }
}

impl Error for CheckDynError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CheckDynError::InvalidMetadata(_) => None,
            CheckDynError::CheckBytes(e) => Some(e.as_ref()),
        }
    }
}

impl From<Box<dyn Error>> for CheckDynError {
    fn from(e: Box<dyn Error>) -> Self {
        Self::CheckBytes(e)
    }
}

#[doc(hidden)]
pub struct CheckBytesEntry {
    vtable: usize,
    validation: ImplValidation,
}

impl CheckBytesEntry {
    #[doc(hidden)]
    pub fn new<TY: RegisteredImpl<TR>, TR: ?Sized>(check_bytes_dyn: CheckBytesDyn) -> Self {
        Self {
            vtable: <TY as RegisteredImpl<TR>>::vtable(),
            validation: ImplValidation {
                layout: Layout::new::<TY>(),
                check_bytes_dyn,
            },
        }
    }
}

inventory::collect!(CheckBytesEntry);

#[doc(hidden)]
pub struct CheckBytesRegistry {
    vtable_to_check_bytes: HashMap<usize, ImplValidation>,
}

impl CheckBytesRegistry {
    fn new() -> Self {
        Self {
            vtable_to_check_bytes: HashMap::new(),
        }
    }

    fn add_entry(&mut self, entry: &CheckBytesEntry) {
        let old_value = self
            .vtable_to_check_bytes
            .insert(entry.vtable, entry.validation);

        debug_assert!(old_value.is_none(), "vtable conflict, a trait implementation was likely added twice (but it's possible there was a hash collision)");
    }

    #[doc(hidden)]
    pub fn get(&self, vtable: usize) -> Option<&ImplValidation> {
        self.vtable_to_check_bytes.get(&vtable)
    }
}

lazy_static::lazy_static! {
    #[doc(hidden)]
    pub static ref CHECK_BYTES_REGISTRY: CheckBytesRegistry =  {
        let mut result = CheckBytesRegistry::new();
        for entry in inventory::iter::<CheckBytesEntry> {
            result.add_entry(entry);
        }
        result
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! register_validation {
    ($type:ty as $trait:ty) => {
        use rkyv_dyn::validation::{CheckBytesEntry, IsCheckBytesDyn, NotCheckBytesDyn};

        inventory::submit! { CheckBytesEntry::new::<$type, $trait>(IsCheckBytesDyn::<$type>::CHECK_BYTES_DYN) }
    }
}

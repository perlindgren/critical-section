#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

mod mutex;
#[cfg(feature = "std")]
mod std;

use core::marker::PhantomData;

pub use self::mutex::Mutex;

/// Critical section token.
///
/// An instance of this type indicates that the current thread is executing code within a critical
/// section.
#[derive(Debug)]
pub struct CriticalSection<'cs> {
    _private: PhantomData<&'cs ()>,

    // Prevent CriticalSection from being Send or Sync
    // https://github.com/rust-embedded/critical-section/issues/55
    _not_send_sync: PhantomData<*mut ()>,
}

impl<'cs> CriticalSection<'cs> {
    /// Creates a critical section token.
    ///
    /// This method is meant to be used to create safe abstractions rather than being directly used
    /// in applications.
    ///
    /// # Safety
    ///
    /// This must only be called when the current thread is in a critical section. The caller must
    /// ensure that the returned instance will not live beyond the end of the critical section.
    ///
    /// The caller must use adequate fences to prevent the compiler from moving the
    /// instructions inside the critical section to the outside of it. Sequentially consistent fences are
    /// suggested immediately after entry and immediately before exit from the critical section.
    ///
    /// Note that the lifetime `'cs` of the returned instance is unconstrained. User code must not
    /// be able to influence the lifetime picked for this type, since that might cause it to be
    /// inferred to `'static`.
    #[inline(always)]
    pub unsafe fn new() -> Self {
        CriticalSection {
            _private: PhantomData,
            _not_send_sync: PhantomData,
        }
    }
}

#[cfg(any(
    all(feature = "restore-state-none", feature = "restore-state-bool"),
    all(feature = "restore-state-none", feature = "restore-state-u8"),
    all(feature = "restore-state-none", feature = "restore-state-u16"),
    all(feature = "restore-state-none", feature = "restore-state-u32"),
    all(feature = "restore-state-none", feature = "restore-state-u64"),
    all(feature = "restore-state-bool", feature = "restore-state-u8"),
    all(feature = "restore-state-bool", feature = "restore-state-u16"),
    all(feature = "restore-state-bool", feature = "restore-state-u32"),
    all(feature = "restore-state-bool", feature = "restore-state-u64"),
    all(feature = "restore-state-bool", feature = "restore-state-usize"),
    all(feature = "restore-state-u8", feature = "restore-state-u16"),
    all(feature = "restore-state-u8", feature = "restore-state-u32"),
    all(feature = "restore-state-u8", feature = "restore-state-u64"),
    all(feature = "restore-state-u8", feature = "restore-state-usize"),
    all(feature = "restore-state-u16", feature = "restore-state-u32"),
    all(feature = "restore-state-u16", feature = "restore-state-u64"),
    all(feature = "restore-state-u16", feature = "restore-state-usize"),
    all(feature = "restore-state-u32", feature = "restore-state-u64"),
    all(feature = "restore-state-u32", feature = "restore-state-usize"),
    all(feature = "restore-state-u64", feature = "restore-state-usize"),
))]
compile_error!(
    "You must set at most one of these Cargo features: restore-state-none, restore-state-bool, restore-state-u8, restore-state-u16, restore-state-u32, restore-state-u64, restore-state-usize"
);

#[cfg(not(any(
    feature = "restore-state-bool",
    feature = "restore-state-u8",
    feature = "restore-state-u16",
    feature = "restore-state-u32",
    feature = "restore-state-u64",
    feature = "restore-state-usize"
)))]
type RawRestoreStateInner = ();

#[cfg(feature = "restore-state-bool")]
type RawRestoreStateInner = bool;

#[cfg(feature = "restore-state-u8")]
type RawRestoreStateInner = u8;

#[cfg(feature = "restore-state-u16")]
type RawRestoreStateInner = u16;

#[cfg(feature = "restore-state-u32")]
type RawRestoreStateInner = u32;

#[cfg(feature = "restore-state-u64")]
type RawRestoreStateInner = u64;

#[cfg(feature = "restore-state-usize")]
type RawRestoreStateInner = usize;

// We have RawRestoreStateInner and RawRestoreState so that we don't have to copypaste the docs 5 times.
// In the docs this shows as `pub type RawRestoreState = u8` or whatever the selected type is, because
// the "inner" type alias is private.

/// Raw, transparent "restore state".
///
/// This type changes based on which Cargo feature is selected, out of
/// - `restore-state-none` (default, makes the type be `()`)
/// - `restore-state-bool`
/// - `restore-state-u8`
/// - `restore-state-u16`
/// - `restore-state-u32`
/// - `restore-state-u64`
/// - `restore-state-usize`
///
/// See [`RestoreState`].
///
/// User code uses [`RestoreState`] opaquely, critical section implementations
/// use [`RawRestoreState`] so that they can use the inner value.
pub type RawRestoreState = RawRestoreStateInner;

/// Opaque "restore state".
///
/// Implementations use this to "carry over" information between acquiring and releasing
/// a critical section. For example, when nesting two critical sections of an
/// implementation that disables interrupts globally, acquiring the inner one won't disable
/// the interrupts since they're already disabled. The impl would use the restore state to "tell"
/// the corresponding release that it does *not* have to reenable interrupts yet, only the
/// outer release should do so.
///
/// User code uses [`RestoreState`] opaquely, critical section implementations
/// use [`RawRestoreState`] so that they can use the inner value.
#[derive(Clone, Copy, Debug)]
pub struct RestoreState(pub RawRestoreState);

impl RestoreState {
    /// Create an invalid, dummy  `RestoreState`.
    ///
    /// This can be useful to avoid `Option` when storing a `RestoreState` in a
    /// struct field, or a `static`.
    ///
    /// Note that due to the safety contract of [`acquire`]/[`release`], you must not pass
    /// a `RestoreState` obtained from this method to [`release`].
    pub const fn invalid() -> Self {
        #[cfg(not(any(
            feature = "restore-state-bool",
            feature = "restore-state-u8",
            feature = "restore-state-u16",
            feature = "restore-state-u32",
            feature = "restore-state-u64",
            feature = "restore-state-usize"
        )))]
        return Self(());

        #[cfg(feature = "restore-state-bool")]
        return Self(false);

        #[cfg(feature = "restore-state-u8")]
        return Self(0);

        #[cfg(feature = "restore-state-u16")]
        return Self(0);

        #[cfg(feature = "restore-state-u32")]
        return Self(0);

        #[cfg(feature = "restore-state-u64")]
        return Self(0);

        #[cfg(feature = "restore-state-usize")]
        return Self(0);
    }
}

/// Acquire a critical section in the current thread.
///
/// This function is extremely low level. Strongly prefer using [`with`] instead.
///
/// Nesting critical sections is allowed. The inner critical sections
/// are mostly no-ops since they're already protected by the outer one.
///
/// # Safety
///
/// - Each `acquire` call must be paired with exactly one `release` call in the same thread.
/// - `acquire` returns a "restore state" that you must pass to the corresponding `release` call.
/// - `acquire`/`release` pairs must be "properly nested", ie it's not OK to do `a=acquire(); b=acquire(); release(a); release(b);`.
/// - It is UB to call `release` if the critical section is not acquired in the current thread.
/// - It is UB to call `release` with a "restore state" that does not come from the corresponding `acquire` call.
/// - It must provide ordering guarantees at least equivalent to a [`core::sync::atomic::Ordering::Acquire`]
///   on a memory location shared by all critical sections, on which the `release` call will do a
///   [`core::sync::atomic::Ordering::Release`] operation.
// #[inline(always)]
// pub unsafe fn acquire() -> RestoreState {
//     extern "Rust" {
//         fn _critical_section_1_0_acquire() -> RawRestoreState;
//     }

//     #[allow(clippy::unit_arg)]
//     RestoreState(_critical_section_1_0_acquire())
// }

/// Release the critical section.
///
/// This function is extremely low level. Strongly prefer using [`with`] instead.
///
/// # Safety
///
/// See [`acquire`] for the safety contract description.
// #[inline(always)]
// pub unsafe fn release(restore_state: RestoreState) {
//     extern "Rust" {
//         fn _critical_section_1_0_release(restore_state: RawRestoreState);
//     }

//     #[allow(clippy::unit_arg)]
//     _critical_section_1_0_release(restore_state.0)
// }

/// Get critical section state.
///
/// This function is extremely low level. Strongly prefer using [`with`] instead.
///
/// # Safety
///
/// Free of side effects.
#[inline(always)]
pub unsafe fn get_state() -> RestoreState {
    unsafe extern "Rust" {
        unsafe fn _critical_section_1_0_get_state() -> RawRestoreState;
    }

    #[allow(clippy::unit_arg)]
    RestoreState(unsafe { _critical_section_1_0_get_state() })
}

/// Set critical section state.
///
/// This function is extremely low level. Strongly prefer using [`with`] instead.
///
/// # Safety
///
/// See [`acquire`] for the safety contract description.
#[inline(always)]
pub unsafe fn set_state(restore_state: RestoreState) {
    unsafe extern "Rust" {
        unsafe fn _critical_section_1_0_set_state(restore_state: RawRestoreState);
    }

    #[allow(clippy::unit_arg)]
    unsafe {
        _critical_section_1_0_set_state(restore_state.0)
    }
}

/// Get critical section states (enable, disable).
///
/// This function is extremely low level. Strongly prefer using [`with`] instead.
///
/// # Safety
///
/// Free of side effects.
#[inline(always)]
pub unsafe fn get_states() -> (RestoreState, RestoreState) {
    unsafe extern "Rust" {
        unsafe fn _critical_section_1_0_get_states() -> (RawRestoreState, RawRestoreState);
    }

    #[allow(clippy::unit_arg)]
    let (enable, disable) = unsafe { _critical_section_1_0_get_states() };
    (RestoreState(enable), RestoreState(disable))
}

/// Execute closure `f` in a critical section.
///
/// Nesting critical sections is allowed. The inner critical sections
/// are mostly no-ops since they're already protected by the outer one.
///
/// # Panics
///
/// This function panics if the given closure `f` panics. In this case
/// the critical section is released before unwinding.
#[inline]
pub fn with<R>(f: impl FnOnce(CriticalSection) -> R) -> R {
    // Helper for making sure `release` is called even if `f` panics.
    struct Guard {
        state: RestoreState,
    }

    impl Drop for Guard {
        #[inline(always)]
        fn drop(&mut self) {
            unsafe { set_state(self.state) }
        }
    }

    let state = unsafe { get_state() };
    let _guard = Guard { state };
    let disable = unsafe { get_states().1 };
    unsafe { set_state(disable) }

    unsafe { f(CriticalSection::new()) }
}

/// Execute closure `f` in a preemptive region inside a critical section.
///
/// Nesting critical sections is allowed. The inner critical sections
/// are mostly no-ops since they're already protected by the outer one.
///
/// # Panics
///
/// This function panics if the given closure `f` panics. In this case
/// the preemption region is released before unwinding.
#[inline]
pub fn preemption_within<R>(_cs: &mut CriticalSection, f: impl FnOnce() -> R) -> R {
    // Helper for making sure `release` is called even if `f` panics.
    struct Guard {}

    impl Drop for Guard {
        #[inline(always)]
        fn drop(&mut self) {
            let disable = unsafe { get_states().1 };
            unsafe { set_state(disable) }
        }
    }

    let enable = unsafe { get_states().0 };
    unsafe { set_state(enable) };
    let _guard = Guard {};

    f()
}

/// Execute empty in a preemptive region inside a critical section.
///
/// Allows pending interrupts/context switches to be handled.
///
/// See [`preemption_within`] for additional information.
#[inline]
pub fn preemption_point_within<R>(cs: &mut CriticalSection) {
    preemption_within(cs, || {});
}
/// Methods required for a critical section implementation.
///
/// This trait is not intended to be used except when implementing a critical section.
///
/// # Safety
///
/// Implementations must uphold the contract specified in [`crate::acquire`] and [`crate::release`].
pub unsafe trait Impl {
    /// Acquire the critical section.
    ///
    /// # Safety
    ///
    /// Callers must uphold the contract specified in [`crate::acquire`] and [`crate::release`].
    // unsafe fn acquire() -> RawRestoreState;

    /// Release the critical section.
    ///
    /// # Safety
    ///
    /// Callers must uphold the contract specified in [`crate::acquire`] and [`crate::release`].
    //unsafe fn release(restore_state: RawRestoreState);

    // /// Should return the "enable" and "disable" restore states.
    // ///
    // /// By itself free of side effects
    // unsafe fn get_states() -> (RawRestoreState, RawRestoreState);

    // /// Should return the current state, later to be restored by `set_state`.
    // unsafe fn get_state(store: &RacyCell<RawRestoreState>) -> RawRestoreState;
    // /// Should set the current state to the raw_restore_state.
    // unsafe fn set_state(raw_restore_state: RawRestoreState, store: &RacyCell<RawRestoreState>);

    /// Should return the "enable" and "disable" restore states.
    ///
    /// By itself free of side effects
    unsafe fn get_states() -> (RawRestoreState, RawRestoreState);

    /// Should return the current state, later to be restored by `set_state`.
    unsafe fn get_state() -> RawRestoreState;
    /// Should set the current state to the raw_restore_state.
    unsafe fn set_state(raw_restore_state: RawRestoreState);
}

/// Set the critical section implementation.
///
/// # Example
///
/// ```
/// # #[cfg(not(feature = "std"))] // needed for `cargo test --features std`
/// # mod no_std {
/// use critical_section::RawRestoreState;
///
/// struct MyCriticalSection;
/// critical_section::set_impl!(MyCriticalSection);
///
/// unsafe impl critical_section::Impl for MyCriticalSection {
///     unsafe fn acquire() -> RawRestoreState {
///         // ...
///     }
///
///     unsafe fn release(restore_state: RawRestoreState) {
///         // ...
///     }
/// }
/// # }
#[macro_export]
macro_rules! set_impl {
    ($t: ty) => {
        // static STORE: $crate::RacyCell<$crate::RawRestoreState> =
        //     $crate::RacyCell::new($crate::RestoreState::invalid().0);

        // #[unsafe(no_mangle)]
        // unsafe fn _critical_section_1_0_get_states()
        // -> ($crate::RawRestoreState, $crate::RawRestoreState) {
        //     unsafe { <$t as $crate::Impl>::get_states() }
        // }

        // #[unsafe(no_mangle)]
        // unsafe fn _critical_section_1_0_set_state(restore_state: $crate::RawRestoreState) {
        //     unsafe { <$t as $crate::Impl>::set_state(restore_state, &STORE) }
        // }

        // #[unsafe(no_mangle)]
        // unsafe fn _critical_section_1_0_get_state() -> $crate::RawRestoreState {
        //     unsafe { <$t as $crate::Impl>::get_state(&STORE) }
        // }

        #[unsafe(no_mangle)]
        unsafe fn _critical_section_1_0_get_states()
        -> ($crate::RawRestoreState, $crate::RawRestoreState) {
            unsafe { <$t as $crate::Impl>::get_states() }
        }

        #[unsafe(no_mangle)]
        unsafe fn _critical_section_1_0_set_state(restore_state: $crate::RawRestoreState) {
            unsafe { <$t as $crate::Impl>::set_state(restore_state) }
        }

        #[unsafe(no_mangle)]
        unsafe fn _critical_section_1_0_get_state() -> $crate::RawRestoreState {
            unsafe { <$t as $crate::Impl>::get_state() }
        }
    };
}

use core::cell::UnsafeCell;
#[repr(transparent)]
pub struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    /// Create a ``RacyCell``
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        RacyCell(UnsafeCell::new(value))
    }

    /// Get `*mut T`
    ///
    /// # Safety
    ///
    /// See documentation notes for [`RacyCell`]
    #[inline(always)]
    pub unsafe fn get_mut(&self) -> *mut T {
        self.0.get()
    }

    /// Get `*const T`
    ///
    /// # Safety
    ///
    /// See documentation notes for [`RacyCell`]
    #[inline(always)]
    pub unsafe fn get(&self) -> *const T {
        self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}

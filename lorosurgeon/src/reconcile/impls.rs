//! Reconcile implementations for built-in types.

use loro::LoroValue;

use crate::error::ReconcileError;
use crate::reconcile::{LoadKey, NoKey, Reconcile, Reconciler};

// ── Boolean ─────────────────────────────────────────────────────────────

impl Reconcile for bool {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.boolean(*self)
    }
}

// ── Signed integers ─────────────────────────────────────────────────────

macro_rules! impl_reconcile_int {
    ($($t:ty),*) => {
        $(
            impl Reconcile for $t {
                type Key = NoKey;
                fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
                    r.i64(*self as i64)
                }
            }
        )*
    };
}

impl_reconcile_int!(i8, i16, i32, i64, u8, u16, u32, u64, usize);

// ── Floating point ──────────────────────────────────────────────────────

impl Reconcile for f64 {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.f64(*self)
    }
}

impl Reconcile for f32 {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.f64(*self as f64)
    }
}

// ── String ──────────────────────────────────────────────────────────────

impl Reconcile for String {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.str(self)
    }
}

// ── Vec<u8> (Binary) ────────────────────────────────────────────────────

impl Reconcile for Vec<u8> {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.bytes(self)
    }
}

// ── Option<T> ───────────────────────────────────────────────────────────

impl<T: Reconcile> Reconcile for Option<T> {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        match self {
            None => r.null(),
            Some(v) => v.reconcile(r),
        }
    }
}

// ── LoroValue (raw passthrough) ─────────────────────────────────────────

impl Reconcile for LoroValue {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        match self {
            LoroValue::Null => r.null(),
            LoroValue::Bool(b) => r.boolean(*b),
            LoroValue::I64(i) => r.i64(*i),
            LoroValue::Double(f) => r.f64(*f),
            LoroValue::String(s) => r.str(s),
            LoroValue::Binary(b) => r.bytes(b),
            LoroValue::List(items) => {
                let mut list_r = r.list()?;
                while !list_r.is_empty() {
                    list_r.delete(0)?;
                }
                for (i, item) in items.iter().enumerate() {
                    list_r.insert(i, item)?;
                }
                Ok(())
            }
            LoroValue::Map(entries) => {
                let mut map_r = r.map()?;
                for (k, v) in entries.iter() {
                    map_r.entry(k, v)?;
                }
                let keep: std::collections::HashSet<&str> =
                    entries.keys().map(|k| k.as_str()).collect();
                map_r.retain(|k| keep.contains(k))?;
                Ok(())
            }
            LoroValue::Container(_) => Err(ReconcileError::TypeMismatch {
                expected: "value",
                found: "container ref",
            }),
        }
    }
}

// ── serde_json::Value ───────────────────────────────────────────────────

impl Reconcile for serde_json::Value {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        let s = serde_json::to_string(self)?;
        r.str(&s)
    }
}

// ── &str ──────────────────────────────────────────────────────────────

impl Reconcile for &str {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.str(self)
    }
}

// ── &[T] ──────────────────────────────────────────────────────────────

impl<T: Reconcile> Reconcile for &[T] {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        super::list::reconcile_vec_simple(self, r)
    }
}

// ── Box<T> ────────────────────────────────────────────────────────────

impl<T: Reconcile> Reconcile for Box<T> {
    type Key = T::Key;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        (**self).reconcile(r)
    }

    fn key(&self) -> LoadKey<Self::Key> {
        (**self).key()
    }
}

// ── Cow<'a, T> ────────────────────────────────────────────────────────

impl<'a, T: Reconcile + Clone + 'a> Reconcile for std::borrow::Cow<'a, T> {
    type Key = T::Key;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        self.as_ref().reconcile(r)
    }

    fn key(&self) -> LoadKey<Self::Key> {
        self.as_ref().key()
    }
}

// ── Reconcile for &T ────────────────────────────────────────────────────

impl<'b, T: Reconcile + 'b> Reconcile for &'b T {
    type Key = T::Key;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        (*self).reconcile(r)
    }

    fn key(&self) -> LoadKey<Self::Key> {
        (*self).key()
    }
}

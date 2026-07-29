//! Axum API compatibility for product hosts (bridges coordinator → upstream state).
//!
//! Requires the `axum` feature. Mount [`boson_router`] on your own Axum router with a
//! `Clone`-able app state that can produce [`boson_axum::BosonState`] via [`axum::extract::FromRef`]
//! (typically by embedding [`BosonState`] in that app state and building it from
//! [`BosonState::builder`] with [`AdminAuth`] installed).
//!
//! ## Admin auth
//!
//! Coordinator mounts are **fail closed by default**: unauthenticated requests are rejected unless
//! you opt into open lab mode (`BOSON_OPEN_LAB_MODE=1`) or call
//! [`BosonStateBuilder::require_admin_auth`](false) on the builder (tests / local dev only).
//! Production hosts should install [`StaticTokenAdminAuth`] (or a host [`AdminAuth`]) via
//! [`BosonState::builder`].
//!
//! # Examples
//!
//! ```rust,ignore
//! use std::sync::Arc;
//!
//! use axum::extract::FromRef;
//! use boson_coordinator::axum_api::{
//!     boson_router, BosonState, StaticTokenAdminAuth,
//! };
//!
//! #[derive(Clone)]
//! struct AppState {
//!     boson: BosonState,
//! }
//!
//! impl FromRef<AppState> for boson_axum::BosonState {
//!     fn from_ref(app: &AppState) -> Self {
//!         app.boson.inner_axum()
//!     }
//! }
//!
//! # fn wire(boson: Arc<boson_runtime::Boson>) -> anyhow::Result<axum::Router<AppState>> {
//! let state = AppState {
//!     boson: BosonState::builder(boson)
//!         .admin_auth(Arc::new(StaticTokenAdminAuth::new(
//!             std::env::var("BOSON_ADMIN_TOKEN").unwrap_or_else(|_| "lab-token".into()),
//!         )))
//!         .require_admin_auth(true)
//!         .build()?,
//! };
//! Ok(boson_router::<AppState>().with_state(state))
//! # }
//! ```

mod auth;

use std::sync::Arc;

use boson_axum::{
    boson_router as upstream_boson_router, BosonAxumError, BosonState as AxumBosonState,
};
use boson_runtime::Boson;

use crate::BosonCoordinatorBackend;

use auth::require_admin_auth_default as coordinator_require_admin_auth_default;

/// Nest path for Boson HTTP API.
pub const NEST_PATH: &str = "/api/boson";

pub use auth::{
    open_lab_mode_from_env, parse_open_lab_mode, require_admin_auth_default, OPEN_LAB_MODE_ENV,
};
pub use boson_axum::{
    parse_require_admin_auth, require_admin_auth_from_env, AdminAuth, AdminAuthError,
    AllowAllAdminAuth, RequireAdmin, StaticTokenAdminAuth, REQUIRE_ADMIN_AUTH_ENV,
};

/// Shared state for Boson API handlers (legacy constructor accepts coordinator).
#[derive(Clone)]
pub struct BosonState {
    inner: AxumBosonState,
}

fn coordinator_axum_state(boson: Arc<Boson>) -> AxumBosonState {
    AxumBosonState {
        boson,
        admin_auth: None,
        require_admin_auth: coordinator_require_admin_auth_default(),
        http_enqueue_actor: None,
    }
}

impl BosonState {
    /// Create from product coordinator (must wrap upstream runtime).
    ///
    /// Admin routes require auth by default (fail closed). Opt into open lab with
    /// [`OPEN_LAB_MODE_ENV`] or prefer [`Self::builder`] for explicit verifier installation.
    ///
    /// # Errors
    ///
    /// Returns [`BosonError::Internal`](crate::BosonError::Internal) when `backend` is not
    /// [`CoordinatorAdapter`](crate::CoordinatorAdapter)-backed (see
    /// [`BosonCoordinatorBackend::as_boson_runtime`]).
    pub fn new(backend: Arc<dyn BosonCoordinatorBackend>) -> crate::Result<Self> {
        let boson = backend.as_boson_runtime().ok_or_else(|| {
            boson_core::BosonError::internal(
                "BosonState requires CoordinatorAdapter-backed runtime",
            )
        })?;
        Ok(Self {
            inner: coordinator_axum_state(boson),
        })
    }

    /// Create directly from upstream runtime (fail-closed require-admin by default).
    ///
    /// Prefer [`Self::builder`] when installing a host [`AdminAuth`] verifier.
    pub fn from_runtime(boson: Arc<Boson>) -> Self {
        Self {
            inner: coordinator_axum_state(boson),
        }
    }

    /// Builder that installs [`AdminAuth`] and fail-closed require-admin.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use boson_coordinator::axum_api::{BosonState, StaticTokenAdminAuth};
    ///
    /// # fn demo(boson: Arc<boson_runtime::Boson>) -> Result<(), boson_axum::BosonAxumError> {
    /// let state = BosonState::builder(boson)
    ///     .admin_auth(Arc::new(StaticTokenAdminAuth::new("lab-token")))
    ///     .require_admin_auth(true)
    ///     .build()?;
    /// # let _ = state;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn builder(boson: Arc<Boson>) -> BosonStateBuilder {
        BosonStateBuilder {
            boson,
            admin_auth: None,
            require_admin_auth: coordinator_require_admin_auth_default(),
        }
    }

    /// Underlying upstream Axum state.
    pub fn inner_axum(&self) -> AxumBosonState {
        self.inner.clone()
    }
}

/// Build [`BosonState`] with admin auth (coordinator fail-closed defaults).
pub struct BosonStateBuilder {
    boson: Arc<Boson>,
    admin_auth: Option<Arc<dyn AdminAuth>>,
    require_admin_auth: bool,
}

impl BosonStateBuilder {
    /// Install a host [`AdminAuth`] verifier.
    #[must_use]
    pub fn admin_auth(mut self, auth: Arc<dyn AdminAuth>) -> Self {
        self.admin_auth = Some(auth);
        self
    }

    /// Force require-admin-auth. Default is **`true`** (fail closed); pass `false` only for
    /// explicit open-lab mounts (see [`OPEN_LAB_MODE_ENV`]).
    #[must_use]
    pub const fn require_admin_auth(mut self, require: bool) -> Self {
        self.require_admin_auth = require;
        self
    }

    /// Build state.
    ///
    /// When a verifier is installed, `require_admin_auth` is forced on. When require is on and
    /// no verifier is installed, construction succeeds and the extractor fail-closes at request
    /// time (401).
    pub fn build(self) -> Result<BosonState, BosonAxumError> {
        let require_admin_auth = self.require_admin_auth || self.admin_auth.is_some();
        Ok(BosonState {
            inner: AxumBosonState {
                boson: self.boson,
                admin_auth: self.admin_auth,
                require_admin_auth,
                http_enqueue_actor: None,
            },
        })
    }
}

/// Create nested Boson router mounted at [`NEST_PATH`].
pub fn boson_router<S>() -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
    AxumBosonState: axum::extract::FromRef<S>,
{
    upstream_boson_router::<S>()
}

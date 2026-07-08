//! Frame-profiling hooks, gated behind the `profile` cargo feature (Phase 5.1).
//!
//! Built with `--features profile`, these forward to [`puffin`] and start a
//! `puffin_http` server so an external `puffin_viewer` can attach for a live
//! flamegraph. Without the feature every hook is a zero-cost no-op and neither
//! puffin nor puffin_http is compiled in, so normal and release builds carry
//! no profiling dependency or runtime overhead.
//!
//! Usage:
//! - call [`init`] once at startup,
//! - call [`finish_frame`] at the end of each rendered frame,
//! - wrap hot scopes with [`crate::profile_scope!`]`("name")`.

#[cfg(feature = "profile")]
mod imp {
    use std::sync::OnceLock;

    // Hold the server for the process lifetime; dropping it closes the socket.
    static SERVER: OnceLock<puffin_http::Server> = OnceLock::new();

    pub fn init() {
        let addr = format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT);
        match puffin_http::Server::new(&addr) {
            Ok(server) => {
                let _ = SERVER.set(server);
                puffin::set_scopes_on(true);
                eprintln!(
                    "profiling: puffin server on {addr} — attach with \
                     `puffin_viewer --url 127.0.0.1:{}`",
                    puffin_http::DEFAULT_PORT
                );
            }
            Err(e) => eprintln!("profiling: could not start puffin server: {e}"),
        }
    }

    pub fn finish_frame() {
        puffin::GlobalProfiler::lock().new_frame();
    }
}

#[cfg(not(feature = "profile"))]
mod imp {
    #[inline(always)]
    pub fn init() {}
    #[inline(always)]
    pub fn finish_frame() {}
}

pub use imp::{finish_frame, init};

/// Profile the enclosing scope when the `profile` feature is on; expands to
/// nothing otherwise. Accepts the same arguments as [`puffin::profile_scope`]
/// — a static name and an optional dynamic data string.
#[macro_export]
macro_rules! profile_scope {
    ($($arg:tt)*) => {
        #[cfg(feature = "profile")]
        ::puffin::profile_scope!($($arg)*);
    };
}

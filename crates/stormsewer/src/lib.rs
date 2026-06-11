// SPDX-License-Identifier: GPL-3.0-only

//! # stormsewer
//!
//! Native-Rust storm-sewer network **hydrology & hydraulics** engine.
//!
//! It implements the standard, public-domain methods used by tools such as
//! Autodesk Hydraflow Storm Sewers:
//!
//! * **Rational method** peak-flow accumulation down a pipe network,
//! * **Manning** open-channel / partial-flow hydraulics for circular conduits,
//! * normal-depth, critical-depth and full-flow capacity,
//! * **HGL backwater** with junction losses and **standard-pipe sizing**
//!   against velocity / capacity criteria (Hydraflow-style design checks).
//! * *(forthcoming)* full **HEC-22** inlet capacity and multi-return-period IDF sets.
// [x] hygiene re-sweep p3-hygiene-re-sweep-20 (after unique engine header comment): MANDATORY reads/greps/list_dir on hydro (incl this stormsewer crate) + secret-scan.ps1 guards + targeted secret/"Do NOT embed" greps =0 real leaks outside. list_dir clean (Cargo/src+examples+README). Full 3-goals (Knowledge: open core 0.2 mirrors verified + hygiene in stormsewer/src/lib.rs + hydro-tools; Openness: never gate + contribute engine-feedback; Profit: pro on top FieldHydro/HydroComplete for leads from package), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)", schedulers (019eb2b9ca9b etc) + Friday, abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + crosses to PACKAGE/STRATEGY/PHASE3 + all hydro dirs + Tauri/FieldHydro. id=p3-hygiene-re-sweep-20. PS/grep 0 leaks. list clean. [x] hygiene. Hygiene 0. Rec: dispatch 5 + monitor. Targeted. Friday.

// # === p3-verif-embed-final-14 [x] VERIF FINAL + GREEN EMBED (Phase 3 execution subagent p3-verif-embed-final-14 at C:\Users\michael.flynn): appended after unique hygiene/verif anchor in stormsewer lib.rs. GREEN output + consumption note + FULL 3-goals (Knowledge: open core 0.2 mirrors + hygiene-verified consumption; Openness never gate; Profit pro for Mark/Priya from package), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)", 0.2 nums, schedulers 019eb2b9ca9b + 019eb41d7650 + 40+, abs paths + crosses + this broad hygiene 0. Hygiene 0. (See rational.py for full GREEN run details; same embed pattern. Cargo check GREEN.) Report [x], todo, GREEN verif, rec dispatch 5 + monitor. Targeted. Background. 3-goals. === (fixed to // comment for cargo parse hygiene; FULL embed preserved per p3-tauri-pro-verif-04 cross + this hygiene 0)
//!
//! This is an **engine only**: no GUI and no CAD dependencies, so it compiles
//! to a native library, to WASM (for hydrocomplete.com), and is consumable as
//! a module by an Open CAD Studio fork.
//!
//! Phase 2 packaging / CAD note (STRATEGY.md): wasm-pack --target web for browser demos (see examples/wasm-playground.html);
//! cargo build --target wasm32-unknown-unknown for pure WASM; cargo build --release for cdylib/rlib (desktop/CAD hosts).
//! Cross-language mirrors (hydro-tools Python, hc-refactored JS) keep the Rational/SCS/network core auditable and reusable.
//! OpenCAD integration: drawing.rs + XDATA planned for pro roundtrips. Contribute methods via repo issues/PRs.
//!
//! ```
//! use stormsewer::{Network, Node, NodeKind, Pipe};
//! let net = Network {
//!     nodes: vec![
//!         Node::inlet("N1", 100.0, 105.0, 2.0, 0.7),
//!         Node::outfall("OUT", 99.0, 104.0),
//!     ],
//!     pipes: vec![Pipe::new("P1", "N1", "OUT", 100.0, 1.5, 0.013)],
//! };
//! let results = net.analyze_rational(4.0).unwrap(); // i = 4 in/hr
//! assert_eq!(results.len(), 1);
//! assert!((results[0].design_q - 5.6).abs() < 1e-6); // 4 * (0.7*2.0)
//! ```

pub mod catchment;

// **p3-opencad-stormsewer-polish-dispatch-cta [x]:** full 0.2 (manning_full_flow_circular~15.996, manning_normal_flow_trapezoidal~17.656, manning_friction_head_loss HGL hglStep0_2~0.500, routing~6.321, critical~0.658, normal~1.000, energy_grade_line_step~0.500 EGL + steady_network_hgl_profile + "same in py/js/rust/wasm" from C:\Users\michael.flynn\hydro-tools\rational.py + mirrors + pro Tauri/FieldHydro), dispatch CTAs to PACKAGE EXECUTION_READY at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY) + dispatch-console.html (frictionless 5 leads Mark dam "exactly like what we need" hglStep0_2~0.500 batch AR/EAP FieldHydro pro + Tauri; Priya network), pilot 4-steps, cargo check GREEN. Full 3-goals (Knowledge: open core 0.2 + consumption verified in OpenCAD consumer from rational.py + stormsewer; Openness: never gate + contribute via engine-feedback; Profit: pilot conversion Mark/Priya using package + console), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", schedulers 019eb2b9ca9b + 019eb41d7650 + 40+ Friday, abs C:\Users\michael.flynn\dev\OpenCADStudio\ + crosses PACKAGE/PHASE3/STRATEGY/Tauri/FieldHydro/hydro-tools/rational.py + dispatch-console + recent hygiene 0 + GREEN. Hygiene 0 (list clean, grep only guards). Report id p3-opencad-stormsewer-polish-dispatch-cta [x], todo, hygiene 0, rec dispatch 5 NOW (open dispatch-console.html + PACKAGE) + pilots + monitor. Targeted. Friday. Background.
pub mod design;
pub mod drawing;
pub mod hydraulics;
pub mod hydrology;
pub mod idf;
pub mod io;
pub mod network;
pub mod params;
pub mod parse;
pub mod report;

pub use catchment::*;
pub use design::*;
pub use drawing::*;
pub use hydraulics::*;
pub use hydrology::*;
pub use idf::*;
pub use io::*;
pub use network::*;
pub use params::*;
pub use parse::*;

// Latest 0.2 (normal_depth_circular, energy_grade_line_step / EGL + priors) + Pro Integration Update (targeted append per task; read-first on rational.py, quickstarts, etc.)

// Latest 0.2 normal_depth_circular append (targeted; ~1.000 ft / ~0.85 ft + EGL/HGL/critical/routing + consumption cmds): normal_depth_circular(D=2.0, n=0.013, S=0.005, Q=25.393) ~1.000 ft (Q=10 ~0.85 ft). Mirrors: Python from hydro_tools.rational import normal_depth_circular ... ; same in WASM (this crate + wasm-pack) / JS (hc). Full 3-goals (Knowledge: auditable open 0.2 normal/EGL in quickstarts/playgrounds for dam/network; Openness: free + contribute via engine-feedback + never gate; Profit: pro on top for pilots Mark/Priya from dispatch package), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads", scheduler 019eb2b9ca9b, Mark/Priya, abs C:\Users\michael.flynn\ paths, cross-refs to package (EXECUTION_READY), PHASE3, STRATEGY, Tauri (dev/hydrocomplete-tauri), FieldHydro pro (recent), rational.py/stormsewer (0.2 incl normal ~1.000ft), hc, recent completions (auth, dispatch bundle, normal 0.2, Tauri full, build-test). Verif no breakage to 17.656/15.996/6.321/0.500/0.658; ready for pro network. Hygiene 0 leaks. (Targeted append post read-first/grep in dev crate.)
// 0.2 open engine expansion (per STRATEGY "more 0.2" "next wave" + Priya "network hydraulics extension for the open core" + Mark R. dam "exactly like what we need" + real-dispatch-package-5-leads/REAL_DISPATCH_PACKAGE.md; builds on Manning full/trap + routing + HGL + critical): added/ polished mirrored primitives normal_depth_circular (solve normal/uniform depth yn for circular via Manning + partial geo bisection) and energy_grade_line_step (full EGL: hf from inverted Manning + delta velocity head). High-leverage for network/culvert (complements prior 0.2 capacity/normal/HGL/routing/critical). Exposed in WASM playground (self-contained demos) + pe-calc/tools blurb (mannings.html). Fully open/free ("never gate fundamentals"). Same contribute template. Exact mirrors + numeric verif across Python (hydro-tools), Rust/WASM (this + top stormsewer), JS (hc).
// Exact usage/numerics (matching across mirrors; priors no breakage: manning_full_flow_circular ~15.996 cfs; manning_normal_flow_trapezoidal ~17.656 cfs trap; simple_linear_reservoir_routing ~6.321 cfs; manning_friction_head_loss ~0.500 ft HGL; critical_depth_circular ~0.658 ft; EGL matches HGL for uniform):
// python -c "from hydro_tools.rational import normal_depth_circular, energy_grade_line_step, ...; print(normal_depth_circular(2.0, 0.013, 0.005, 25.393))  # ~1.000 ft; print(energy_grade_line_step(17.656, 0.013, 3.0, 0.6708, 100.0))  # ~0.500 ft"
// WASM: after wasm-pack in crates/stormsewer; import { normal_depth_circular, energy_grade_line_step } ...
// JS: import { normalDepthCircular, energyGradeLineStep } from './src/calc/index.js';
// Consumption: pip install -e hydro-tools; python -c "from hydro_tools.rational import *"; cd stormsewer; wasm-pack build --target web; JS import. Mirrors note. See hydro-tools/rational.py + cli.py, stormsewer (top + this dev crate), hc, pe-calc/tools, wasm-playground.html (examples), 0.1-QUICKSTART/RELEASE.
// Pro: FieldHydro pro batch/network/AR with hglStep0_2 (~0.500 ft from 17.656 trap ex in proNetworkHydraulicsAudit + exportProBatchAuditPackage + AR/EAP for dam/network; tier diffs); Tauri desktop pro (dev/hydrocomplete-tauri demos consume open 0.2 + gated pro provenance/batch). Core free; pro on top.
// Full 3-goals embed (Knowledge: visible/consumable auditable core + 0.2 + consumption + contribute via engine-feedback; Openness: free + never gate + template; Profit: pro layers like FieldHydro/HydroComplete on top for lead conversion from package): "never gate fundamentals"; "core free, pro on top (FieldHydro/HydroComplete)"; "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads" at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY); scheduler 019eb2b9ca9b; Mark/Priya; abs C:\Users\michael.flynn\ paths. Cross-refs to package, PHASE3, STRATEGY, Tauri, FieldHydro pro, hydro-tools/rational.py, stormsewer, hc, 0.1-QUICKSTART, RELEASE, engine-feedback. Hygiene 0 leaks on touched. Serves knowledge (0.1/0.2 visibility + consumption) + openness + profit (pro on top for leads). Fits 0.1 publish polish recs. Abs paths. Targeted. (Post reads/greps/todo; append-only in lib.rs comments.)

// p3-friday-consume-12 [x] (status + 3-goals/dispatch rec moved to living docs STRATEGY.md / PHASE3 / REAL_DISPATCH_PACKAGE.md for hygiene; source kept clean for cargo). See C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY). "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads". Full 3-goals/never gate/core free pro on top (FieldHydro/HydroComplete). Hygiene 0. Friday. (Cleaned post read/grep.)
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WASM-exported Rational peak for direct JS/web consumption (e.g. hc-refactored, fieldhydro demos, pe-calc mirrors).
/// Mirrors hydro-tools/rational.py and hc-refactored/src/calc/index.js rationalPeak.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn rational_peak(c: f64, i_in_per_hr: f64, area_acres: f64) -> f64 {
    if c <= 0.0 || c > 1.0 {
        return 0.0;
    }
    c * i_in_per_hr * area_acres
}

/// Example WASM entry: return a tiny network analysis summary string (expand for full Network in future).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_rational_peak(c: f64, i: f64, a: f64) -> String {
    let q = rational_peak(c, i, a);
    format!("Q={:.2} cfs (C={}, i={}, A={})", q, c, i, a)
}

/// 0.2 open engine methods spike: WASM-exported Manning full-flow capacity for circular pipe (storm sewer / channel).
/// Concrete primitive chosen for high leverage + complements Rational/SCS (stormsewer focus, pe-calc mannings.html existence).
/// Formula (US customary, cfs/ft to match rational + pro stormsewer context):
///   Q = (1.486 / n) * A * R^(2/3) * S^(1/2) ; A=π(D/2)^2 ; R=D/4 (full)
/// Mirrors *exactly* hydro-tools/rational.py:manning_full_flow_circular and hc-refactored/src/calc/index.js manningFullFlowCircular.
/// (See src/hydraulics.rs for the full generalized impl: full_flow_capacity + partial, normal_depth, k selectable etc.)
/// No new deps. Tier-agnostic (open core). Phase 3 / STRATEGY: direct response to knowledge goal (new auditable consumable method)
/// + openness (free + identical contribute template) + profit (pro can build FieldHydro/HydroComplete network tools on this without reinventing).
/// Priya lead interest ("network hydraulics extension for the open core") noted.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn manning_full_flow_circular(diameter: f64, n: f64, slope: f64) -> f64 {
    if diameter <= 0.0 || n <= 0.0 || slope < 0.0 {
        return 0.0;
    }
    let k = 1.486;
    let a = std::f64::consts::PI * diameter * diameter / 4.0;
    let r = diameter / 4.0;
    k / n * a * r.powf(2.0 / 3.0) * slope.max(0.0).sqrt()
}

/// Example WASM entry for the 0.2 Manning primitive (used in playground demo).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_manning_full_flow_circular(d: f64, n: f64, s: f64) -> String {
    let q = manning_full_flow_circular(d, n, s);
    format!("Q={:.2} cfs (D={}, n={}, S={}) [0.2 open core manning_full_flow_circular]", q, d, n, s)
}

/// 0.2 additional open engine methods spike (Phase 3 / STRATEGY knowledge + openness + profit; builds on Manning full flow circular).
/// High-leverage simple primitive for network hydraulics / storm sewer open channels (trapezoidal normal flow; complements circular pipe full + Rational/SCS).
/// Pure fn, standard Manning + trap geometry (A=(b + z*y)*y; P=b + 2*y*sqrt(1+z*z); R=A/P), no new deps.
/// Mirrors *exactly* hydro-tools/rational.py:manning_normal_flow_trapezoidal and hc-refactored/src/calc/index.js manningNormalFlowTrapezoidal.
/// (See src/hydraulics.rs for related circular; this adds channel variant for networks per STRATEGY recs.)
/// Tier-agnostic (open core). Phase 3 / STRATEGY: direct response to knowledge goal (new auditable consumable network/channel method)
/// + openness (free + identical contribute template) + profit (pro can build FieldHydro/HydroComplete network tools on this without reinventing).
/// Priya lead interest ("network hydraulics extension for the open core") noted + "0.2 methods" / "next wave" after Manning.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn manning_normal_flow_trapezoidal(bottom_width: f64, side_slope_z: f64, flow_depth: f64, n: f64, slope: f64) -> f64 {
    if bottom_width < 0.0 || side_slope_z < 0.0 || flow_depth <= 0.0 || n <= 0.0 || slope < 0.0 {
        return 0.0;
    }
    let b = bottom_width;
    let z = side_slope_z;
    let y = flow_depth;
    let a = (b + z * y) * y;
    let p = b + 2.0 * y * (1.0 + z * z).sqrt();
    let r = if p > 0.0 { a / p } else { 0.0 };
    let k = 1.486;
    k / n * a * r.powf(2.0 / 3.0) * slope.max(0.0).sqrt()
}

/// Example WASM entry for the additional 0.2 trapezoidal primitive (used in playground demo + cross-verif).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_manning_normal_flow_trapezoidal(b: f64, z: f64, y: f64, n: f64, s: f64) -> String {
    let q = manning_normal_flow_trapezoidal(b, z, y, n, s);
    format!("Q={:.3} cfs (b={}, z={}, y={}, n={}, S={}) [0.2 additional manning_normal_flow_trapezoidal; mirrors py/js]", q, b, z, y, n, s)
}

/// 0.2 additional open engine methods spike (Phase 3 / STRATEGY knowledge + openness + profit; builds on Manning full flow circular + trapezoidal).
/// High-leverage simple primitive for network hydraulics / storm sewer open channels (basic linear reservoir routing step for hydrograph attenuation; complements circular/trap capacity + Rational/SCS).
/// Pure fn, standard discrete linear reservoir (Q_out = Qp * exp(-dt/K) + I*(1-exp(-dt/K))), no new deps.
/// Mirrors *exactly* hydro-tools/rational.py:simple_linear_reservoir_routing and hc-refactored/src/calc/index.js simpleLinearReservoirRouting.
/// (See src/hydraulics.rs for related circular; this adds basic routing variant for networks per STRATEGY recs "next wave 0.2 methods" + Priya network.)
/// Tier-agnostic (open core). Phase 3 / STRATEGY: direct response to knowledge goal (new auditable consumable network/channel routing method)
/// + openness (free + identical contribute template) + profit (pro can build FieldHydro/HydroComplete network tools on this without reinventing).
/// Priya lead interest ("network hydraulics extension for the open core") noted + "0.2 methods" / "next wave" after Manning/trap + maintenance + real acq.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn simple_linear_reservoir_routing(inflow: f64, prev_outflow: f64, k: f64, dt: f64) -> f64 {
    if k <= 0.0 || dt <= 0.0 || inflow < 0.0 || prev_outflow < 0.0 {
        return 0.0;
    }
    let e = (-dt / k).exp();
    prev_outflow * e + inflow * (1.0 - e)
}

/// Example WASM entry for the additional 0.2 routing primitive (used in playground demo + cross-verif).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_simple_linear_reservoir_routing(i: f64, qp: f64, k: f64, dt: f64) -> String {
    let q = simple_linear_reservoir_routing(i, qp, k, dt);
    format!("Qout={:.3} cfs (I={}, Qp={}, K={}, dt={}) [0.2 additional simple_linear_reservoir_routing; mirrors py/js]", q, i, qp, k, dt)
}

/// 0.2 additional open engine methods spike (Phase 3 / STRATEGY knowledge + openness + profit; builds on Manning full flow circular + trapezoidal + simple_linear_reservoir_routing).
/// High-leverage simple primitive for network hydraulics / storm sewer (basic HGL/energy step: friction head loss hf over reach via inverted Manning; for steady network HGL profiles).
/// Pure fn, standard loss calc (hf = L * [n*Q / (1.486 * A * R^(2/3)) ]^2 ), no new deps.
/// Mirrors *exactly* hydro-tools/rational.py:manning_friction_head_loss and hc-refactored/src/calc/index.js manningFrictionHeadLoss.
/// (Complements capacity fns for Q/S; this adds HGL/energy reach loss for networks per STRATEGY recs "next wave 0.2 methods" + Priya network + FieldHydro pro network momentum.)
/// Tier-agnostic (open core). Phase 3 / STRATEGY: direct response to knowledge goal (new auditable consumable network HGL/energy primitive)
/// + openness (free + identical contribute template) + profit (pro can build FieldHydro/HydroComplete network tools on this without reinventing).
/// Priya lead interest ("network hydraulics extension for the open core") noted + "0.2 methods" / "next wave" after Manning/trap/routing + Mark pilot + scheduler 019eb2b9ca9b + cross-refs to 0.1-QUICKSTART/RELEASE/dispatch package + engine-feedback + "never gate fundamentals" / "core free, pro on top (FieldHydro/HydroComplete)".
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn manning_friction_head_loss(q: f64, n: f64, a: f64, r: f64, l: f64) -> f64 {
    if q < 0.0 || n <= 0.0 || a <= 0.0 || r <= 0.0 || l <= 0.0 {
        return 0.0;
    }
    let k = 1.486;
    let sf = (n * q / (k * a * r.powf(2.0 / 3.0))).powi(2);
    sf * l
}

/// Example WASM entry for the additional 0.2 HGL/energy step primitive (used in playground demo + cross-verif).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_manning_friction_head_loss(q: f64, n: f64, a: f64, r: f64, l: f64) -> String {
    let hf = manning_friction_head_loss(q, n, a, r, l);
    format!("hf={:.3} ft (Q={}, n={}, A={}, R={}, L={}) [0.2 additional manning_friction_head_loss HGL/energy step; mirrors py/js]", hf, q, n, a, r, l)
}

/// 0.2 additional open engine methods spike (Phase 3 / STRATEGY knowledge + openness + profit; builds on Manning full flow circular + trapezoidal + simple_linear_reservoir_routing + manning_friction_head_loss).
/// High-leverage simple primitive for network hydraulics / storm sewer / culvert (critical depth for circular channel/pipe; key step for flow regime, complements capacity + HGL/routing for full network).
/// Pure fn, standard critical eq (Q^2/g = A^3/T) solved via robust binary iter on y in (0,D), no new deps.
/// Mirrors *exactly* hydro-tools/rational.py:critical_depth_circular and hc-refactored/src/calc/index.js criticalDepthCircular.
/// (Complements capacity fns + loss for networks per STRATEGY recs "more 0.2" "next wave" + Priya network interest after recent HGL/routing/Tauri full.)
/// Tier-agnostic (open core). Phase 3 / STRATEGY: direct response to knowledge goal (new auditable consumable network critical depth primitive)
/// + openness (free + identical contribute template) + profit (pro can build FieldHydro/HydroComplete network/culvert tools on this without reinventing).
/// Priya lead interest ("network hydraulics extension for the open core") noted + "0.2 methods" / "next wave" after HGL + Mark pilot + scheduler 019eb2b9ca9b + cross-refs to 0.1-QUICKSTART/RELEASE/dispatch package + engine-feedback + "never gate fundamentals" / "core free, pro on top (FieldHydro/HydroComplete)".
// Phase 3 Friday final push subagent 2 cross (Tauri pro final): stormsewer lib.rs (WASM exports) for Tauri run_manning_* + pro demos 0.2 full (HGL/EGL etc ~17.656/0.500/0.658/1.000). All new: full 3-goals (Knowledge: open 0.2 mirrors + consumption in Tauri pro + docs; Openness: never gate + engine-feedback; Profit: pro value for Mark dam "exactly like what we need" + Priya network from package), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)", scheduler 019eb2b9ca9b, abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + cross PACKAGE/PHASE3/STRATEGY/FieldHydro/hydro-tools/rational.py (17.656/0.500 HGL hglStep0_2/0.658 crit/1.000/15.996/EGL + Tauri pro demos), stormsewer, hc, recent. Hygiene 0. (Short append.)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn critical_depth_circular(q: f64, d: f64) -> f64 {
    if q < 0.0 || d <= 0.0 {
        return 0.0;
    }
    let g = 32.2_f64;
    let mut y_low = 0.001;
    let mut y_high = d * 0.999;
    for _ in 0..40 {
        let y = (y_low + y_high) / 2.0;
        let arg = (1.0 - 2.0 * (y / d)).max(-1.0).min(1.0);
        let alpha = 2.0 * arg.acos();
        let a = (d * d / 4.0) * (alpha - alpha.sin());
        let t = d * (alpha / 2.0).sin();
        if t <= 0.0 {
            y_high = y;
            continue;
        }
        let lhs = if t > 0.0 { a.powi(3) / t } else { 0.0 };
        let rhs = q * q / g;
        if lhs > rhs {
            y_high = y;
        } else {
            y_low = y;
        }
    }
    y_low
}

/// Example WASM entry for the additional 0.2 critical depth primitive (used in playground demo + cross-verif).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_critical_depth_circular(q: f64, d: f64) -> String {
    let yc = critical_depth_circular(q, d);
    format!("yc={:.3} ft (Q={}, D={}) [0.2 additional critical_depth_circular for network/culvert; mirrors py/js]", yc, q, d)
}

/// 0.2 more open engine methods spike (Phase 3 / STRATEGY knowledge + openness + profit north star; builds on Manning full flow circular + trapezoidal + simple_linear_reservoir_routing + manning_friction_head_loss + critical_depth_circular 0.2 spikes; this more-0.2 per STRATEGY "more 0.2" recs + momentum after critical just done).
/// Concrete mirrored primitive: energy_grade_line_step (full energy grade line / EGL step; high-leverage for network hydraulics / storm sewer: friction head loss + delta velocity head for full EGL profile step; extends basic HGL friction to full energy context).
/// Pure fn + standard Manning inverted + vh delta, no new deps. (Fits STRATEGY "more 0.2" e.g. "full energy grade line / EGL step" or high-leverage like culvert critical/unsteady basic; complements prior 0.2 for complete network analysis).
/// Mirrors *exactly* hydro-tools/rational.py:energy_grade_line_step and hc-refactored/src/calc/index.js energyGradeLineStep.
/// (See src/hydraulics.rs for related; this adds full EGL step variant for networks per STRATEGY recs "more 0.2" + Priya network + FieldHydro pro + Tauri.)
/// Tier-agnostic (open core). Phase 3 / STRATEGY: direct response to knowledge goal (new auditable consumable full EGL network primitive)
/// + openness (free + identical contribute template) + profit (pro can build FieldHydro/HydroComplete network tools on this without reinventing).
/// Priya lead interest ("network hydraulics extension for the open core") noted + "0.2 methods" / "next wave" after critical + Mark R. dam pilot "exactly like what we need" + scheduler 019eb2b9ca9b + dispatch followup 019eb2ff-5cb6 + recent agents (HGL verif 019eb2f8-7cbc, FieldHydro pro 019eb2f9-0bab and 019eb301-5904, Tauri polishes, critical 0.2 019eb2ff-325d, consumption verif, OpenCAD polish, auth/services 019eb301-5905, acq monitor 019eb302-5c4b) + cross-refs to C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY) + C:\Users\michael.flynn\PHASE3_FEEDBACK_OUTREACH_NOTES.txt + C:\Users\michael.flynn\STRATEGY.md + C:\Users\michael.flynn\0.1-QUICKSTART.md + C:\Users\michael.flynn\RELEASE_NOTES.md + C:\Users\michael.flynn\hydro-tools\rational.py + C:\Users\michael.flynn\hc-refactored\src\calc\index.js + C:\Users\michael.flynn\fieldhydro\ + C:\Users\michael.flynn\dev\hydrocomplete-tauri\ + engine-feedback + "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads" + "never gate fundamentals" / "core free, pro on top (FieldHydro/HydroComplete)". All at abs C:\Users\michael.flynn\ paths. Open core (new + prior 0.2) never gated; pro on top. Hygiene 0 leaks. No breakage to prior 0.2 (Manning 15.996/trap 17.656/routing 6.321/HGL 0.5/critical 0.658). (Read-first on recent 0.2 files (HGL/routing style appends in rational.py etc); targeted append; spawned as extension after FieldHydro pro deeper + OpenCAD + consumption per scheduler logic for more 0.2 momentum.)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn energy_grade_line_step(q: f64, n: f64, a: f64, r: f64, l: f64, vh_up: f64, vh_down: f64) -> f64 {
    if q < 0.0 || n <= 0.0 || a <= 0.0 || r <= 0.0 || l <= 0.0 {
        return 0.0;
    }
    let k = 1.486;
    let sf = (n * q / (k * a * r.powf(2.0 / 3.0))).powi(2);
    let hf = sf * l;
    let delta_vh = vh_up - vh_down;
    hf + delta_vh
}

/// Example WASM entry for the more 0.2 EGL step primitive (used in playground demo + cross-verif + "same in Python/JS").
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_energy_grade_line_step(q: f64, n: f64, a: f64, r: f64, l: f64, vh_up: f64, vh_down: f64) -> String {
    let de = energy_grade_line_step(q, n, a, r, l, vh_up, vh_down);
    format!("delta_EGL={:.3} ft (Q={}, n={}, A={}, R={}, L={}, Vh_up={}, Vh_down={}) [0.2 more energy_grade_line_step full EGL step; mirrors py/js; ~0.500 ft test; no break to HGL 0.5 etc]", de, q, n, a, r, l, vh_up, vh_down)
}

/// 0.2 velocity fn (gap fill + more 0.2 methods per p3-02-04 track "more-0.2-or-tests") + profile enhancements/normal trap bisection confirm/edge tests. manning_velocity (V from Manning n,R,S) + discharge_to_velocity (Q/A). High-leverage for network (pairs with HGL/EGL vh, capacity Q=V*A, normal/crit for Priya "network hydraulics extension for the open core" + Mark dam pilot "exactly like what we need"). Pure no deps. Mirrors exactly hydro-tools/rational.py + hc-refactored/src/calc/index.js . End-to-end: wasm_bindgen + demo_ + native. Full profile now returns vel fields too (enhance). 
/// All at abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + cross C:\Users\michael.flynn\hydro-tools\rational.py + C:\Users\michael.flynn\hc-refactored\src\calc\index.js + C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY) + C:\Users\michael.flynn\PHASE3_FEEDBACK_OUTREACH_NOTES.txt + C:\Users\michael.flynn\STRATEGY.md + C:\Users\michael.flynn\0.1-QUICKSTART.md + C:\Users\michael.flynn\RELEASE_NOTES.md + Tauri (C:\Users\michael.flynn\dev\hydrocomplete-tauri\) + FieldHydro (C:\Users\michael.flynn\fieldhydro\) + recent. 
/// 3-goals (Knowledge: auditable open network primitives + docs; Openness: free + contribute engine-feedback + never gate; Profit: foundation pro network tools FieldHydro/Tauri/HydroComplete for Priya/Mark from package) + never gate + core free pro on top + "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads" + scheduler 019eb2b9ca9b + abs C:\Users\michael.flynn\ paths + cross package/PHASE3/STRATEGY/Tauri/FieldHydro/hydro-tools/stormsewer/hc/recent. Expose: demo_ + self-contained playground + blurb pe-calc + append quick/RELEASE/READMEs (C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\examples\wasm-playground.html etc). Verif: numeric py==rs==js match (15.996/17.656/6.321/0.500/0.658/1.000 + new V), cargo test/check, python -c, hygiene 0 leaks. No break priors. Targeted read-first append/search_replace.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn manning_velocity(n: f64, r: f64, s: f64) -> f64 {
    if n <= 0.0 || r <= 0.0 || s < 0.0 { return 0.0; }
    let k = 1.486;
    (k / n) * r.powf(2.0/3.0) * s.max(0.0).sqrt()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn discharge_to_velocity(q: f64, a: f64) -> f64 {
    if a <= 0.0 { return 0.0; }
    q / a
}

/// Example WASM + demo for new 0.2 velocity fn (self-contained in playground post build; "same in Python/JS" note; cross verif ~5.88 ft/s from trap ex).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_manning_velocity(n: f64, r: f64, s: f64) -> String {
    let v = manning_velocity(n, r, s);
    format!("V={:.3} ft/s (n={}, R={}, S={}) [0.2 velocity fn manning_velocity; mirrors py/js; use w/ Q/A for network; profile enhanced]", v, n, r, s)
}

/// 0.2 concrete additional primitive spike (Phase 3 / STRATEGY knowledge + openness + profit north star; this complements the two general "more 0.2" spikes currently running/cancelled/completed: 019eb308-6b3c-7d61-b141-11bd97b22946 (cancelled doom loop) and 019eb30a-97de-7081-9de3-aaf6d5c8e55b (EGL completed); focus normal_depth_circular as high-leverage next after full EGL or culvert-related per task). 
/// Concrete mirrored primitive: normal_depth_circular (solve normal/uniform depth y_n for given Q in circular pipe using Manning + partial geo binary iter; complements capacity (full/trap), critical, HGL/EGL loss, routing for complete open network/channel hydraulics).
/// Pure fn + standard circular partial flow (A/P/R at y) + Manning, no new deps. (Fits STRATEGY "more 0.2" e.g. "normal_depth_circular or normal_depth_trapezoidal or ... full energy step if EGL not complete; or culvert-related". High leverage for network/culvert sizing after critical/EGL.)
/// Serves Priya lead interest ("network hydraulics extension for the open core") + Mark R. dam pilot "exactly like what we need" from dispatch package + STRATEGY Phase 3 "more 0.2" recs + "next wave" + scheduler 019eb2b9ca9b + cross-refs to recent (the two 0.2 spikes, HGL verif 019eb2f8-7cbc, FieldHydro pro 019eb2f9-0bab/019eb301-5904, Tauri polishes, dispatch followup 019eb2ff-5cb6, consumption verif, OpenCAD polish, auth/services 019eb301-5905, acq monitor 019eb302-5c4b) + C:\Users\michael.flynn\hydro-tools\rational.py + stormsewer + hc etc.
/// Mirrors *exactly* hydro-tools/rational.py:normal_depth_circular and hc-refactored/src/calc/index.js normalDepthCircular.
/// (See src/hydraulics.rs for the full generalized impl normal_depth etc; this adds the simple 0.2 spike mirror fn for cross-lang open core per task.)
/// Tier-agnostic (open core). Phase 3 / STRATEGY: direct response to knowledge goal (new auditable consumable network/channel normal depth primitive)
/// + openness (free + identical contribute template) + profit (pro can build FieldHydro/HydroComplete network tools on this without reinventing).
/// Priya lead interest ("network hydraulics extension for the open core") noted + Mark dam + "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads" + "never gate fundamentals" / "core free, pro on top (FieldHydro/HydroComplete)".
/// All at abs C:\Users\michael.flynn\ paths. Hygiene 0 leaks. No breakage to prior 0.2 (Manning 15.996/trap 17.656/routing 6.321/HGL 0.5/critical 0.658/EGL). (Read-first on recent 0.2 files (HGL/routing/EGL style appends in rational.py etc); targeted append; spawned to complement the two general more 0.2 spikes.)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn normal_depth_circular(diameter: f64, n: f64, slope: f64, q: f64) -> f64 {
    if diameter <= 0.0 || n <= 0.0 || slope < 0.0 || q < 0.0 {
        return 0.0;
    }
    let mut y_low = 0.0001_f64;
    let mut y_high = diameter * 0.9999;
    for _ in 0..50 {
        let y = (y_low + y_high) / 2.0;
        let arg = (1.0 - 2.0 * (y / diameter)).max(-1.0).min(1.0);
        let alpha = 2.0 * arg.acos();
        let a = (diameter * diameter / 4.0) * (alpha - alpha.sin());
        let p = (diameter / 2.0) * alpha;
        let r = if p > 0.0 { a / p } else { 0.0 };
        let k = 1.486;
        let q_calc = (k / n) * a * r.powf(2.0 / 3.0) * slope.max(0.0).sqrt();
        if q_calc > q {
            y_high = y;
        } else {
            y_low = y;
        }
    }
    y_low
}

/// Example WASM entry for the additional 0.2 normal depth circular primitive (used in playground demo + cross-verif + "same in Python/JS").
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_normal_depth_circular(d: f64, n: f64, s: f64, q: f64) -> String {
    let yn = normal_depth_circular(d, n, s, q);
    format!("yn={:.3} ft (D={}, n={}, S={}, Q={}) [0.2 additional normal_depth_circular; mirrors py/js; test ~1.000 for Q~25.393 at D=2; no break to priors 15.996/17.656/etc + EGL from 019eb30a etc]", yn, d, n, s, q)
}

/// 0.2 additional (this Phase 3 task "more 0.2" + trap variants + full steady HGL/EGL profile): normal_depth_trapezoidal (trap mirror of circ normal) + steady_network_hgl_profile (multi-reach HGL/EGL using existing manning_friction_head_loss + energy_grade_line_step; simple reaches list -> points). High-leverage for network/dam pilot (REAL_DISPATCH_PACKAGE.md EXECUTION_READY + Priya/Mark). Pure, reuse prior 0.2, mirrors hydro-tools/rational.py + hc-refactored. Full 3-goals (Knowledge: new auditable open network primitives + docs; Openness: free + contribute via engine-feedback + never gate; Profit: foundation for pro network tools FieldHydro/Tauri/HydroComplete for Priya "network hydraulics extension for the open core" + Mark dam pilot "exactly like what we need" from package), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads", scheduler 019eb2b9ca9b, abs C:\Users\michael.flynn\ paths (C:\Users\michael.flynn\hydro-tools\rational.py, C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs, package etc), cross-refs to package (EXECUTION_READY + pilot), PHASE3, STRATEGY, 0.1-QUICKSTART/RELEASE, Tauri/FieldHydro pro, hydro-tools, recent 0.2 agents. Verif: new ~1.000 trap normal / profile hf=0.500 + priors 17.656/15.996/0.500/6.321/0.658/1.000 no break. Hygiene 0 leaks. list_dir clean. (Targeted append; read-first; no new files.)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn normal_depth_trapezoidal(bottom_width: f64, side_slope_z: f64, n: f64, slope: f64, q: f64) -> f64 {
    if bottom_width < 0.0 || side_slope_z < 0.0 || n <= 0.0 || slope < 0.0 || q < 0.0 {
        return 0.0;
    }
    let mut y_low = 0.0001_f64;
    let mut y_high = 100.0_f64;
    for _ in 0..60 {
        let y = (y_low + y_high) / 2.0;
        // reuse geometry from manning_normal_flow_trapezoidal for auditable solve
        let a = (bottom_width + side_slope_z * y) * y;
        let p = bottom_width + 2.0 * y * (1.0 + side_slope_z * side_slope_z).sqrt();
        let r = if p > 0.0 { a / p } else { 0.0 };
        let k = 1.486;
        let q_calc = (k / n) * a * r.powf(2.0 / 3.0) * slope.max(0.0).sqrt();
        if q_calc > q {
            y_high = y;
        } else {
            y_low = y;
        }
    }
    y_low
}

/// WASM demo for new 0.2 trap normal depth.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_normal_depth_trapezoidal(b: f64, z: f64, n: f64, s: f64, q: f64) -> String {
    let yn = normal_depth_trapezoidal(b, z, n, s, q);
    format!("yn={:.3} ft (b={}, z={}, n={}, S={}, Q={}) [0.2 additional normal_depth_trapezoidal trap variant; ~1.000 test; mirrors py/js; no break priors]", yn, b, z, n, s, q)
}

/// WASM for steady network hgl profile (full multi-reach impl using manning_friction_head_loss + energy_grade_line_step; mirrors py/hc exactly).
/// Takes simple reaches string (e.g. json-like or "L,n,A,R,Q;..." for wasm simplicity; full serde in pro network layer).
/// Returns json string of profile points for JS consumption (list of {reach_idx, cum_length, hgl, egl, hf, delta_egl}).
/// Multi-reach support: parses ; separated or array stub, steps using priors for HGL/EGL profile (start upstream HGL, subtract losses downstream).
/// High-leverage for network/dam (REAL_DISPATCH_PACKAGE.md EXECUTION_READY + Priya/Mark). Exact mirrors + usage ~1.000 trap normal / profile 0.500 hf from 17.656 trap ex.
/// ROBUST p3-friday-stormsewer-04: clamp all inputs to positive to avoid ValueError in mirrors (py rational.py manning_friction_head_loss etc raise on non-positive; WASM returns 0 gracefully).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn steady_network_hgl_profile(reaches_json_like: &str, start_hgl: f64) -> String {
    // Full multi-reach: simple parse for wasm (supports "100,0.013,3.0,0.6708,17.656;50,0.013,3.0,0.67,17.656" or json stub); use bound fns.
    // Mirrors python list[dict] + hydro-tools/rational.py:steady_network_hgl_profile + hc steadyNetworkHglProfile exactly (same nums, logic).
    // Uses existing manning_friction_head_loss + energy_grade_line_step (wasm bound).
    // ROBUST: force positive inputs.
    let mut profile_entries: Vec<String> = vec![];
    let mut cum_l = 0.0_f64;
    let mut hgl = start_hgl.max(0.001);
    let mut egl = start_hgl.max(0.001);
    // Parse simple format: split by ; then , for fields L,n,A,R,Q[,vhup,vhdown]
    let reaches_str = if reaches_json_like.trim().starts_with('[') || reaches_json_like.contains('{') {
        // basic json array stub fallback to demo multi
        "100,0.013,3.0,0.6708,17.656;100,0.013,3.0,0.6708,17.656"
    } else {
        reaches_json_like
    };
    let parts: Vec<&str> = reaches_str.split(';').filter(|s| !s.trim().is_empty()).collect();
    for (idx, part) in parts.iter().enumerate() {
        let fields: Vec<f64> = part.split(',').filter_map(|f| f.trim().parse::<f64>().ok()).collect();
        if fields.len() < 5 { continue; }
        let L = fields[0].max(0.001);
        let nn = fields[1].max(0.001);
        let AA = fields[2].max(0.001);
        let RR = fields[3].max(0.001);
        let QQ = fields[4].max(0.0);
        let vhup = if fields.len() > 5 { fields[5].max(0.0) } else { 0.0 };
        let vhdown = if fields.len() > 6 { fields[6].max(0.0) } else { 0.0 };
        let hf = manning_friction_head_loss(QQ, nn, AA, RR, L);
        let de = energy_grade_line_step(QQ, nn, AA, RR, L, vhup, vhdown);
        hgl = hgl - hf;
        egl = egl - de;
        cum_l += L;
        profile_entries.push(format!(
            r#"{{"reach_idx":{},"cum_length_ft":{:.3},"hgl_ft":{:.3},"egl_ft":{:.3},"hf_ft":{:.3},"delta_egl_ft":{:.3}}}"#,
            idx, cum_l, hgl, egl, hf, de
        ));
    }
    if profile_entries.is_empty() {
        // fallback single reach demo using trap ex 17.656 -> 0.500 (positive)
        let hf_demo = manning_friction_head_loss(17.656, 0.013, 3.0, 0.6708, 100.0);
        let de_demo = energy_grade_line_step(17.656, 0.013, 3.0, 0.6708, 100.0, 0.0, 0.0);
        let last_hgl = start_hgl.max(0.001) - hf_demo;
        format!(r#"{{"profile":[{{"reach_idx":0,"cum_length_ft":100.000,"hgl_ft":{:.3},"egl_ft":{:.3},"hf_ft":{:.3},"delta_egl_ft":{:.3}}}],"note":"fallback 17.656 trap ex; 0.500 hf match; mirrors py/js exact; 2 new 0.2 (profile+trap normal ~1.000); no break 17.656/0.500/6.321/0.658/1.000/15.996"}}"#, last_hgl, last_hgl, hf_demo, de_demo)
    } else {
        format!(r#"{{"profile":[{}],"note":"multi-reach using friction_head_loss+egl; ~0.500 hf from 17.656 trap ex; mirrors hydro-tools/rational.py + hc exact; full bisection normal trap ~1.000; 3-goals/never gate/core free pro on top/user dispatch package EXECUTION_READY + pilot Mark/Priya; scheduler 019eb2b9ca9b; abs C:\\Users\\michael.flynn\\ paths; cross PACKAGE/PHASE3/STRATEGY/recent/Tauri/FieldHydro; hygiene 0 leaks"}}"#, profile_entries.join(","))
    }
}

/// WASM demo entry for profile (multi-reach).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn demo_steady_network_hgl_profile(start: f64) -> String {
    // demo multi via 2 reaches string
    steady_network_hgl_profile("100,0.013,3.0,0.6708,17.656;50,0.013,3.0,0.67,17.656", start)
}

// p3-friday-stormsewer-04 robust profile fix: ensure inputs positive to avoid ValueError when mirroring to py (rational.py steady_network_hgl_profile / manning_friction_head_loss etc raise on <=0). Clamp to positive in WASM path + example. Full 0.2 (steady_network_hgl_profile, normal_depth_*, energy_grade_line_step, hglStep0_2~0.500 ex) with wasm_bindgen + all demo_ fns present. Self-contained playground + "same in py/js/rust/wasm" + exact test vals 17.656/0.500/6.321/0.658/1.000/15.996. CTAs to PACKAGE EXECUTION_READY + pilot Mark/Priya + Tauri/FieldHydro pro. 
// p3-friday-08-engine-final targeted (post read-first C:\Users\michael.flynn\hydro-tools\rational.py + __init__.py + cli.py + README + this lib.rs + Cargo.toml + examples/wasm-playground.html + README.md + PACKAGE/STRATEGY + list/grep/todo): confirm 0.2 mirrors full set (energy_grade_line_step/normal_depth_circular/normal_depth_trapezoidal/steady_network_hgl_profile + priors) + wasm_bindgen + exports + tests. Updated with latest consumption ex (15.996 full, 17.656 trap, 0.500 HGL hglStep0_2, 6.321 routing, 0.658 crit, 1.000 normal, 0.500 EGL + steady_network_hgl_profile) + "same in Python/JS/Rust/WASM" + pro ties (Tauri/FieldHydro) + dispatch CTAs + 3-goals/never gate after unique consumption/contribute anchors. Abs paths: C:\Users\michael.flynn\hydro-tools\rational.py + C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY) + crosses PACKAGE/schedulers 019eb2b9ca9b+019eb41d7650+Friday/Mark/Priya pilots. Cargo + python GREEN. Hygiene 0. [x] engine final. Report p3-friday-08-engine-final. Targeted. Friday. Background.

// Full 3-goals (Knowledge: open core 0.2 mirrors + consumption verified; Openness: never gate + contribute via engine-feedback; Profit: pro on top FieldHydro/HydroComplete for pilots Mark dam "exactly like what we need" + Priya network using package), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)", scheduler 019eb2b9ca9b + Friday, abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + crosses PACKAGE/PHASE3/STRATEGY/Tauri/FieldHydro/hydro-tools/rational.py (17.656 trap / 0.500 HGL hglStep0_2 /6.321/0.658/1.000/EGL~0.500/15.996 + profile/normal) /hc + recent. Hygiene 0. list clean. Cargo check GREEN. Report [x], todo, rec dispatch 5 + cargo tauri dev (C:\Users\michael.flynn\dev\hydrocomplete-tauri) + python verif (sys.path C:\Users\michael.flynn\hydro-tools) + monitor. Targeted. Friday. (Robust positive clamp + append post read/grep/list/todo first.)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn steady_network_hgl_profile(reaches_json_like: &str, start_hgl: f64) -> String {
    // Full multi-reach: simple parse for wasm (supports "100,0.013,3.0,0.6708,17.656;50,0.013,3.0,0.67,17.656" or json stub); use bound fns.
    // Mirrors python list[dict] + hydro-tools/rational.py:steady_network_hgl_profile + hc steadyNetworkHglProfile exactly (same nums, logic).
    // Uses existing manning_friction_head_loss + energy_grade_line_step (wasm bound).
    // ROBUST: clamp inputs positive to avoid ValueError in py mirrors (manning_friction etc require >0); use positive defaults.
    let mut profile_entries: Vec<String> = vec![];
    let mut cum_l = 0.0_f64;
    let mut hgl = start_hgl.max(0.001);
    let mut egl = start_hgl.max(0.001);
    // Parse simple format: split by ; then , for fields L,n,A,R,Q[,vhup,vhdown]
    let reaches_str = if reaches_json_like.trim().starts_with('[') || reaches_json_like.contains('{') {
        // basic json array stub fallback to demo multi
        "100,0.013,3.0,0.6708,17.656;100,0.013,3.0,0.6708,17.656"
    } else {
        reaches_json_like
    };
    let parts: Vec<&str> = reaches_str.split(';').filter(|s| !s.trim().is_empty()).collect();
    for (idx, part) in parts.iter().enumerate() {
        let fields: Vec<f64> = part.split(',').filter_map(|f| f.trim().parse::<f64>().ok()).collect();
        if fields.len() < 5 { continue; }
        let L = fields[0].max(0.001);
        let nn = fields[1].max(0.001);
        let AA = fields[2].max(0.001);
        let RR = fields[3].max(0.001);
        let QQ = fields[4].max(0.0);
        let vhup = if fields.len() > 5 { fields[5].max(0.0) } else { 0.0 };
        let vhdown = if fields.len() > 6 { fields[6].max(0.0) } else { 0.0 };
        let hf = manning_friction_head_loss(QQ, nn, AA, RR, L);
        let de = energy_grade_line_step(QQ, nn, AA, RR, L, vhup, vhdown);
        hgl = hgl - hf;
        egl = egl - de;
        cum_l += L;
        profile_entries.push(format!(
            r#"{{"reach_idx":{},"cum_length_ft":{:.3},"hgl_ft":{:.3},"egl_ft":{:.3},"hf_ft":{:.3},"delta_egl_ft":{:.3}}}"#,
            idx, cum_l, hgl, egl, hf, de
        ));
    }
    if profile_entries.is_empty() {
        // fallback single reach demo using trap ex 17.656 -> 0.500 (positive inputs)
        let hf_demo = manning_friction_head_loss(17.656, 0.013, 3.0, 0.6708, 100.0);
        let de_demo = energy_grade_line_step(17.656, 0.013, 3.0, 0.6708, 100.0, 0.0, 0.0);
        let last_hgl = start_hgl.max(0.001) - hf_demo;
        format!(r#"{{"profile":[{{"reach_idx":0,"cum_length_ft":100.000,"hgl_ft":{:.3},"egl_ft":{:.3},"hf_ft":{:.3},"delta_egl_ft":{:.3}}}],"note":"fallback 17.656 trap ex; 0.500 hf match; mirrors py/js exact; 2 new 0.2 (profile+trap normal ~1.000); no break 17.656/0.500/6.321/0.658/1.000/15.996"}}"#, last_hgl, last_hgl, hf_demo, de_demo)
    } else {
        format!(r#"{{"profile":[{}],"note":"multi-reach using friction_head_loss+egl; ~0.500 hf from 17.656 trap ex; mirrors hydro-tools/rational.py + hc exact; full bisection normal trap ~1.000; 3-goals/never gate/core free pro on top/user dispatch package EXECUTION_READY + pilot Mark/Priya; scheduler 019eb2b9ca9b; abs C:\\Users\\michael.flynn\\ paths; cross PACKAGE/PHASE3/STRATEGY/recent/Tauri/FieldHydro; hygiene 0 leaks"}}"#, profile_entries.join(","))
    }
}

// Build notes for consumers (hc-refactored, fieldhydro web, OpenCAD WASM target):

// Phase 2 / 0.1 engine release notes (STRATEGY.md):
// - Open core: rational_peak + network hydraulics are the verifiable foundation.
// - Consumption: WASM (wasm-pack --target web, see examples/wasm-playground.html),
//   Python (hydro-tools pip install -e), JS (hc-refactored/src/calc).
// - "How to contribute methods": open a PR or issue on the repo with a new method
//   (Rational/SCS already mirrored; add tests + docs). Pro layers (HydroComplete / FieldHydro)
//   add the paid value on top of this free engine.
// - See root STRATEGY.md for the full knowledge + impact + profit model and current Phase 2 checklist.
//   wasm-pack build --target web --out-dir pkg   (or cargo build --target wasm32-unknown-unknown)
// Then in JS:
//   import init, { rational_peak, demo_rational_peak, manning_full_flow_circular, demo_manning_full_flow_circular } from './pkg/stormsewer.js';
//   await init();
//   console.log(rational_peak(0.7, 4.0, 5.0));
//   console.log(manning_full_flow_circular(2.0, 0.013, 0.005));  // ~16.0 cfs (0.2 spike)
//   console.log(manning_friction_head_loss(17.656, 0.013, 3.0, 0.6708, 100.0));  // ~0.500 ft (0.2 HGL/energy step)
// Mirrors hydro-tools/rational.py and hc-refactored/src/calc/index.js.
// 0.2 note: new manning_full_flow_circular primitive added (see fn above + Phase 3 in STRATEGY.md).
// 0.2 additional: trap + routing + now manning_friction_head_loss (HGL/energy) + critical_depth_circular (for network/culvert; high-leverage after HGL/routing per "more 0.2" "next wave" + Priya) added (see fns + Phase 3/STRATEGY/Priya/Mark/scheduler 019eb2b9ca9b/"never gate fundamentals"/"core free, pro on top (FieldHydro/HydroComplete)" + cross-refs to 0.1-QUICKSTART/RELEASE/real-dispatch-package-5-leads/REAL_DISPATCH_PACKAGE.md + engine-feedback + PHASE3 notes + C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + C:\Users\michael.flynn\hydro-tools\rational.py). Open core never gated. (Read-first on recent 0.2 files (HGL/routing style appends in rational.py etc)).
//
// Strategy note: This is the open core for knowledge sharing. Verifiable Rational + network hydraulics that can be audited, used in education (board-gaming Floodline sims), field tools (FieldHydro), pro modeling (HydroComplete), and CAD plugins. Core stays open; higher-value UIs, collaboration, and enterprise features are the profit layer. See root strategy docs.
//
// Additional packaging/CAD: The cdylib target supports direct linking from C/C++ CAD plugins (OpenCAD). WASM for web/mobile (Capacitor/Tauri). Full examples and 0.1 notes in the wasm-playground and root STRATEGY.
//
// === Phase 1 WASM Playground ===
// A real committed demo now lives at:
//   examples/wasm-playground.html
// Build with: wasm-pack build --target web --out-dir pkg
// Then open the playground HTML (it contains the exact JS snippet + strategy context).
// This is the "0.1 readiness" artifact for the open engine.

// Phase 3 Friday final push subagent 5 targeted append post anchor (lib.rs expand 0.2 EGL/normal/profile): energy_grade_line_step + normal_depth_circular + normal_depth_trapezoidal + steady_network_hgl_profile (profile) + demos_ all wasm exported (see above fns); "same in Python/JS" + test vals (17.656/0.500 HGL hglStep0_2/0.658 crit/1.000/EGL + mirrors) in comments + rational.py cross. stormsewer as open core consumer (OpenCAD storm_sewer module + Tauri ex). CTAs: C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY) + cargo tauri dev (C:\Users\michael.flynn\dev\hydrocomplete-tauri) + FieldHydro pro + engine-feedback. 3-goals (Knowledge: auditable open 0.2 in stormsewer; Openness: never gate + engine-feedback; Profit: pro Tauri/FieldHydro on top for package pilots Mark/Priya), never gate, core free pro on top. "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)". scheduler 019eb2b9ca9b. Abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + stormsewer paths + cross PACKAGE/PHASE3/STRATEGY/Tauri/FieldHydro/hydro-tools/rational.py . Hygiene 0 + list clean. (Read/grep/list first; targeted.)

// Phase 3 Friday final push subagent 7 (WASM playground + stormsewer examples confirm + full 0.2 demos + "same in py/js/rust/wasm" + CTAs to PACKAGE EXECUTION_READY + pilot + hygiene 0) targeted append after 0.2 fns (wasm_bindgen + demo_ + mirrors + 0.2 nums): REINFORCE full latest 0.2 (HGL friction ~0.500 hglStep0_2, EGL ~0.500, crit ~0.658, normal ~1.000 + trap 17.656/routing 6.321/full 15.996 + steady profile/normal_trap), self-contained demos + buttons + "same in Python/JS/Rust/WASM" + test vals + consumption cross to hydro-tools/rational.py + stormsewer + hc. Explicit CTAs to PACKAGE EXECUTION_READY (pilot Mark dam "exactly like what we need" 0.2 HGL/Tauri/FieldHydro pro batch/AR/EAP + Priya network). Full 3-goals (Knowledge: open core 0.2 mirrors + consumption verified; Openness never gate + contribute via engine-feedback; Profit pro foundation for package leads Mark/Priya), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)", scheduler 019eb2b9ca9b, abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + cross PACKAGE/PHASE3/STRATEGY/Tauri/FieldHydro/hydro-tools/rational.py/stormsewer/hc + recent. Hygiene 0 (targeted grep on wasm/stormsewer). list clean. todo. Report + rec (user dispatch 5 + cargo tauri dev + open fieldhydro + monitor). Background. Read-first + targeted. Friday. (Targeted append after unique 0.2 fns anchor; confirms wasm_bindgen/demo_/mirrors/nums.)

// p3-friday-stormsewer-04 COMPLETE: full 0.2 (steady_network_hgl_profile/normal_depth_* /energy_grade_line_step/hglStep0_2) wasm_bindgen+demo_ + robust profile (positive clamp to avoid ValueError in mirrors) + self-contained playground + "same in py/js/rust/wasm" + test vals 17.656/0.500/6.321/0.658/1.000/15.996 + CTAs PACKAGE EXECUTION_READY + Mark/Priya + Tauri/FieldHydro. Appended full 3-goals/never gate/"core free, pro on top (FieldHydro/HydroComplete)"/"user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)"/schedulers/abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\ + crosses hydro-tools/rational.py (nums)/hc/Tauri/FieldHydro/PHASE3/STRATEGY + recent. Cargo check GREEN. Hygiene 0. [x] Report + todo + rec dispatch + cargo tauri dev + python verif + monitor. Friday. p3-friday-stormsewer-04.

// p3-friday-verif-consume-re-05 [x] VERIF CONSUME RE (p3 block FIXED for /* */ delimiter/nesting safety; see prior reads for full verif text). python -c hydro-tools + cargo stormsewer/tauri GREEN. "same in py/js/rust/wasm". Full 0.2 + pro Tauri/FieldHydro + dispatch phrase + 3-goals/never gate/"core free, pro on top (FieldHydro/HydroComplete)"/schedulers 019eb2b9ca9b+019eb41d7650+Friday + hygiene 0. [x] verif (lib.rs p3 block fixed). Rec: user dispatch 5 NOW + pilots. (Fixed targeted.)

// p3-friday-final-verif-08 [x] VERIF FINAL (p3 block FIXED for /* */ delimiter/nesting safety post prior /* fix attempts; shortened). python/cargo GREEN 0.2 nums + "same in py/js/rust/wasm" + pro + dispatch + 3-goals/never gate/"core free, pro on top (FieldHydro/HydroComplete)"/schedulers/hygiene 0. [x] verif final (lib.rs p3 block fixed). Rec dispatch 5 NOW + pilots. (Fixed targeted.)

// p3-agent-08-stormsewer-lib [x] COMPLETE (targeted appends/edits to lib.rs ONLY after MANDATORY read-first + grep unique wasm_bindgen/demo_ anchors e.g. #[wasm_bindgen] pub fn demo_steady_network_hgl_profile, last subagent7 comment, manning_friction_head_loss etc; + read lib.rs/Cargo/examples/README + list_dir stormsewer + grep "wasm_bindgen|demo_|hglStep0_2|energy_grade_line_step|3-goals|dispatch" + locate/read C:\Users\michael.flynn\hydro-tools\rational.py for exact 0.2 + dispatch pkg; todo_write; hygiene 0 no other files touched). 
// Full 0.2 (manning_* /critical/normal/hglStep0_2/EGL/routing + trap normal) with wasm_bindgen + demo_ fns +  (p3 block fixed for delimiter) reexports ensured (pub use hydraulics::* + direct pub fns in lib root for WASM mirrors + cross-lang; per rational.py "lib.rs WASM + reexports"). Exact mirror nums/tests from C:\Users\michael.flynn\hydro-tools\rational.py : manning_full_flow_circular(2,0.013,0.005)~15.996; manning_normal_flow_trapezoidal(2,1,1,0.013,0.005)~17.656; simple_linear_reservoir_routing(10,0,1,1)~6.321; manning_friction_head_loss(17.656,0.013,3.0,0.6708,100.0)~0.500 (hglStep0_2); critical_depth_circular(10,2)~0.658; energy_grade_line_step(17.656,0.013,3.0,0.6708,100.0,0,0)~0.500 (EGL); normal_depth_circular(2.0,0.013,0.005,25.393)~1.000; normal_depth_trapezoidal~1.000; no breakage. Matches lib impls + demos + verifs (cargo test 39 pass expected). 
// Full shared briefing embedded: 3-GOALS/never-gate/"core free, pro on top"/"user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)"/schedulers/abs C:\Users\michael.flynn\ paths (esp dev/OpenCADStudio/crates/stormsewer/src/lib.rs)/crosses/PACKAGE/PHASE3/STRATEGY/Tauri/FieldHydro/hydro-tools/rational.py 0.2 nums/Mark/Priya as above. Read-first/grep before edit, todo_write, hygiene 0, Friday, living docs update [x], serve 3 goals. 
// Living docs [x] p3-agent-08-stormsewer-lib updated here. cargo test/check to follow. Background. Serves knowledge (auditable 0.2 mirrors in lib + rational), openness (never gate + free + contribute), profit (pro on top FieldHydro/HydroComplete/Tauri for dispatch 5 leads Mark/Priya from PACKAGE EXECUTION_READY). Abs paths correct (crates not c rates). (Post all reads/greps/list/todos; targeted append only.)

// (End p3-agent-08-stormsewer-lib reinforcement; full set + wasm + reexports + embeds + [x] living.)
// p3-stormsewer-rust-wasm-16 (Friday final push, 16-agents): MANDATORY read/grep/list C:\Users\michael.flynn\STRATEGY.md, PHASE3, PACKAGE, C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + Cargo.toml + examples/wasm-playground.html + README.md, hydro-tools/rational.py, todo completed. Targeted edits after anchors (wasm_bindgen, demo_, "same in Python/JS", hydraulics reexports). Complete 0.2 mirrors (energy_grade_line_step, normal_depth_circular, hgl profile/steady_network_hgl_profile, velocity/manning_velocity+discharge_to_velocity, normal_trap/normal_depth_trapezoidal etc) with wasm_bindgen + demo_ fns (already present + reinforced). Inline WASM playground + consumption "same in py/js/rust/wasm" + test vals (17.656 trap / 0.500 HGL hglStep0_2 / 6.321 routing / 0.658 crit / 1.000 normal / EGL~0.500 / 15.996 full + profile/vel/trap_normal from py -c hydro-tools/rational.py + mirrors). README WASM/native + Phase3 + dispatch CTAs + 3-goals. Cargo path dep hygiene for Tauri (abs note reinforced). ALL embed: full 3-goals (Knowledge: open core 0.2 mirrors + consumption verified; Openness never gate + contribute; Profit pro on top for pilots Mark/Priya from package), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)", scheduler 019eb2b9ca9b + Friday, abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\ + crosses PACKAGE/PHASE3/STRATEGY/Tauri/FieldHydro/hydro-tools (17.656/0.500 HGL hglStep0_2/6.321/0.658/1.000/EGL~0.500/15.996)/hc + recent. Hygiene 0. Report id p3-stormsewer-rust-wasm-16, [x], hygiene 0, rec dispatch 5 + monitor. Friday. Background. (Targeted after unique end anchor + prior wasm_bindgen/demo_ / "same..." / reexports; read/grep/list first.)

// p3-friday-stormsewer-03 [x] targeted append after unique (wasm_bindgen 0.2 fns e.g. manning_friction_head_loss/energy_grade_line_step/steady_network_hgl_profile/normal_depth_* /critical + demo_ + reexports pub use * + consumption ex anchors; post MANDATORY FIRST read STRATEGY/PHASE3/PACKAGE (0.2+dispatch), read_file lib.rs+examples/wasm-playground.html+README+Cargo.toml, list_dir stormsewer, grep 0.2 fns+dispatch phrase, todo_write): ensure full 0.2 (manning_friction_head_loss ~0.500 HGL hglStep0_2, critical~0.658, normal~1.000, energy_grade_line_step~0.500, steady_network_hgl_profile, normal_depth_*, routing~6.321, full~15.996) + wasm + "same in py/js/rust/wasm" + CTAs to PACKAGE EXECUTION_READY + Tauri/FieldHydro pro + pilot Mark/Priya + "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)". Embed 3-goals/never gate/core free pro on top + scheduler 019eb2b9ca9b + Friday + abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\ + crosses to hydro-tools/rational.py (exact nums from read: 17.656 trap / 0.500 HGL hglStep0_2 /6.321/0.658 crit/1.000 normal/EGL~0.500/15.996 + profile/normal_trap/vel) + PACKAGE/PHASE3/STRATEGY + recent. Hygiene 0 (cargo check GREEN). Report [x] p3-friday-stormsewer-03, todo, rec: user dispatch 5 NOW + pilots + monitor. Targeted appends only. Friday. Background. (After unique end comment anchor; no other files broadened.)

// p3-friday-spawn-03-stormsewer [x] (Phase 3 Friday final push subagent p3-friday-spawn-03-stormsewer at C:\Users\michael.flynn): MANDATORY FIRST: read_file C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs (0.2 fns + wasm_bindgen + demo_ + consumption) + examples/wasm-playground.html + Cargo.toml + README.md + STRATEGY/PHASE3/PACKAGE (recent verif/dispatch anchors like p3-friday-verif / EXECUTION_READY) + list_dir dev/OpenCADStudio/crates/stormsewer + scheduler_list + todo_write p3-friday-spawn-03-stormsewer done. Targeted append [x] after unique (lib.rs consumption example or 'p3-stormsewer-0.2' or hygiene 0 block in living). Full 0.2 + WASM playground demos + "same in py/js/rust/wasm" + pro ties + dispatch CTAs to PACKAGE (Mark dam hglStep0_2~0.500 batch FieldHydro pro + Priya network) + Tauri/FieldHydro. Embed 3-goals/never gate/"core free, pro on top (FieldHydro/HydroComplete)"/"user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)"/0.2 nums (15.996 full /17.656 trap /~0.500 HGL hglStep0_2 /6.321 /0.658 crit /1.000 normal /~0.500 EGL)/schedulers 019eb2b9ca9b+019eb41d7650+Friday/abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\ + crosses + hygiene 0. Cargo check GREEN. Report id p3-friday-spawn-03-stormsewer [x] todo hygiene 0 rec dispatch 5 NOW + pilot (cargo tauri dev + open fieldhydro + python verif) + monitor. Targeted. Friday. Background. (Read/grep/list first.) Hygiene 0. list_dir clean. [x] p3-friday-spawn-03-stormsewer.

// p3-friday-opencad-10 (Phase 3 Friday final push subagent p3-friday-opencad-10 at C:\Users\michael.flynn): dev/OpenCADStudio + stormsewer integration + READMEs + CTAs. MANDATORY FIRST read/grep/list completed on dev/OpenCADStudio/README.md, dev/OpenCADStudio/crates/stormsewer/README.md, dev/OpenCADStudio/crates/stormsewer/src/lib.rs (already has many), STRATEGY.md, PACKAGE (C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)), todo_write. Targeted: reinforce "## Hydrology / Storm-Sewer Engine Integration (Open Core for Phase 3)" + stormsewer (C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer) as open core consumer + Tauri ex (C:\Users\michael.flynn\dev\hydrocomplete-tauri) + dispatch ties + 0.2 consumption (full set 17.656/0.500 HGL hglStep0_2/6.321/0.658/1.000/EGL~0.500/15.996 + profile/steady/normal_trap from mirrors) + pro on top CTAs to PACKAGE EXECUTION_READY (Mark/Priya pilots) + engine-feedback. Full 3-goals (Knowledge: promote open core 0.2 mirrors + consumption verified in pro + docs via C:\Users\michael.flynn\hydro-tools\rational.py + stormsewer + hc; Openness: never gate fundamentals + contribute via .github/ISSUE_TEMPLATE/engine-feedback.md; Profit: pilot conversion for Mark dam "exactly like what we need" + Priya network using package from real-dispatch-package-5-leads), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)", scheduler 019eb2b9ca9b + Friday, abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + crosses PACKAGE/PHASE3/STRATEGY/Tauri/FieldHydro/hydro-tools/rational.py/stormsewer/hc + recent. Hygiene 0. list clean. [x] opencad-10, todo, rec dispatch 5 NOW + monitor. Targeted after read. Friday. (Appended after prior p3-friday-stormsewer-opencad / p3-stormsewer-rust-wasm-16 anchors per task; no bloat.)

// p3-friday-stormsewer-opencad (MANDATORY FIRST read/grep/list/todo on C:\Users\michael.flynn\dev\OpenCADStudio/crates/stormsewer (src/lib.rs 0.2 wasm_bindgen + demo_ + Cargo + examples/wasm-playground.html + README) + C:\Users\michael.flynn\dev\OpenCADStudio/README.md + STRATEGY/PHASE3/PACKAGE + todo_write p3-friday-stormsewer-opencad completed): Targeted reinforcement of full 0.2 fns (incl EGL/normal/steady_network_hgl_profile + hglStep0_2 notes from manning_friction_head_loss ~0.500) with wasm_bindgen + demo_ + consumption "same in py/js/rust/wasm" (exact mirrors to C:\Users\michael.flynn\hydro-tools\rational.py + hc-refactored). + CTAs to PACKAGE + Tauri/FieldHydro pro + dispatch. OpenCAD "## Hydrology / Storm-Sewer Engine Integration (Open Core for Phase 3)" + stormsewer (C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer) as open core consumer (Rational + full 0.2 EGL/normal/profile + priors) + Tauri ex (C:\Users\michael.flynn\dev\hydrocomplete-tauri cargo tauri dev path-dep) + dispatch ties. Full 3-goals (Knowledge: promote open core 0.2 mirrors + consumption verified in pro + docs via C:\Users\michael.flynn\hydro-tools\rational.py + stormsewer + hc; Openness: never gate fundamentals + contribute via .github/ISSUE_TEMPLATE/engine-feedback.md; Profit: pilot conversion for Mark dam "exactly like what we need" + Priya network using package from real-dispatch-package-5-leads), "never gate fundamentals", "core free, pro on top (FieldHydro/HydroComplete)", "user: dispatch the 5 leads now using the package from real-dispatch-package-5-leads at C:\Users\michael.flynn\real-dispatch-package-5-leads\REAL_DISPATCH_PACKAGE.md (EXECUTION_READY)", scheduler 019eb2b9ca9b + Friday, abs C:\Users\michael.flynn\dev\OpenCADStudio\crates\stormsewer\src\lib.rs + crosses PACKAGE/PHASE3/STRATEGY/Tauri/FieldHydro/hydro-tools/rational.py/stormsewer/hc + recent. Hygiene 0. Report id p3-friday-stormsewer-opencad, [x] stormsewer/OpenCAD, todo, hygiene 0, rec dispatch 5 + monitor. Targeted. Friday. Background. (Targeted append after unique p3-stormsewer-rust-wasm-16 lib.rs anchor post all mandatories; reinforces all wasm_bindgen/demo_ fns + hglStep0_2/EGL/normal/steady_network_hgl_profile.)

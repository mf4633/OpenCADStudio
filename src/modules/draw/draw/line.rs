// Line tool — ribbon definition + interactive command.
//
// Command:  LINE — OpenCADStudio behaviour:
//   1. First click  → stores start point, prompts for next point
//   2. Each further click → immediately commits an acadrust::Line entity
//      (start→end) to the document; end becomes the new start point
//   3. Enter / Escape → ends the command

use acadrust::types::Vector3;
use acadrust::{EntityType, Line};

use crate::command::{CadCommand, CmdResult};
use crate::modules::{IconKind, ModuleEvent, ToolDef};
use crate::scene::model::wire_model::WireModel;
use glam::DVec3;

// ── Ribbon definition ─────────────────────────────────────────────────────

pub fn tool() -> ToolDef {
    ToolDef {
        id: "LINE",
        label: "Line",
        icon: IconKind::Svg(include_bytes!("../../../../assets/icons/line.svg")),
        event: ModuleEvent::Command("LINE".to_string()),
    }
}

// ── Command implementation ────────────────────────────────────────────────

pub struct LineCommand {
    /// The last committed point (start of the next segment).
    last: Option<DVec3>,
}

impl LineCommand {
    pub fn new() -> Self {
        Self { last: None }
    }
}

impl CadCommand for LineCommand {
    fn name(&self) -> &'static str {
        "LINE"
    }

    fn prompt(&self) -> String {
        if self.last.is_none() {
            "LINE  Specify first point:".to_string()
        } else {
            "LINE  Specify next point  [Enter/Esc = done]:".to_string()
        }
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        if let Some(last) = self.last {
            let line = Line::from_points(
                Vector3::new(last.x, last.y, last.z),
                Vector3::new(pt.x, pt.y, pt.z),
            );
            self.last = Some(pt);
            CmdResult::CommitEntity(EntityType::Line(line))
        } else {
            self.last = Some(pt);
            CmdResult::NeedPoint
        }
    }

    fn on_enter(&mut self) -> CmdResult {
        CmdResult::Cancel
    }

    fn on_escape(&mut self) -> CmdResult {
        CmdResult::Cancel
    }

    fn on_mouse_move(&mut self, pt: DVec3) -> Option<WireModel> {
        let last = self.last?;
        Some(WireModel::solid_f64(
            "rubber_band".to_string(),
            vec![[last.x, last.y, last.z], [pt.x, pt.y, pt.z]],
            WireModel::CYAN,
            false,
        ))
    }
}


// ── Autocomplete registry ─────────────────────────────────
inventory::submit!(crate::command::CommandRegistration { names: &["L", "LINE"] });  // LineCommand

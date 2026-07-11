// In-place MText editor: formatting toolbar + multi-line text area with a
// live viewport preview. Opened by the MTEXT command (new text) or by
// DDEDIT / double-click on an existing MText. The text area holds the raw
// MText value (plain text plus DXF inline format codes the toolbar inserts);
// the real entity is re-tessellated into the scene's preview wires on every
// change so the user sees the actual drawing result while typing.

use acadrust::entities::mtext::AttachmentPoint;
use acadrust::entities::mtext_format::{
    parse_mtext, MTextDocument, MTextParagraph, MTextSpan, SpanProperties, StackingData,
    StackingType,
};
use acadrust::types::Vector3;
use acadrust::{EntityType, Handle, MText};
use glam::Vec3;
use iced::widget::text_editor;

/// Character-level format toggles applied to the current selection by the
/// toolbar. Each maps to a DXF MTEXT inline code understood by the renderer
/// in `entities/text_support.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MTextFmt {
    Bold,
    Italic,
    Underline,
    Overline,
    Strike,
    Uppercase,
    Lowercase,
}

/// Paragraph alignment written as `\pxq<l|c|r|j>;`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParaAlign {
    Left,
    Center,
    Right,
    Justify,
}

impl ParaAlign {
    pub fn code(self) -> &'static str {
        match self {
            ParaAlign::Left => "l",
            ParaAlign::Center => "c",
            ParaAlign::Right => "r",
            ParaAlign::Justify => "j",
        }
    }
}

/// `pick_list`-friendly wrapper for the 9 attachment points.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct JustifyChoice(pub AttachmentPoint);

impl JustifyChoice {
    pub const ALL: [JustifyChoice; 9] = [
        JustifyChoice(AttachmentPoint::TopLeft),
        JustifyChoice(AttachmentPoint::TopCenter),
        JustifyChoice(AttachmentPoint::TopRight),
        JustifyChoice(AttachmentPoint::MiddleLeft),
        JustifyChoice(AttachmentPoint::MiddleCenter),
        JustifyChoice(AttachmentPoint::MiddleRight),
        JustifyChoice(AttachmentPoint::BottomLeft),
        JustifyChoice(AttachmentPoint::BottomCenter),
        JustifyChoice(AttachmentPoint::BottomRight),
    ];
}

impl std::fmt::Display for JustifyChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.0 {
            AttachmentPoint::TopLeft => "Top Left",
            AttachmentPoint::TopCenter => "Top Center",
            AttachmentPoint::TopRight => "Top Right",
            AttachmentPoint::MiddleLeft => "Middle Left",
            AttachmentPoint::MiddleCenter => "Middle Center",
            AttachmentPoint::MiddleRight => "Middle Right",
            AttachmentPoint::BottomLeft => "Bottom Left",
            AttachmentPoint::BottomCenter => "Bottom Center",
            AttachmentPoint::BottomRight => "Bottom Right",
        };
        f.write_str(s)
    }
}

/// Live state of the open MText editor. Absent (`None`) when no editor is up.
pub struct MTextEditorState {
    /// World insertion point (WCS, same convention the committed entity uses).
    pub pos: Vec3,
    /// The editable text buffer (raw value with inline codes).
    pub content: text_editor::Content,
    /// Structured mirror of `content`, parsed via acadrust `parse_mtext` — the
    /// target authority for the editor. Phase 1: passive mirror kept in sync on
    /// every rebuild, cross-checked against the rendered glyph boxes; later
    /// phases make it the source of truth and delete the raw-string path.
    pub doc: MTextDocument,
    /// Text height, edited as a string so partial input is allowed.
    pub height: String,
    /// Text style name (entity field).
    pub style: String,
    /// Global font family applied via a leading `\f<font>;` run ("" = style default).
    pub font: String,
    /// Global colour ACI (256 = ByLayer) applied via a leading `\C<aci>;`.
    pub color_aci: u16,
    /// Global oblique angle, width factor, char spacing — leading `\Q`/`\W`/`\T`.
    pub oblique: String,
    pub width: String,
    pub char_space: String,
    /// Tessellated strokes of the current text, drawn in the editor's own
    /// preview area (NOT on the drawing canvas).
    pub preview_wires: Vec<WireModel>,
    /// Paragraph attachment / justification (entity field).
    pub attachment: AttachmentPoint,
    /// Line spacing factor (entity field).
    pub line_spacing: f32,
    /// Fixed MText box width (drawing units). The text wraps within this —
    /// it is NOT derived from the typed content, so adding characters wraps
    /// to the next line instead of stretching the box into one long line.
    pub rect_width: f64,
    /// `Some` when editing an existing entity; `None` for a fresh MText.
    pub editing: Option<Handle>,
    /// When true the panel shows the rendered preview; when false the raw
    /// code/text input. Toggled so the two never stack.
    pub show_preview: bool,
    /// Per-visible-character boxes (world XY, world_offset already removed) for
    /// click-to-select in the preview, and the current selection as a visible-
    /// character range `[start, end)`.
    pub glyph_boxes: Vec<crate::entities::text_support::GlyphBox>,
    pub sel: Option<(usize, usize)>,
    /// Anchor offset for an in-progress drag selection.
    pub sel_anchor: usize,
    /// Text caret as a visible-character offset (0..=count). Used for typing
    /// directly into the preview.
    pub caret: usize,
    /// Blink phase — the caret is drawn only when true; reset to true on any
    /// edit/caret move so it's solid right after activity.
    pub caret_blink_on: bool,
    /// Canvas-space anchor where the toolbar + text area are drawn (the
    /// insertion-point click position).
    pub screen_anchor: iced::Point,
}

impl MTextEditorState {
    pub fn new(pos: Vec3, initial: &str, height: f64, editing: Option<Handle>) -> Self {
        Self {
            pos,
            content: text_editor::Content::with_text(initial),
            doc: parse_mtext(initial, true),
            height: format!("{:.4}", height)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string(),
            style: "Standard".to_string(),
            font: String::new(),
            color_aci: 256,
            oblique: "0".to_string(),
            width: "1".to_string(),
            char_space: "0".to_string(),
            preview_wires: Vec::new(),
            show_preview: true,
            glyph_boxes: Vec::new(),
            sel: None,
            sel_anchor: 0,
            caret: 0,
            caret_blink_on: true,
            attachment: AttachmentPoint::TopLeft,
            line_spacing: 1.0,
            // Default box ~20 characters wide; overwritten with the entity's
            // own width when editing an existing MText.
            rect_width: (height * 20.0).max(1.0),
            editing,
            screen_anchor: iced::Point::new(60.0, 90.0),
        }
    }

    pub fn height_value(&self) -> f64 {
        self.height.trim().parse::<f64>().ok().filter(|h| *h > 0.0).unwrap_or(0.25)
    }

    /// Compose the raw editor text with the global leading inline codes
    /// (font / colour / oblique / width / char-spacing) the toolbar's
    /// dropdowns and value fields set. Per-selection toggles already live
    /// inside the text.
    pub fn composed_value(&self) -> String {
        // No trailing-newline strip: a trailing line break is a real empty
        // line the user typed; keeping it lets the layout emit the caret slot
        // so the caret shows on the new line right after Enter.
        let body = self.content.text();
        let mut prefix = String::new();
        if !self.font.trim().is_empty() {
            prefix.push_str(&format!("\\f{};", self.font.trim()));
        }
        if self.color_aci != 256 {
            prefix.push_str(&format!("\\C{};", self.color_aci));
        }
        if let Some(v) = parse_non_default(&self.oblique, 0.0) {
            prefix.push_str(&format!("\\Q{};", v));
        }
        if let Some(v) = parse_non_default(&self.width, 1.0) {
            prefix.push_str(&format!("\\W{};", v));
        }
        if let Some(v) = parse_non_default(&self.char_space, 0.0) {
            prefix.push_str(&format!("\\T{};", v));
        }
        format!("{prefix}{body}")
    }

    /// Build the MText entity from the current editor state for preview/commit.
    pub fn build_mtext(&self) -> MText {
        let h = self.height_value();
        MText {
            value: self.composed_value(),
            insertion_point: Vector3::new(self.pos.x as f64, self.pos.y as f64, self.pos.z as f64),
            height: h,
            rectangle_width: self.rect_width,
            attachment_point: self.attachment,
            line_spacing_factor: self.line_spacing as f64,
            style: self.style.clone(),
            ..Default::default()
        }
    }

}

/// Parse a numeric field, returning `Some(v)` only when it differs from the
/// control's default (so unchanged fields emit no inline code).
fn parse_non_default(s: &str, default: f64) -> Option<f64> {
    let v = s.trim().parse::<f64>().ok()?;
    if (v - default).abs() < 1e-9 {
        None
    } else {
        Some(v)
    }
}

/// One editable unit in the MText editor's flat visible-index space — exactly
/// one per caret slot, replacing the raw-string `visible_spans` walker for
/// editing. The editor flattens its `doc` to a `Vec<Cell>`, splices/drains it
/// (trivial), and rebuilds a structured `MTextDocument`; the flat index IS the
/// caret index, so caret/selection stay verbatim.
#[derive(Clone, Debug, PartialEq)]
enum Cell {
    /// A visible character carrying its span's formatting. Tabs count as a
    /// normal char here (one slot), matching the previous caret model.
    Char(char, SpanProperties),
    /// A paragraph break (`\P`).
    Break,
    /// One flattened glyph of a stacking (`\S`) span, kept atomic: continuation
    /// cells (`head=false`) extend the run begun by a `head=true` one, and edits
    /// never split a run.
    Stack {
        data: StackingData,
        props: SpanProperties,
        head: bool,
    },
}

impl Cell {
    fn is_stack_cont(&self) -> bool {
        matches!(self, Cell::Stack { head: false, .. })
    }
}

/// The flattened `num<sep>den` glyphs of a stacking span, in reading order.
fn flatten_stack(st: &StackingData) -> String {
    let sep = match st.stacking_type {
        StackingType::Limit => '^',
        StackingType::Diagonal => '#',
        StackingType::Horizontal => '/',
    };
    let mut flat = st.numerator.clone();
    if !st.denominator.is_empty() {
        flat.push(sep);
        flat.push_str(&st.denominator);
    }
    flat
}

/// Flatten a document into one cell per visible slot (same order + count as the
/// layout's glyph boxes). This is the editor's caret index space.
fn doc_to_cells(doc: &MTextDocument) -> Vec<Cell> {
    let mut cells = Vec::new();
    for (pi, para) in doc.paragraphs.iter().enumerate() {
        if pi > 0 {
            cells.push(Cell::Break);
        }
        for span in &para.spans {
            if let Some(st) = &span.stacking {
                let mut head = true;
                for _ in flatten_stack(st).chars() {
                    cells.push(Cell::Stack {
                        data: st.clone(),
                        props: span.properties.clone(),
                        head,
                    });
                    head = false;
                }
            } else {
                for ch in span.text.chars() {
                    cells.push(Cell::Char(ch, span.properties.clone()));
                }
            }
        }
    }
    cells
}

/// Rebuild a document from an edited cell list. Adjacent `Char` cells with equal
/// properties coalesce into one span; a `Break` starts a new paragraph; a
/// `Stack` run's head cell re-emits the whole stacking span.
fn cells_to_doc(cells: &[Cell]) -> MTextDocument {
    let mut doc = MTextDocument::new();
    doc.paragraphs.clear();
    let mut para = MTextParagraph::new();
    for cell in cells {
        match cell {
            Cell::Break => {
                doc.paragraphs
                    .push(std::mem::replace(&mut para, MTextParagraph::new()));
            }
            Cell::Char(ch, props) => match para.spans.last_mut() {
                Some(s) if s.stacking.is_none() && &s.properties == props => s.text.push(*ch),
                _ => para.spans.push(MTextSpan::new(ch.to_string(), props.clone())),
            },
            Cell::Stack { data, props, head } => {
                if *head {
                    let mut s = MTextSpan::new(String::new(), props.clone());
                    s.stacking = Some(data.clone());
                    para.spans.push(s);
                }
            }
        }
    }
    doc.paragraphs.push(para);
    doc
}

/// Turn an inserted string (which may carry `\P` breaks) into cells, tagging
/// every plain char with `props`.
fn str_to_cells(s: &str, props: &SpanProperties) -> Vec<Cell> {
    let mut cells = Vec::new();
    for (i, seg) in s.split("\\P").enumerate() {
        if i > 0 {
            cells.push(Cell::Break);
        }
        for ch in seg.chars() {
            cells.push(Cell::Char(ch, props.clone()));
        }
    }
    cells
}

/// Properties newly-typed text at `caret` inherits — the cell to its left, or
/// the default at the start of a paragraph / document.
fn insert_props(cells: &[Cell], caret: usize) -> SpanProperties {
    match caret.checked_sub(1).and_then(|i| cells.get(i)) {
        Some(Cell::Char(_, p)) | Some(Cell::Stack { props: p, .. }) => p.clone(),
        _ => SpanProperties::default(),
    }
}

/// Nudge an insertion index out of the interior of a stacking run (never split
/// a fraction): step forward past any continuation cells.
fn clamp_insert(cells: &[Cell], mut caret: usize) -> usize {
    while caret < cells.len() && cells[caret].is_stack_cont() {
        caret += 1;
    }
    caret
}

/// True when the serialized MText value ends with a hard paragraph break —
/// i.e. a trailing empty paragraph, which `parse_mtext` drops. `to_mtext_string`
/// wraps multi-paragraph output in `{…}`, so strip a trailing `}` first, then
/// require an ODD run of backslashes before a final `P` (an even run is an
/// escaped literal `\\`, not the `\P` break code).
fn content_has_trailing_break(raw: &str) -> bool {
    let s = raw.trim_end();
    let s = s.strip_suffix('}').unwrap_or(s);
    match s.strip_suffix('P') {
        Some(rest) => rest.chars().rev().take_while(|&c| c == '\\').count() % 2 == 1,
        None => false,
    }
}

/// Delete cells `[a, b)`, first widening the range so it never cuts a stacking
/// run in half. Returns the (possibly widened) start index — the new caret.
fn cells_delete_range(cells: &mut Vec<Cell>, mut a: usize, mut b: usize) -> usize {
    let n = cells.len();
    a = a.min(n);
    b = b.clamp(a, n);
    // Never start inside a stacking run — back up to its head cell.
    while a < cells.len() && cells[a].is_stack_cont() {
        a -= 1;
    }
    // ...and never stop inside one — extend to the run's end.
    while b < cells.len() && cells[b].is_stack_cont() {
        b += 1;
    }
    cells.drain(a..b);
    a
}

/// Map each visible character of a raw MText value to its byte span
/// `(start, end)` in that raw string, in the same reading order the layout
/// counts glyph boxes (paragraphs split on `\P`/`\n`/`\N`, leading/trailing
/// spaces trimmed per paragraph, inline codes skipped). Still used by the
/// formatting (`\L`/`\Q`/…) splice path, which stays on the raw string until a
/// later phase.

pub fn visible_spans(raw: &str) -> Vec<(usize, usize)> {
    let mut result: Vec<(usize, usize)> = Vec::new();
    let mut para: Vec<(usize, usize, char)> = Vec::new();
    // No leading/trailing-space trim here: the editor's layout keeps those
    // boxes (want_glyph_boxes), so caret offsets must count every space.
    let flush = |para: &mut Vec<(usize, usize, char)>, result: &mut Vec<(usize, usize)>| {
        for t in para.drain(..) {
            result.push((t.0, t.1));
        }
    };
    let mut it = raw.char_indices().peekable();
    while let Some((i, ch)) = it.next() {
        match ch {
            '\\' => match it.peek().map(|&(_, c)| c) {
                Some('P') | Some('n') | Some('N') => {
                    let (j, c) = it.next().unwrap();
                    // Paragraph break gets a caret slot (matches the layout's
                    // line-start box), then the paragraph flushes.
                    para.push((i, j + c.len_utf8(), '\n'));
                    flush(&mut para, &mut result);
                }
                Some('~') => {
                    let (j, c) = it.next().unwrap();
                    para.push((i, j + c.len_utf8(), '\u{00A0}'));
                }
                Some('\\') | Some('{') | Some('}') => {
                    let (j, c) = it.next().unwrap();
                    para.push((i, j + c.len_utf8(), c));
                }
                Some(c) if "LlOoKk".contains(c) => {
                    it.next(); // value-less toggle, no visible glyph
                }
                Some(_) => {
                    // Value code (\f… \C… \H… \pxq… \U… etc) — skip to ';'.
                    it.next();
                    while let Some(&(_, c)) = it.peek() {
                        it.next();
                        if c == ';' {
                            break;
                        }
                    }
                }
                None => {}
            },
            '{' | '}' => { /* group markers — not visible */ }
            '\n' | '\r' => {
                // Raw line break = paragraph break with a caret slot.
                para.push((i, i + ch.len_utf8(), '\n'));
                flush(&mut para, &mut result);
            }
            '%' if it.peek().map(|&(_, c)| c) == Some('%') => {
                it.next(); // second '%'
                match it.peek().copied() {
                    Some((k, '%')) => {
                        it.next();
                        para.push((i, k + 1, '%'));
                    }
                    Some((_, d)) if d.is_ascii_digit() => {
                        let mut last = i;
                        let mut n = 0;
                        while n < 3 {
                            match it.peek().copied() {
                                Some((m, c)) if c.is_ascii_digit() => {
                                    last = m;
                                    it.next();
                                    n += 1;
                                }
                                _ => break,
                            }
                        }
                        para.push((i, last + 1, '\u{25A1}'));
                    }
                    Some((m, c)) => {
                        it.next();
                        let g = match c {
                            'd' | 'D' => '°',
                            'c' | 'C' => 'Ø',
                            'p' | 'P' => '±',
                            other => other,
                        };
                        para.push((i, m + c.len_utf8(), g));
                    }
                    None => para.push((i, i + 1, '%')),
                }
            }
            _ => para.push((i, i + ch.len_utf8(), ch)),
        }
    }
    flush(&mut para, &mut result);
    result
}

// ── App-side editor driver ──────────────────────────────────────────────────

use crate::scene::convert::tessellate;
use crate::scene::model::wire_model::WireModel;
use iced::widget::text_editor::{Action, Edit};
use std::sync::Arc;

impl super::OpenCADStudio {
    /// Open the in-place editor for a new (`handle = None`) or existing MText.
    /// Open the rich MText editor for a new or existing MText / MultiLeader.
    /// The committed slot is chosen by the edited entity's type.
    pub(super) fn open_mtext_editor(
        &mut self,
        pos: Vec3,
        handle: Option<Handle>,
        initial: &str,
        height: f64,
    ) {
        let mut state = MTextEditorState::new(pos, initial, height, handle);
        if let Some(p) = self.tabs[self.active_tab].scene.selection.borrow().last_move_pos {
            state.screen_anchor = p;
        }
        // Seed attachment / line-spacing / box width from the entity being edited.
        if let Some(h) = handle {
            match self.tabs[self.active_tab].scene.document.get_entity(h) {
                Some(EntityType::MText(m)) => {
                    state.attachment = m.attachment_point;
                    state.line_spacing = m.line_spacing_factor as f32;
                    if !m.style.trim().is_empty() {
                        state.style = m.style.clone();
                    }
                    if m.rectangle_width > 0.0 {
                        state.rect_width = m.rectangle_width;
                    }
                }
                Some(EntityType::MultiLeader(ml)) => {
                    state.line_spacing = ml.context.line_spacing_factor as f32;
                    if ml.context.text_width > 0.0 {
                        state.rect_width = ml.context.text_width;
                    }
                }
                _ => {}
            }
        } else {
            // New MText inherits the document's current text style (STYLE),
            // not the "Standard" default. See #92.
            let cur_style = self.tabs[self.active_tab]
                .scene
                .document
                .header
                .current_text_style_name
                .clone();
            if !cur_style.trim().is_empty() {
                state.style = cur_style;
            }
        }
        self.mtext_editor = Some(state);
        self.rebuild_mtext_preview();
        // Place the caret at the end so typing works without a click first.
        let end = self.mtext_vis_count();
        if let Some(ed) = self.mtext_editor.as_mut() {
            ed.caret = end;
            ed.sel = Some((end, end));
        }
    }

    /// Re-tessellate the current editor text into the editor's OWN preview
    /// strokes (stored on the state, drawn in the dedicated preview area —
    /// never on the drawing canvas).
    pub(super) fn rebuild_mtext_preview(&mut self) {
        let i = self.active_tab;
        let Some(ed) = self.mtext_editor.as_ref() else { return };
        let mut mt = ed.build_mtext();
        // A trailing empty paragraph (a fresh line after Enter) is dropped by the
        // layout's parser, so it emits no caret box there and the caret would be
        // invisible on that line. Inject a space into the PREVIEW value only
        // (never committed) so the layout lays the line out and the caret gets a
        // box on it. Keyed off the current content — `ed.doc` is only re-synced
        // further down in this function, so it is still stale here.
        if content_has_trailing_break(&ed.content.text()) {
            match mt.value.rfind('}') {
                Some(pos) => mt.value.insert(pos, ' '),
                None => mt.value.push(' '),
            }
        }
        let entity = EntityType::MText(mt.clone());
        let anno = self.tabs[i].scene.annotation_scale;
        let bg = self.tabs[i].scene.bg_color;
        let wires: Vec<WireModel> = tessellate::tessellate(
            &self.tabs[i].scene.document,
            ed.editing.unwrap_or(Handle::new(1)),
            &entity,
            false,
            [0.92, 0.92, 0.92, 1.0],
            0.0,
            [0.0; 8],
            1.0,
            anno,
            None,
            bg,
            // Editor preview draws on a 2D canvas with no SDF shader — force the
            // glyph outline strokes so the text is visible (#308).
            true,
        );
        // Per-character boxes share the preview wires' absolute coordinate frame.
        let boxes = crate::entities::mtext::glyph_boxes(&mt, &self.tabs[i].scene.document);
        if let Some(ed) = self.mtext_editor.as_mut() {
            ed.preview_wires = wires;
            ed.glyph_boxes = boxes;
            // Re-sync the structured doc from the (authoritative) content — the
            // single sync point, since every edit path ends in a rebuild.
            // `parse_mtext` drops a trailing empty paragraph, so restore it when
            // the content ends in a hard break, else Enter at the end would lose
            // the new line.
            let raw = ed.content.text();
            let mut d = parse_mtext(&raw, true);
            if content_has_trailing_break(&raw) {
                d.paragraphs.push(MTextParagraph::new());
            }
            ed.doc = d;
        }
    }

    /// Splice text around the preview selection (visible-char range) in the
    /// raw value. `case` optionally transforms the selected slice. Returns
    /// true when a preview selection was present and applied.
    fn mtext_splice_sel(&mut self, prefix: &str, suffix: &str, case: Option<bool>) -> bool {
        let Some(ed) = self.mtext_editor.as_mut() else { return false };
        let Some((a, b)) = ed.sel else { return false };
        if a >= b {
            return false;
        }
        let raw = ed.content.text();
        let spans = visible_spans(&raw);
        if a >= spans.len() || b > spans.len() {
            return false;
        }
        let start = spans[a].0;
        let end = spans[b - 1].1;
        let mut s = raw;
        if let Some(upper) = case {
            let slice = &s[start..end];
            let repl = if upper { slice.to_uppercase() } else { slice.to_lowercase() };
            s.replace_range(start..end, &repl);
            // Length may change; recompute end for the suffix insert.
            let new_end = start + repl.len();
            s.insert_str(new_end, suffix);
            s.insert_str(start, prefix);
        } else {
            s.insert_str(end, suffix);
            s.insert_str(start, prefix);
        }
        ed.content = iced::widget::text_editor::Content::with_text(&s);
        ed.sel = None;
        true
    }

    /// Apply a character-format toggle to the preview selection (preferred) or
    /// the Edit-box selection. The stroke-font renderer has no true bold /
    /// italic, so Bold switches the run to the heavier Gothic stroke font and
    /// Italic applies an oblique slant — both produce a visible effect.
    pub(super) fn mtext_apply_fmt(&mut self, kind: MTextFmt) {
        // Font to restore to after a Bold run (the current global font).
        let restore = self
            .mtext_editor
            .as_ref()
            .map(|e| {
                if e.font.trim().is_empty() {
                    "Standard".to_string()
                } else {
                    e.font.clone()
                }
            })
            .unwrap_or_else(|| "Standard".to_string());
        let (pre, suf, case): (String, String, Option<bool>) = match kind {
            MTextFmt::Bold => ("\\fGothic;".into(), format!("\\f{restore};"), None),
            MTextFmt::Italic => ("\\Q15;".into(), "\\Q0;".into(), None),
            MTextFmt::Underline => ("\\L".into(), "\\l".into(), None),
            MTextFmt::Overline => ("\\O".into(), "\\o".into(), None),
            MTextFmt::Strike => ("\\K".into(), "\\k".into(), None),
            MTextFmt::Uppercase => (String::new(), String::new(), Some(true)),
            MTextFmt::Lowercase => (String::new(), String::new(), Some(false)),
        };
        if !self.mtext_splice_sel(&pre, &suf, case) {
            if let Some(ed) = self.mtext_editor.as_mut() {
                let sel = ed.content.selection().unwrap_or_default();
                let text = match case {
                    Some(true) => sel.to_uppercase(),
                    Some(false) => sel.to_lowercase(),
                    None => format!("{pre}{sel}{suf}"),
                };
                ed.content.perform(Action::Edit(Edit::Paste(Arc::new(text))));
            }
        }
        self.rebuild_mtext_preview();
    }

    /// Prefix the selection (or cursor) with a paragraph-alignment code.
    pub(super) fn mtext_apply_align(&mut self, align: ParaAlign) {
        let code = format!("\\pxq{};", align.code());
        if !self.mtext_splice_sel(&code, "", None) {
            if let Some(ed) = self.mtext_editor.as_mut() {
                let sel = ed.content.selection().unwrap_or_default();
                let text = format!("{code}{sel}");
                ed.content.perform(Action::Edit(Edit::Paste(Arc::new(text))));
            }
        }
        self.rebuild_mtext_preview();
    }

    // ── Caret editing directly on the preview ───────────────────────────────

    /// Insert text (or replace the current selection) at the preview caret.
    pub(super) fn mtext_type(&mut self, s: &str) {
        // MTEXT stores a hard line break as the `\P` code. A literal newline —
        // from Enter or a multi-line paste — is not a break to the layout (only
        // the caret logic honours it), so the preview would stay on one line.
        // Normalise every literal newline to `\P` so it breaks and the saved
        // value stays standard (#308).
        let normalized;
        let s: &str = if s.contains(['\n', '\r']) {
            normalized = s
                .replace("\r\n", "\\P")
                .replace('\n', "\\P")
                .replace('\r', "\\P");
            &normalized
        } else {
            s
        };
        if let Some(ed) = self.mtext_editor.as_mut() {
            // Edit the structured document as a flat cell list, then serialize
            // it back into the raw content the preview/commit still read.
            let mut cells = doc_to_cells(&ed.doc);
            let count = cells.len();
            let mut caret = match ed.sel {
                Some((a, b)) if a < b && b <= count => cells_delete_range(&mut cells, a, b),
                _ => ed.caret.min(count),
            };
            caret = clamp_insert(&cells, caret);
            let props = insert_props(&cells, caret);
            let ins = str_to_cells(s, &props);
            let added = ins.len();
            cells.splice(caret..caret, ins);
            caret += added;
            ed.content =
                text_editor::Content::with_text(&cells_to_doc(&cells).to_mtext_string());
            ed.caret = caret;
            ed.sel = Some((caret, caret));
            ed.caret_blink_on = true;
        }
        self.rebuild_mtext_preview();
    }

    /// Delete the selection, or the visible character before the caret.
    pub(super) fn mtext_backspace(&mut self) {
        if let Some(ed) = self.mtext_editor.as_mut() {
            let mut cells = doc_to_cells(&ed.doc);
            let count = cells.len();
            let caret = match ed.sel {
                Some((a, b)) if a < b && b <= count => cells_delete_range(&mut cells, a, b),
                _ if ed.caret > 0 && ed.caret <= count => {
                    cells_delete_range(&mut cells, ed.caret - 1, ed.caret)
                }
                _ => ed.caret.min(count),
            };
            ed.content =
                text_editor::Content::with_text(&cells_to_doc(&cells).to_mtext_string());
            ed.caret = caret;
            ed.sel = Some((caret, caret));
            ed.caret_blink_on = true;
        }
        self.rebuild_mtext_preview();
    }

    /// Delete the selection, or the visible character at the caret.
    pub(super) fn mtext_delete(&mut self) {
        if let Some(ed) = self.mtext_editor.as_mut() {
            let mut cells = doc_to_cells(&ed.doc);
            let count = cells.len();
            let caret = match ed.sel {
                Some((a, b)) if a < b && b <= count => cells_delete_range(&mut cells, a, b),
                _ if ed.caret < count => {
                    cells_delete_range(&mut cells, ed.caret, ed.caret + 1)
                }
                _ => ed.caret.min(count),
            };
            ed.content =
                text_editor::Content::with_text(&cells_to_doc(&cells).to_mtext_string());
            ed.caret = caret;
            ed.sel = Some((caret, caret));
            ed.caret_blink_on = true;
        }
        self.rebuild_mtext_preview();
    }

    /// Move the caret by `delta` visible characters (clears the selection).
    pub(super) fn mtext_caret_move(&mut self, delta: i32) {
        if let Some(ed) = self.mtext_editor.as_mut() {
            let n = doc_to_cells(&ed.doc).len() as i32;
            let c = (ed.caret as i32 + delta).clamp(0, n) as usize;
            ed.caret = c;
            ed.sel = Some((c, c));
            ed.caret_blink_on = true;
        }
    }

    /// Visible-character count of the current text.
    pub(super) fn mtext_vis_count(&self) -> usize {
        self.mtext_editor
            .as_ref()
            .map(|ed| doc_to_cells(&ed.doc).len())
            .unwrap_or(0)
    }

    /// Commit the editor — create a new MText or update the edited one.
    pub(super) fn mtext_commit(&mut self) -> bool {
        let i = self.active_tab;
        let Some(ed) = self.mtext_editor.take() else { return false };
        let body_empty = ed.content.text().trim().is_empty();
        let mut mt = ed.build_mtext();
        if body_empty {
            // Empty content: drop a new entity; leave an edited one untouched.
            self.refresh_properties();
            return false;
        }
        if let Some(h) = ed.editing {
            self.push_undo_snapshot(i, "MTEXT");
            match self.tabs[i].scene.document.get_entity_mut(h) {
                Some(EntityType::MText(t)) => {
                    t.value = mt.value;
                    t.height = mt.height;
                    t.attachment_point = mt.attachment_point;
                    t.line_spacing_factor = mt.line_spacing_factor;
                    t.rectangle_width = mt.rectangle_width;
                }
                Some(EntityType::MultiLeader(ml)) => {
                    ml.context.text_string = mt.value;
                    ml.context.text_height = mt.height;
                    ml.context.line_spacing_factor = mt.line_spacing_factor;
                    if mt.rectangle_width > 0.0 {
                        ml.context.text_width = mt.rectangle_width;
                    }
                }
                _ => {}
            }
            self.tabs[i].scene.bump_geometry();
            self.tabs[i].dirty = true;
        } else {
            // Align new MText to the active UCS (text runs along the UCS X axis).
            mt.rotation = self.tabs[i].ucs_rotation_angle();
            self.push_undo_snapshot(i, "MTEXT");
            self.commit_entity(EntityType::MText(mt));
            self.tabs[i].dirty = true;
        }
        self.refresh_properties();
        true
    }

    /// Discard the editor without changing the drawing.
    pub(super) fn mtext_cancel(&mut self) {
        self.mtext_editor = None;
    }
}

#[cfg(test)]
mod cell_tests {
    use super::*;

    fn rt(s: &str) -> String {
        cells_to_doc(&doc_to_cells(&parse_mtext(s, true))).to_mtext_string()
    }
    fn paras(s: &str) -> Vec<String> {
        parse_mtext(s, true)
            .paragraphs
            .iter()
            .map(|p| p.spans.iter().map(|sp| sp.text.clone()).collect())
            .collect()
    }
    fn cells_of(s: &str) -> Vec<Cell> {
        doc_to_cells(&parse_mtext(s, true))
    }

    #[test]
    fn roundtrip_idempotent() {
        for s in ["hello", "a b c", "one\\Ptwo", "a\\Pb\\Pc", "trailing\\P", "a  b", "\\S1/2;"] {
            let once = rt(s);
            let twice = rt(&once);
            assert_eq!(once, twice, "not idempotent for {s:?}: {once:?} vs {twice:?}");
        }
    }

    #[test]
    fn cell_count_matches_plain() {
        assert_eq!(cells_of("ab\\Pcd").len(), 5); // a b Break c d
        // parse_mtext drops the trailing empty paragraph; the app restores it in
        // rebuild() (raw.ends_with("\\P")), so this pure-parse count is 2.
        assert_eq!(cells_of("ab\\P").len(), 2);
    }

    #[test]
    fn trailing_break_survives_doc_roundtrip() {
        // A trailing Break must round-trip at the DOC level (the level the ops
        // work at) even though to_mtext_string+parse would drop it.
        let cells = vec![
            Cell::Char('a', SpanProperties::default()),
            Cell::Char('b', SpanProperties::default()),
            Cell::Break,
        ];
        let back = doc_to_cells(&cells_to_doc(&cells));
        assert_eq!(back, cells);
        // Typing after the trailing break lands on the new (empty) paragraph.
        let mut c2 = cells.clone();
        c2.splice(3..3, str_to_cells("x", &SpanProperties::default()));
        assert_eq!(paras(&cells_to_doc(&c2).to_mtext_string()), vec!["ab", "x"]);
    }

    #[test]
    fn insert_break_splits_paragraph() {
        let mut cells = cells_of("abcd");
        cells.splice(2..2, str_to_cells("\\P", &SpanProperties::default()));
        assert_eq!(paras(&cells_to_doc(&cells).to_mtext_string()), vec!["ab", "cd"]);
    }

    #[test]
    fn backspace_break_merges_paragraphs() {
        let mut cells = cells_of("ab\\Pcd"); // [a,b,Break,c,d]
        let caret = cells_delete_range(&mut cells, 2, 3); // delete the Break slot
        assert_eq!(caret, 2);
        assert_eq!(paras(&cells_to_doc(&cells).to_mtext_string()), vec!["abcd"]);
    }

    #[test]
    fn delete_range_across_paragraphs() {
        let mut cells = cells_of("abc\\Pdef"); // [a,b,c,Break,d,e,f]
        cells_delete_range(&mut cells, 1, 5); // b c Break d
        assert_eq!(paras(&cells_to_doc(&cells).to_mtext_string()), vec!["aef"]);
    }

    #[test]
    fn stacking_is_atomic() {
        let cells = cells_of("\\S1/2;");
        assert_eq!(cells.len(), 3);
        assert!(matches!(cells[0], Cell::Stack { head: true, .. }));
        assert!(cells[1].is_stack_cont() && cells[2].is_stack_cont());
        let mut c2 = cells.clone();
        let start = cells_delete_range(&mut c2, 1, 2); // interior delete widens to whole run
        assert_eq!(start, 0);
        assert!(c2.is_empty());
    }

    #[test]
    fn trailing_break_detection() {
        assert!(content_has_trailing_break("{ab\\P}")); // braced (to_mtext_string form)
        assert!(content_has_trailing_break("ab\\P")); // unbraced
        assert!(!content_has_trailing_break("{a\\Pb}")); // break in the middle
        assert!(!content_has_trailing_break("ab"));
        assert!(!content_has_trailing_break("{x\\\\P}")); // escaped literal backslash + P
    }

    #[test]
    fn enter_at_end_survives_rebuild() {
        // Type "ab", Enter at end -> cells [a, b, Break].
        let cells = vec![
            Cell::Char('a', SpanProperties::default()),
            Cell::Char('b', SpanProperties::default()),
            Cell::Break,
        ];
        let raw = cells_to_doc(&cells).to_mtext_string();
        // rebuild() re-parses + restores the trailing empty paragraph.
        let mut d = parse_mtext(&raw, true);
        if content_has_trailing_break(&raw) {
            d.paragraphs.push(MTextParagraph::new());
        }
        assert_eq!(doc_to_cells(&d).len(), 3, "trailing break must survive rebuild");
    }
}

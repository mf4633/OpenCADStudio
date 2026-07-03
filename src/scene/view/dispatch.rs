// Dispatch entry points for entity editing.

use acadrust::types::{Color as AcadColor, LineWeight, Transparency};
use acadrust::{EntityType, Handle};

use crate::command::EntityTransform;
use crate::entities::traits::EntityTypeOps;
use crate::scene::model::object::{GripDef, PropSection};
use crate::scene::cache::properties;

pub fn grips(entity: &EntityType) -> Vec<GripDef> {
    EntityTypeOps::grips(entity)
}

pub fn properties_sectioned(
    handle: Handle,
    entity: &EntityType,
    text_style_names: &[String],
) -> Vec<PropSection> {
    let mut sections = vec![properties::general_section(entity)];
    if let Some(viz) = properties::visualization_section(entity) {
        sections.push(viz);
    }
    let groups = entity.geometry_properties(text_style_names);
    if groups.is_empty() {
        sections.push(properties::fallback_properties(handle, entity));
    } else {
        sections.extend(groups);
    }
    sections
}

pub fn apply_common_prop(entity: &mut EntityType, field: &str, value: &str) {
    let e = entity.as_entity_mut();
    match field {
        "layer" => e.set_layer(value.to_string()),
        "linetype" => {
            entity.common_mut().linetype = if value == "ByLayer" {
                String::new()
            } else {
                value.to_string()
            };
        }
        "linetype_scale" => {
            if let Ok(v) = value.trim().parse::<f64>() {
                if v > 0.0 {
                    entity.common_mut().linetype_scale = v;
                }
            }
        }
        "transparency" => {
            if let Ok(pct) = value.trim().parse::<f64>() {
                let alpha = (pct.clamp(0.0, 100.0) / 100.0 * 255.0).round() as u8;
                entity
                    .as_entity_mut()
                    .set_transparency(Transparency::new(alpha));
            }
        }
        "thickness" => {
            if let Ok(v) = value.trim().parse::<f64>() {
                set_entity_thickness(entity, v);
            }
        }
        _ => {}
    }
}

/// The extrusion thickness (DXF 39) of the entities that carry one, or `None`
/// for entity types that have none. Thickness is a per-entity field but is
/// surfaced in the General group (as in a standard properties palette), so
/// this bridges the two.
pub fn entity_thickness(entity: &EntityType) -> Option<f64> {
    Some(match entity {
        EntityType::Arc(e) => e.thickness,
        EntityType::Circle(e) => e.thickness,
        EntityType::Line(e) => e.thickness,
        EntityType::LwPolyline(e) => e.thickness,
        EntityType::Point(e) => e.thickness,
        EntityType::PolyfaceMesh(e) => e.thickness,
        EntityType::Polyline2D(e) => e.thickness,
        EntityType::Shape(e) => e.thickness,
        EntityType::Solid(e) => e.thickness,
        EntityType::Text(e) => e.thickness,
        _ => return None,
    })
}

/// Set the extrusion thickness on the entity types that carry one; no-op for
/// the rest.
pub fn set_entity_thickness(entity: &mut EntityType, v: f64) {
    match entity {
        EntityType::Arc(e) => e.thickness = v,
        EntityType::Circle(e) => e.thickness = v,
        EntityType::Line(e) => e.thickness = v,
        EntityType::LwPolyline(e) => e.thickness = v,
        EntityType::Point(e) => e.thickness = v,
        EntityType::PolyfaceMesh(e) => e.thickness = v,
        EntityType::Polyline2D(e) => e.thickness = v,
        EntityType::Shape(e) => e.thickness = v,
        EntityType::Solid(e) => e.thickness = v,
        EntityType::Text(e) => e.thickness = v,
        _ => {}
    }
}

pub fn toggle_invisible(entity: &mut EntityType) {
    let cur = entity.as_entity_mut().is_invisible();
    entity.as_entity_mut().set_invisible(!cur);
}

pub fn apply_color(entity: &mut EntityType, color: AcadColor) {
    entity.as_entity_mut().set_color(color);
}

pub fn apply_line_weight(entity: &mut EntityType, lw: LineWeight) {
    entity.as_entity_mut().set_line_weight(lw);
}

pub fn apply_geom_prop(entity: &mut EntityType, field: &str, value: &str) {
    EntityTypeOps::apply_geom_prop(entity, field, value);
}

pub fn apply_grip(entity: &mut EntityType, grip_id: usize, apply: crate::scene::model::object::GripApply) {
    EntityTypeOps::apply_grip(entity, grip_id, apply);
}

pub fn apply_transform(entity: &mut EntityType, t: &EntityTransform) {
    EntityTypeOps::apply_transform(entity, t);
}

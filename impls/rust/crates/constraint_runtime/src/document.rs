//! Document: discrete sketch graph + committed Vector \(p\).
//!
//! Solve/assemble take `&Document` as geometry resolution plus an explicit
//! Vector (committed or a working copy). Structure and Vector stay separable
//! at call sites even though both live on Document.

use std::collections::HashMap;

use crate::constraint::{ConstraintEntry, ConstraintId};
use crate::constraint_kinds::ConstraintInst;
use crate::expr::{Expr, LengthExpr, PointExpr, VecExpr};
use crate::geom::CurveParam;
use crate::geometry::{
    Caps, Chart, Formula, Geometry, GeometryId, GeometryRole, GeometryShell, LengthFormula,
    PointFormula, ShapePiece, VecFormula,
};
use crate::geometry_kinds::{with_kind, GeometryInst, KindAttrs, LabelAttrs};
use crate::id::IdMap;
use crate::names::{NameError, Names, ObjectId};
use crate::style::{Style, StyleError, StylePatch, StyleRef, Styles, DEFAULT_STYLE};
use crate::vector::Vector;

/// One Congruence edge, indexed under each participating geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
struct IdentificationEdge {
    constraint: ConstraintId,
    a: PointExpr,
    b: PointExpr,
}

impl IdentificationEdge {
    fn partner_of(self, point: &PointExpr) -> Option<PointExpr> {
        if &self.a == point {
            Some(self.b)
        } else if &self.b == point {
            Some(self.a)
        } else {
            None
        }
    }
}

/// Drawing-wide settings (document-settings spec). Ranges are checked by the
/// command layer, not here.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentSettings {
    /// Millimetres per world unit.
    pub scale: f64,
    /// Font size of labels that inherit theirs, in pt.
    pub font_size: f64,
    /// LaTeX source with macros for every label in `latex` markup.
    pub preamble: String,
}

/// 1 world unit = 1 cm, 10 pt, no preamble.
impl Default for DocumentSettings {
    fn default() -> Self {
        Self {
            scale: 10.0,
            font_size: 10.0,
            preamble: String::new(),
        }
    }
}

/// Document = discrete sketch (geometries, constraints, incidence) + committed Vector.
#[derive(Clone, Debug, Default)]
pub struct Document {
    /// Geometry store (id → catalog instance; array lookup on the solve path).
    geometries: IdMap<GeometryId, GeometryInst>,
    /// Constraint store (id → entry; id order fixes assembly row order).
    constraints: IdMap<ConstraintId, ConstraintEntry>,
    /// `GeometryId` → constraints that reference it (incidence).
    constraints_on_geometry: HashMap<GeometryId, Vec<ConstraintId>>,
    /// `GeometryId` → Congruence edges touching that geometry (PointExpr partners).
    identification_by_geometry: HashMap<GeometryId, Vec<IdentificationEdge>>,
    /// Committed continuous parameters \(p\).
    pub vector: Vector,
    /// User-visible names of all geometries and constraints (never on the solve path).
    names: Names,
    settings: DocumentSettings,
    /// Named styles (styling); appearance only, never on the solve path.
    styles: Styles,
}

/// Objects dropped by [`Document::remove_geometry`] / [`Document::remove_constraint`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Removal {
    pub geometries: Vec<(GeometryId, String)>,
    /// Includes constraints removed together with a geometry.
    pub constraints: Vec<(ConstraintId, String)>,
}

impl Removal {
    /// Names of all removed objects (geometries first).
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.geometries
            .iter()
            .map(|(_, n)| n.as_str())
            .chain(self.constraints.iter().map(|(_, n)| n.as_str()))
    }
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vector(&self) -> &Vector {
        &self.vector
    }

    pub fn vector_mut(&mut self) -> &mut Vector {
        &mut self.vector
    }

    pub fn committed_vector(&self) -> &Vector {
        &self.vector
    }

    pub fn geometries(&self) -> &IdMap<GeometryId, GeometryInst> {
        &self.geometries
    }

    pub fn geometry(&self, id: GeometryId) -> Option<&GeometryInst> {
        self.geometries.get(id)
    }

    /// Geometry `id`, panicking if absent (expressions only reference live ids).
    #[inline]
    pub fn expect_geometry(&self, id: GeometryId) -> &GeometryInst {
        match self.geometries.get(id) {
            Some(g) => g,
            None => missing_geometry(id),
        }
    }

    /// Name of geometry or constraint `id`.
    pub fn name(&self, id: ObjectId) -> Option<&str> {
        self.names.name(id)
    }

    pub fn geometry_name(&self, id: GeometryId) -> Option<&str> {
        self.names.name(ObjectId::Geometry(id))
    }

    pub fn constraint_name(&self, id: ConstraintId) -> Option<&str> {
        self.names.name(ObjectId::Constraint(id))
    }

    /// Object named `name` (O(1)).
    pub fn lookup(&self, name: &str) -> Option<ObjectId> {
        self.names.lookup(name)
    }

    /// Rename `id` to `new`. Fails if `new` is invalid or used by another object;
    /// renaming to the current name succeeds. The drawing itself is unchanged.
    pub fn rename(&mut self, id: ObjectId, new: &str) -> Result<(), NameError> {
        self.names.rename(id, new)
    }

    pub fn settings(&self) -> &DocumentSettings {
        &self.settings
    }

    pub fn set_settings(&mut self, settings: DocumentSettings) {
        self.settings = settings;
    }

    pub fn styles(&self) -> &Styles {
        &self.styles
    }

    /// Create style `name` from `patch`, or apply `patch` to it.
    pub fn define_style(&mut self, name: &str, patch: &StylePatch) -> Result<(), StyleError> {
        self.styles.define(name, patch)
    }

    /// Replace the whole style table; `default` is added if `styles` lacks it.
    /// Every geometry must still refer to a style in the new table.
    pub fn set_styles(&mut self, styles: Styles) -> Result<(), StyleError> {
        if let Some(g) = self.geometries.values().find(|g| !styles.contains(&g.style().style)) {
            return Err(StyleError::Unknown(g.style().style.clone()));
        }
        self.styles = styles;
        Ok(())
    }

    /// Delete style `name`. `default` and styles that geometries refer to
    /// are rejected with [`StyleError::InUse`] naming those geometries.
    pub fn delete_style(&mut self, name: &str) -> Result<(), StyleError> {
        if !self.styles.contains(name) {
            return Err(StyleError::Unknown(name.to_owned()));
        }
        let users = self.style_users(name);
        if name == DEFAULT_STYLE || !users.is_empty() {
            return Err(StyleError::InUse(users));
        }
        self.styles.remove(name);
        Ok(())
    }

    /// Geometries that refer to style `name`, in id order.
    pub fn style_users(&self, name: &str) -> Vec<GeometryId> {
        self.geometries
            .values()
            .filter(|g| g.style().style == name)
            .map(|g| g.id())
            .collect()
    }

    /// Style reference and overrides of geometry `id`.
    pub fn geometry_style(&self, id: GeometryId) -> Option<&StyleRef> {
        self.geometry(id).map(|g| g.style())
    }

    /// Replace the style reference and overrides of geometry `id`; the named
    /// style must exist and override widths must be valid.
    pub fn set_geometry_style(&mut self, id: GeometryId, style: StyleRef) -> Result<(), StyleError> {
        if !self.styles.contains(&style.style) {
            return Err(StyleError::Unknown(style.style));
        }
        if !style.overrides.is_valid() {
            return Err(StyleError::InvalidWidth);
        }
        let g = self.geometries.get_mut(id).ok_or(StyleError::UnknownGeometry)?;
        g.set_style(style);
        Ok(())
    }

    /// Every style property of geometry `id`, resolved (styling).
    pub fn resolved_style(&self, id: GeometryId) -> Option<Style> {
        self.geometry_style(id).map(|r| self.styles.resolve(r))
    }

    /// Capability set of geometry `id` (`None` if `id` is not in the document).
    pub fn caps(&self, id: GeometryId) -> Option<Caps> {
        self.geometry(id).map(|g| g.kind().caps())
    }

    /// Exact shape of curve `id` at State `p` (geometry-ir); `None` if `id`
    /// is not in the document or has no curve capability.
    pub fn curve_shape(&self, id: GeometryId, p: &Vector) -> Option<Vec<ShapePiece>> {
        let g = self.geometry(id).filter(|g| g.kind().caps().curve)?;
        Some(with_kind!(g.kind(), K => K::shape(g.unpack::<K>(p))))
    }

    /// `formula` on `id` if the geometry exists and `has` its capability.
    ///
    /// Same `(id, tag)` expression the kind's static factory builds.
    fn capability_expr<F: Formula>(
        &self,
        id: GeometryId,
        has: fn(Caps) -> bool,
        formula: F,
    ) -> Option<Expr<F>> {
        self.caps(id)
            .filter(|&c| has(c))
            .map(|_| Expr::new(id, formula))
    }

    /// [`CurveExprs::on_curve_expr`](crate::geometry::CurveExprs::on_curve_expr) by id;
    /// `None` without the curve capability.
    pub fn on_curve_expr(&self, id: GeometryId, param: CurveParam) -> Option<PointExpr> {
        self.capability_expr(id, |c| c.curve, PointFormula::OnCurve { param })
    }

    /// [`CurveExprs::tangent_expr`](crate::geometry::CurveExprs::tangent_expr) by id;
    /// `None` without the curve capability.
    pub fn tangent_expr(&self, id: GeometryId, param: CurveParam) -> Option<VecExpr> {
        self.capability_expr(id, |c| c.curve, VecFormula::TangentAt { param })
    }

    /// [`CurveExprs::normal_expr`](crate::geometry::CurveExprs::normal_expr) by id;
    /// `None` without the curve capability.
    pub fn normal_expr(&self, id: GeometryId, param: CurveParam) -> Option<VecExpr> {
        self.capability_expr(id, |c| c.curve, VecFormula::NormalAt { param })
    }

    /// [`CurveExprs::length_expr`](crate::geometry::CurveExprs::length_expr) by id;
    /// `None` without the curve capability.
    pub fn length_expr(&self, id: GeometryId) -> Option<LengthExpr> {
        self.capability_expr(id, |c| c.curve, LengthFormula::Length)
    }

    /// [`EndpointExprs::start_expr`](crate::geometry::EndpointExprs::start_expr) by id;
    /// `None` without the endpoints capability.
    pub fn start_expr(&self, id: GeometryId) -> Option<PointExpr> {
        self.capability_expr(id, |c| c.endpoints, PointFormula::Start)
    }

    /// [`EndpointExprs::end_expr`](crate::geometry::EndpointExprs::end_expr) by id;
    /// `None` without the endpoints capability.
    pub fn end_expr(&self, id: GeometryId) -> Option<PointExpr> {
        self.capability_expr(id, |c| c.endpoints, PointFormula::End)
    }

    /// [`LineExprs::direction_expr`](crate::geometry::LineExprs::direction_expr) by id;
    /// `None` without the line capability.
    pub fn direction_expr(&self, id: GeometryId) -> Option<VecExpr> {
        self.capability_expr(id, |c| c.line, VecFormula::Direction)
    }

    /// [`CircularExprs::center_expr`](crate::geometry::CircularExprs::center_expr) by id;
    /// `None` without the circular capability.
    pub fn center_expr(&self, id: GeometryId) -> Option<PointExpr> {
        self.capability_expr(id, |c| c.circular, PointFormula::Center)
    }

    /// [`CircularExprs::radius_expr`](crate::geometry::CircularExprs::radius_expr) by id;
    /// `None` without the circular capability.
    pub fn radius_expr(&self, id: GeometryId) -> Option<LengthExpr> {
        self.capability_expr(id, |c| c.circular, LengthFormula::Radius)
    }

    /// [`AnchorExprs::origin_expr`](crate::geometry::AnchorExprs::origin_expr) by id;
    /// `None` without the anchor capability.
    pub fn origin_expr(&self, id: GeometryId) -> Option<PointExpr> {
        self.capability_expr(id, |c| c.anchor, PointFormula::Origin)
    }

    pub fn constraints(&self) -> &IdMap<ConstraintId, ConstraintEntry> {
        &self.constraints
    }

    pub fn constraint(&self, id: ConstraintId) -> Option<&ConstraintInst> {
        self.constraints.get(id).map(|e| e.body())
    }

    /// Constraint ids that reference geometry `id` (empty if none / unknown id).
    pub fn constraints_on(&self, id: GeometryId) -> &[ConstraintId] {
        self.constraints_on_geometry
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Direct Congruence partners of `point` (O(edges on its geometry)).
    pub fn points_identified_with(&self, point: &PointExpr) -> Vec<PointExpr> {
        self.identification_by_geometry
            .get(&point.referenced_geometry())
            .into_iter()
            .flatten()
            .filter_map(|e| e.partner_of(point))
            .collect()
    }

    pub fn parameter_count(&self) -> usize {
        self.geometries.values().map(|g| g.parameter_count()).sum()
    }

    /// Pack chart init → extend Vector → register a geometry instance.
    ///
    /// Chart pack receives only kind init (no identity). Id and Vector offset are
    /// assigned here from [`IdMap::next_id`] and the current Vector length.
    pub fn add_geometry<K: Geometry>(&mut self, role: GeometryRole, init: K::Init) -> GeometryId {
        self.add_geometry_with::<K>(role, init, K::default_attrs())
    }

    /// [`Document::add_geometry`] with explicit attributes; `attrs` must be of
    /// kind `K` (as [`Geometry::default_attrs`]).
    pub fn add_geometry_with<K: Geometry>(
        &mut self,
        role: GeometryRole,
        init: K::Init,
        attrs: KindAttrs,
    ) -> GeometryId {
        let packed = K::pack(K::canonical(init));
        debug_assert_eq!(packed.as_ref().len(), K::PARAMETER_COUNT);
        let id = self.geometries.next_id();
        let param_offset = self.vector.len();
        let mut g = GeometryInst::of::<K>(GeometryShell::new(id, role, param_offset));
        let matches = g.set_attrs(attrs);
        debug_assert!(matches, "attributes of another kind for {:?}", K::KIND);
        self.vector.extend_from_slice(packed.as_ref());
        self.push_geometry(g);
        debug_assert_eq!(self.vector.len(), self.parameter_count());
        id
    }

    /// Replace the attributes of geometry `id`; `false` (unchanged) if it does
    /// not exist or `attrs` belongs to another kind.
    pub fn set_geometry_attrs(&mut self, id: GeometryId, attrs: KindAttrs) -> bool {
        self.geometries
            .get_mut(id)
            .is_some_and(|g| g.set_attrs(attrs))
    }

    /// Text attributes of label `id` (`None` if it is not a label).
    pub fn label_attrs(&self, id: GeometryId) -> Option<&LabelAttrs> {
        match self.geometry(id)?.attrs() {
            KindAttrs::Label(a) => Some(a),
            KindAttrs::None => None,
        }
    }

    /// Replace the text attributes of label `id`; `false` if it is not a label.
    pub fn set_label_attrs(&mut self, id: GeometryId, attrs: LabelAttrs) -> bool {
        self.geometries
            .get_mut(id)
            .is_some_and(|g| g.set_attrs(KindAttrs::Label(attrs)))
    }

    /// Register a geometry whose parameters already live in an external Vector.
    /// Prefer [`Document::add_geometry`] so graph and committed Vector stay matched.
    /// Insert `g` and give it the next automatic name of its kind.
    pub(crate) fn push_geometry(&mut self, g: GeometryInst) {
        let name = self.names.next_auto(g.kind().name_prefix());
        self.names.insert(ObjectId::Geometry(g.id()), name);
        self.geometries.insert(g.id(), g);
    }

    /// Set geometry role (result vs construction). Returns false if id missing.
    pub fn set_geometry_role(&mut self, id: GeometryId, role: GeometryRole) -> bool {
        let Some(g) = self.geometries.get_mut(id) else {
            return false;
        };
        g.set_role(role);
        true
    }

    pub fn add_constraint(&mut self, c: impl Into<ConstraintInst>) -> ConstraintId {
        let body = c.into();
        let geos = body.referenced_geometries();
        let id = self.constraints.next_id();
        if let Some(g) = body.as_congruence() {
            self.link_identification(id, g.a, g.b);
        }
        let name = self.names.next_auto(body.name_prefix());
        self.names.insert(ObjectId::Constraint(id), name);
        self.constraints.insert(id, ConstraintEntry { id, body });
        self.link_constraint(id, &geos);
        id
    }

    /// Remove a constraint by id. Returns false if missing.
    /// Remove constraint `id`; `None` if it is not in the document.
    pub fn remove_constraint(&mut self, id: ConstraintId) -> Option<Removal> {
        let entry = self.constraints.remove(id)?;
        let name = self.names.remove(ObjectId::Constraint(id));
        let geos = entry.body().referenced_geometries();
        let congruence = entry.body().as_congruence().map(|g| (g.a, g.b));
        self.unlink_constraint(id, &geos);
        if let Some((a, b)) = congruence {
            self.unlink_identification(id, a, b);
        }
        Some(Removal {
            constraints: vec![(id, name.expect("constraints are named"))],
            ..Removal::default()
        })
    }

    /// Remove a geometry: compact Vector, remap offsets, drop orphan constraints.
    /// Remove geometry `id` and the constraints that reference it; `None` if
    /// it is not in the document.
    pub fn remove_geometry(&mut self, id: GeometryId) -> Option<Removal> {
        let removed = self.geometries.remove(id)?;
        let name = self.names.remove(ObjectId::Geometry(id));

        let block = removed.param_block();
        let (start, removed_len) = (block.start, block.len());
        let end = start + removed_len;
        assert!(end <= self.vector.len());
        self.vector.drain(start..end);

        for g in self.geometries.values_mut() {
            g.reindex_after_removal(start, removed_len);
        }

        let orphans: Vec<ConstraintId> = self
            .constraints
            .values()
            .filter(|e| e.body().references_geometry(id))
            .map(|e| e.id)
            .collect();
        self.constraints
            .retain(|e| !e.body().references_geometry(id));
        self.rebuild_incidence();
        let constraints = orphans
            .into_iter()
            .map(|c| {
                let n = self.names.remove(ObjectId::Constraint(c));
                (c, n.expect("constraints are named"))
            })
            .collect();

        debug_assert_eq!(self.vector.len(), self.parameter_count());
        Some(Removal {
            geometries: vec![(id, name.expect("geometries are named"))],
            constraints,
        })
    }

    /// Snapshot of committed state for undo / drag.
    pub fn clone_vector(&self) -> Vector {
        self.vector.clone()
    }

    pub fn restore_vector(&mut self, p: &Vector) {
        self.vector.copy_from(p);
    }

    /// Rewrite every geometry block of `p` to its canonical chart values
    /// ([`Geometry::canonical`]); the curves are unchanged.
    pub fn canonicalize(&self, p: &mut Vector) {
        for g in self.geometries.values() {
            let block = g.param_block();
            with_kind!(g.kind(), K => {
                let canonical = K::canonical(g.unpack::<K>(p));
                p[block].copy_from_slice(K::pack(canonical).as_ref());
            });
        }
    }

    pub fn commit_vector(&mut self, p: &Vector) {
        self.restore_vector(p);
    }

    fn link_constraint(&mut self, cid: ConstraintId, geos: &[GeometryId]) {
        for &gid in geos {
            self.constraints_on_geometry
                .entry(gid)
                .or_default()
                .push(cid);
        }
    }

    fn unlink_constraint(&mut self, cid: ConstraintId, geos: &[GeometryId]) {
        for &gid in geos {
            let Some(list) = self.constraints_on_geometry.get_mut(&gid) else {
                continue;
            };
            list.retain(|&x| x != cid);
            if list.is_empty() {
                self.constraints_on_geometry.remove(&gid);
            }
        }
    }

    fn link_identification(&mut self, cid: ConstraintId, a: PointExpr, b: PointExpr) {
        let edge = IdentificationEdge {
            constraint: cid,
            a,
            b,
        };
        let ga = a.referenced_geometry();
        let gb = b.referenced_geometry();
        self.identification_by_geometry
            .entry(ga)
            .or_default()
            .push(edge);
        if gb != ga {
            self.identification_by_geometry
                .entry(gb)
                .or_default()
                .push(edge);
        }
    }

    fn unlink_identification(&mut self, cid: ConstraintId, a: PointExpr, b: PointExpr) {
        for gid in crate::constraint::unique_geometry_ids([
            a.referenced_geometry(),
            b.referenced_geometry(),
        ]) {
            let Some(list) = self.identification_by_geometry.get_mut(&gid) else {
                continue;
            };
            list.retain(|e| e.constraint != cid);
            if list.is_empty() {
                self.identification_by_geometry.remove(&gid);
            }
        }
    }

    fn rebuild_incidence(&mut self) {
        self.constraints_on_geometry.clear();
        self.identification_by_geometry.clear();
        for e in self.constraints.values() {
            for gid in e.body().referenced_geometries() {
                self.constraints_on_geometry
                    .entry(gid)
                    .or_default()
                    .push(e.id);
            }
            if let Some(g) = e.body().as_congruence() {
                let edge = IdentificationEdge {
                    constraint: e.id,
                    a: g.a,
                    b: g.b,
                };
                let ga = g.a.referenced_geometry();
                let gb = g.b.referenced_geometry();
                self.identification_by_geometry
                    .entry(ga)
                    .or_default()
                    .push(edge);
                if gb != ga {
                    self.identification_by_geometry
                        .entry(gb)
                        .or_default()
                        .push(edge);
                }
            }
        }
    }
}

#[cold]
#[track_caller]
fn missing_geometry(id: GeometryId) -> ! {
    panic!("geometry id {id:?} not in document")
}

#[cfg(test)]
#[path = "tests_document.rs"]
mod tests_document;

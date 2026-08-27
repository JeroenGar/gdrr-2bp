use crate::core::cost::Cost;
use crate::core::entities::node::{Node, NodeKey};
use crate::core::entities::parttype::PartType;
use crate::core::layout_index::LayoutIndex;
use crate::core::leftover_valuator;
use crate::core::orientation::Orientation;
use crate::core::rotation::Rotation;

/// Representation of how a part can be inserted into a Node of a Layout
/// Layouts can use InsertionBlueprints to insert parts

#[derive(Debug, Clone)]
pub struct InsertionBlueprint<'a> {
    layout_i: LayoutIndex,
    original_node_i: NodeKey,
    shape: InsertionShape,
    rotation: Rotation,
    parttype: &'a PartType,
    cost: Cost,
}

impl<'a> InsertionBlueprint<'a> {
    pub(crate) fn new(
        layout_i: LayoutIndex,
        original_node_i: NodeKey,
        shape: InsertionShape,
        rotation: Rotation,
        parttype: &'a PartType,
        original_node: &Node,
        leftover_valuation_power: f32,
    ) -> Self {
        let mut blueprint = Self {
            layout_i,
            original_node_i,
            shape,
            rotation,
            parttype,
            cost: Cost::empty(),
        };
        let replacement_cost = blueprint
            .nodes(original_node)
            .iter()
            .flatten()
            .filter(|node| node.kind == InsertionNodeKind::Empty)
            .fold(Cost::empty(), |cost, node| {
                cost.add_leftover_value(leftover_valuator::valuate(
                    node.width * node.height,
                    leftover_valuation_power,
                ))
            });
        blueprint.cost = replacement_cost - original_node.calculate_cost(leftover_valuation_power);
        blueprint
    }

    pub(crate) fn nodes(&self, original_node: &Node) -> [Option<InsertionNode>; 5] {
        let orientation = original_node.next_cut_orient();
        let part_size = match self.rotation {
            Rotation::Default => self.parttype.size(),
            Rotation::Rotated => self.parttype.rotated_size(),
        };
        let (width, height) = (original_node.width(), original_node.height());
        let (part_width, part_height) = (part_size.width(), part_size.height());

        let (
            along_strip,
            along_remainder,
            across_strip,
            across_remainder,
            along_inner_remainder,
            across_inner_remainder,
        ) = match orientation {
            Orientation::Horizontal => (
                (part_width, height),
                (width - part_width, height),
                (width, part_height),
                (width, height - part_height),
                (part_width, height - part_height),
                (width - part_width, part_height),
            ),
            Orientation::Vertical => (
                (width, part_height),
                (width, height - part_height),
                (part_width, height),
                (width - part_width, height),
                (width - part_width, part_height),
                (part_width, height - part_height),
            ),
        };
        let node = |parent, (width, height), next_cut_orient, kind| {
            Some(InsertionNode {
                parent,
                width,
                height,
                next_cut_orient,
                kind,
            })
        };

        match self.shape {
            InsertionShape::AlongCurrentCut => [
                node(None, along_strip, orientation, InsertionNodeKind::Part),
                node(None, along_remainder, orientation, InsertionNodeKind::Empty),
                None,
                None,
                None,
            ],
            InsertionShape::AcrossCurrentCut => [
                node(
                    None,
                    (width, height),
                    orientation,
                    InsertionNodeKind::Structure,
                ),
                node(
                    Some(0),
                    across_strip,
                    orientation.rotate(),
                    InsertionNodeKind::Part,
                ),
                node(
                    Some(0),
                    across_remainder,
                    orientation.rotate(),
                    InsertionNodeKind::Empty,
                ),
                None,
                None,
            ],
            InsertionShape::AlongThenAcross => [
                node(None, along_strip, orientation, InsertionNodeKind::Structure),
                node(
                    Some(0),
                    (part_width, part_height),
                    orientation.rotate(),
                    InsertionNodeKind::Part,
                ),
                node(
                    Some(0),
                    along_inner_remainder,
                    orientation.rotate(),
                    InsertionNodeKind::Empty,
                ),
                node(None, along_remainder, orientation, InsertionNodeKind::Empty),
                None,
            ],
            InsertionShape::AcrossThenAlong => [
                node(
                    None,
                    (width, height),
                    orientation,
                    InsertionNodeKind::Structure,
                ),
                node(
                    Some(0),
                    across_strip,
                    orientation.rotate(),
                    InsertionNodeKind::Structure,
                ),
                node(
                    Some(1),
                    (part_width, part_height),
                    orientation,
                    InsertionNodeKind::Part,
                ),
                node(
                    Some(1),
                    across_inner_remainder,
                    orientation,
                    InsertionNodeKind::Empty,
                ),
                node(
                    Some(0),
                    across_remainder,
                    orientation.rotate(),
                    InsertionNodeKind::Empty,
                ),
            ],
        }
    }

    pub fn parttype(&self) -> &'a PartType {
        self.parttype
    }

    pub fn cost(&self) -> &Cost {
        &self.cost
    }

    pub fn layout_index(&self) -> &LayoutIndex {
        &self.layout_i
    }

    pub fn original_node_index(&self) -> &NodeKey {
        &self.original_node_i
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum InsertionShape {
    AlongCurrentCut,
    AcrossCurrentCut,
    AlongThenAcross,
    AcrossThenAlong,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InsertionNode {
    pub parent: Option<usize>,
    pub width: u64,
    pub height: u64,
    pub next_cut_orient: Orientation,
    pub kind: InsertionNodeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InsertionNodeKind {
    Part,
    Structure,
    Empty,
}

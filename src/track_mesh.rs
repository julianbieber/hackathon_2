use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, Mesh},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};
use std::f32::consts::PI;

use crate::track_gen::BlockType;

// Track width is a global setting - adjust as needed
const TRACK_WIDTH: f32 = 1.0;

pub fn generate_mesh(block: &BlockType) -> Mesh {
    match block {
        BlockType::Straight { length } => generate_straight_mesh(*length),
        BlockType::Turn { angle, radius } => generate_turn_mesh(*angle, *radius),
        BlockType::Curve { radius, angle } => generate_curve_mesh(*radius, *angle),
        BlockType::CurveOffset {
            radius,
            angle,
            offset,
        } => generate_curve_offset_mesh(*radius, *angle, *offset),
        BlockType::Slope {
            length,
            height_change,
        } => generate_slope_mesh(*length, *height_change),
    }
}

// For a straight track segment
fn generate_straight_mesh(length: f32) -> Mesh {
    let half_width = TRACK_WIDTH / 2.0;

    // Vertices for a rectangle
    let positions = vec![
        [-half_width, 0.0, 0.0],    // Left back
        [half_width, 0.0, 0.0],     // Right back
        [half_width, length, 0.0],  // Right front
        [-half_width, length, 0.0], // Left front
    ];

    // UVs for proper texture mapping
    let uvs = vec![
        [0.0, 0.0],    // Left back
        [1.0, 0.0],    // Right back
        [1.0, length], // Right front
        [0.0, length], // Left front
    ];

    // All normals point upwards
    let normals = vec![[0.0, 0.0, 1.0]; 4];

    // Create two triangles to form the rectangle
    let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2]);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(indices)
}

// For a turn block
fn generate_turn_mesh(angle: f32, radius: f32) -> Mesh {
    // Convert angle to radians
    let angle_rad = angle * PI / 180.0;
    let segment_count = (angle_rad.abs() * 10.0) as u32 + 1;

    let inner_radius = radius - TRACK_WIDTH / 2.0;
    let outer_radius = radius + TRACK_WIDTH / 2.0;

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Center of the curve
    let center_x = if angle_rad >= 0.0 { radius } else { -radius };

    // Calculate the length of the arc for UV mapping
    let arc_length = radius * angle_rad.abs();

    // Generate points along the inner and outer arcs
    for i in 0u32..=segment_count {
        let t = (i as f32) / (segment_count as f32);
        let current_angle = if angle_rad >= 0.0 {
            PI / 2.0 - angle_rad * t
        } else {
            PI / 2.0 + angle_rad.abs() * t
        };

        // Calculate positions based on the center and angle
        let inner_x = center_x + inner_radius * current_angle.cos();
        let inner_y = inner_radius * current_angle.sin();

        let outer_x = center_x + outer_radius * current_angle.cos();
        let outer_y = outer_radius * current_angle.sin();

        positions.push([inner_x, inner_y, 0.0]);
        positions.push([outer_x, outer_y, 0.0]);

        // UV coordinates - stretch texture along the arc
        let arc_t = t * arc_length;
        uvs.push([0.0, arc_t]);
        uvs.push([1.0, arc_t]);

        // Create triangles connecting adjacent points
        if i < segment_count {
            let base_idx = i * 2;
            indices.extend_from_slice(&[
                base_idx,
                base_idx + 2,
                base_idx + 1,
                base_idx + 1,
                base_idx + 2,
                base_idx + 3,
            ]);
        }
    }

    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

// For a curve block (smoother than turn)
fn generate_curve_mesh(radius: f32, angle: f32) -> Mesh {
    // Similar to turn but with more segments for smoother appearance
    let angle_rad = angle * PI / 180.0;
    let segment_count = (angle_rad.abs() * 20.0) as u32 + 1; // More segments

    let inner_radius = radius - TRACK_WIDTH / 2.0;
    let outer_radius = radius + TRACK_WIDTH / 2.0;

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Center of the curve
    let center_x = if angle_rad >= 0.0 { radius } else { -radius };

    // Calculate the length of the arc for UV mapping
    let arc_length = radius * angle_rad.abs();

    // Generate points along the inner and outer arcs
    for i in 0..=segment_count {
        let t = (i as f32) / (segment_count as f32);
        let current_angle = if angle_rad >= 0.0 {
            PI / 2.0 - angle_rad * t
        } else {
            PI / 2.0 + angle_rad.abs() * t
        };

        let inner_x = center_x + inner_radius * current_angle.cos();
        let inner_y = inner_radius * current_angle.sin();

        let outer_x = center_x + outer_radius * current_angle.cos();
        let outer_y = outer_radius * current_angle.sin();

        positions.push([inner_x, inner_y, 0.0]);
        positions.push([outer_x, outer_y, 0.0]);

        // UV coordinates
        let arc_t = t * arc_length;
        uvs.push([0.0, arc_t]);
        uvs.push([1.0, arc_t]);

        if i < segment_count {
            let base_idx = i * 2;
            indices.extend_from_slice(&[
                base_idx,
                base_idx + 2,
                base_idx + 1,
                base_idx + 1,
                base_idx + 2,
                base_idx + 3,
            ]);
        }
    }

    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

// For a curve with offset
fn generate_curve_offset_mesh(radius: f32, angle: f32, offset: f32) -> Mesh {
    let angle_rad = angle * PI / 180.0;
    let segment_count = (angle_rad.abs() * 20.0) as u32 + 1;

    let inner_radius = radius - TRACK_WIDTH / 2.0;
    let outer_radius = radius + TRACK_WIDTH / 2.0;

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Center of the curve
    let center_x = if angle_rad >= 0.0 { radius } else { -radius };

    // Calculate the length of the arc for UV mapping
    let arc_length = radius * angle_rad.abs();

    // Generate points along the inner and outer arcs with offset
    for i in 0..=segment_count {
        let t = (i as f32) / (segment_count as f32);
        let current_angle = if angle_rad >= 0.0 {
            PI / 2.0 - angle_rad * t
        } else {
            PI / 2.0 + angle_rad.abs() * t
        };

        // Apply offset perpendicular to the curve direction
        let offset_x = offset * current_angle.cos();
        let offset_y = offset * current_angle.sin();

        let inner_x = center_x + inner_radius * current_angle.cos() + offset_x;
        let inner_y = inner_radius * current_angle.sin() + offset_y;

        let outer_x = center_x + outer_radius * current_angle.cos() + offset_x;
        let outer_y = outer_radius * current_angle.sin() + offset_y;

        positions.push([inner_x, inner_y, 0.0]);
        positions.push([outer_x, outer_y, 0.0]);

        let arc_t = t * arc_length;
        uvs.push([0.0, arc_t]);
        uvs.push([1.0, arc_t]);

        if i < segment_count {
            let base_idx = i * 2;
            indices.extend_from_slice(&[
                base_idx,
                base_idx + 2,
                base_idx + 1,
                base_idx + 1,
                base_idx + 2,
                base_idx + 3,
            ]);
        }
    }

    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

// For a slope
fn generate_slope_mesh(length: f32, height_change: f32) -> Mesh {
    let half_width = TRACK_WIDTH / 2.0;

    // Vertices for a rectangle with changing height
    let positions = vec![
        [-half_width, 0.0, 0.0],              // Left back
        [half_width, 0.0, 0.0],               // Right back
        [half_width, length, height_change],  // Right front
        [-half_width, length, height_change], // Left front
    ];

    // UVs for texture mapping
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, length], [0.0, length]];

    // Calculate normal for the sloped surface
    let dx = 0.0;
    let dy = length;
    let dz = height_change;

    // Cross product to get normal perpendicular to surface
    let normal = [-dz, 0.0, dy];

    // Normalize the normal vector
    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    let normal = [normal[0] / length, normal[1] / length, normal[2] / length];

    let normals = vec![normal; 4];

    // Create two triangles (counter-clockwise winding)
    let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2]);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(indices)
}

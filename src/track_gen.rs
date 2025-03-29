use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct BlockTransform {
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Straight {
        length: f32,
    },
    Turn {
        angle: f32,
        radius: f32,
    },
    Curve {
        radius: f32,
        angle: f32,
    },
    CurveOffset {
        radius: f32,
        angle: f32,
        offset: f32,
    },
    Slope {
        length: f32,
        height_change: f32,
    },
}

#[derive(Debug, Clone)]
pub struct TrackSegment {
    pub block_type: BlockType,
    pub transform: BlockTransform,
}

pub struct Track {
    pub segments: Vec<TrackSegment>,
    pub current_end: BlockTransform,
    noise: Perlin,
}

impl Track {
    pub fn generate(seed: u32, initial_length: f32) -> Self {
        let mut track = Self {
            segments: Vec::new(),
            current_end: BlockTransform {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            },
            noise: Perlin::new(seed),
        };

        track.append_block(BlockType::Slope {
            length: initial_length,
            height_change: -3.0,
        });

        for _ in 0..50 {
            let next_block = track.select_next_block();
            track.append_block(next_block);
        }

        track
    }

    fn select_next_block(&self) -> BlockType {
        let noise_value = self.noise.get([self.segments.len() as f64 * 0.3, 0.0]);

        match (noise_value * 2.0).abs() {
            v if v < 0.3 => BlockType::Straight { length: 10.0 },
            v if v < 0.6 => BlockType::Turn {
                angle: PI / 2.0,
                radius: 8.0,
            },
            v if v < 1.0 => BlockType::Curve {
                radius: 15.0,
                angle: PI / 4.0,
            },
            v if v < 1.4 => BlockType::CurveOffset {
                radius: 12.0,
                angle: PI / 3.0,
                offset: 3.0,
            },
            _ => BlockType::Slope {
                length: 15.0,
                height_change: -2.0,
            },
        }
    }

    fn append_block(&mut self, block_type: BlockType) {
        let end_transform = self.calculate_end_transform(&block_type);

        self.segments.push(TrackSegment {
            block_type,
            transform: self.current_end,
        });

        self.current_end = end_transform;
    }

    fn calculate_end_transform(&self, block_type: &BlockType) -> BlockTransform {
        match block_type {
            BlockType::Straight { length } => BlockTransform {
                position: self.current_end.position + self.current_end.rotation * Vec3::Z * *length,
                rotation: self.current_end.rotation,
            },

            BlockType::Turn { angle, radius } => {
                let rotation = self.current_end.rotation * Quat::from_rotation_y(*angle);
                BlockTransform {
                    position: self.current_end.position
                        + rotation * Vec3::new(*radius, 0.0, *radius),
                    rotation,
                }
            }

            BlockType::Curve { radius, angle } => {
                let rotation = self.current_end.rotation * Quat::from_rotation_y(*angle);
                BlockTransform {
                    position: self.current_end.position
                        + self.current_end.rotation
                            * Vec3::new(radius * angle.sin(), 0.0, radius * angle.cos()),
                    rotation,
                }
            }

            BlockType::CurveOffset {
                radius,
                angle,
                offset,
            } => {
                let rotation = self.current_end.rotation * Quat::from_rotation_y(*angle);
                BlockTransform {
                    position: self.current_end.position
                        + self.current_end.rotation
                            * Vec3::new(radius * angle.sin() + offset, 0.0, radius * angle.cos()),
                    rotation,
                }
            }

            BlockType::Slope {
                length,
                height_change,
            } => {
                let rotation = self.current_end.rotation
                    * Quat::from_rotation_x(-height_change.atan2(*length));

                BlockTransform {
                    position: self.current_end.position
                        + self.current_end.rotation * Vec3::new(0.0, *height_change, *length),
                    rotation,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{BlockTransform, BlockType, Track};
    use approx::assert_relative_eq;
    use bevy::prelude::*;
    use std::f32::consts::FRAC_PI_2;

    const EPSILON: f32 = 0.001;

    fn calculate_end(block_type: &BlockType, start: BlockTransform) -> BlockTransform {
        match block_type {
            BlockType::Straight { length } => {
                let direction = start.rotation * Vec3::Z;
                BlockTransform {
                    position: start.position + direction * length,
                    rotation: start.rotation,
                }
            }
            BlockType::Turn { angle, radius } => {
                let rotation = start.rotation * Quat::from_rotation_y(*angle);
                BlockTransform {
                    position: start.position + rotation * Vec3::new(*radius, 0.0, *radius),
                    rotation,
                }
            }
            BlockType::Curve { radius, angle } => {
                let rotation = start.rotation * Quat::from_rotation_y(*angle);
                BlockTransform {
                    position: start.position
                        + start.rotation
                            * Vec3::new(radius * angle.sin(), 0.0, radius * angle.cos()),
                    rotation,
                }
            }
            BlockType::CurveOffset {
                radius,
                angle,
                offset,
            } => {
                let rotation = start.rotation * Quat::from_rotation_y(*angle);
                BlockTransform {
                    position: start.position
                        + start.rotation
                            * Vec3::new(radius * angle.sin() + offset, 0.0, radius * angle.cos()),
                    rotation,
                }
            }
            BlockType::Slope {
                length,
                height_change,
            } => {
                let rotation =
                    start.rotation * Quat::from_rotation_x(-height_change.atan2(*length));
                BlockTransform {
                    position: start.position
                        + start.rotation * Vec3::new(0.0, *height_change, *length),
                    rotation,
                }
            }
        }
    }

    #[test]
    fn test_consecutive_segments_connect() {
        let track = Track::generate(123, 10.0);

        for i in 0..track.segments.len() - 1 {
            let current = &track.segments[i];
            let next = &track.segments[i + 1];

            let computed_end = calculate_end(&current.block_type, current.transform);

            assert_relative_eq!(
                computed_end.position.x,
                next.transform.position.x,
                epsilon = EPSILON
            );
            assert_relative_eq!(
                computed_end.position.y,
                next.transform.position.y,
                epsilon = EPSILON
            );
            assert_relative_eq!(
                computed_end.position.z,
                next.transform.position.z,
                epsilon = EPSILON
            );
        }
    }

    #[test]
    fn test_net_downward_slope() {
        let track = Track::generate(456, 15.0);
        assert!(
            track.current_end.position.y < 0.0,
            "Track should end below starting height"
        );

        let total_height_change =
            track.current_end.position.y - track.segments.first().unwrap().transform.position.y;
        assert!(
            total_height_change < -5.0,
            "Significant downward slope required. Actual: {}",
            total_height_change
        );
    }

    #[test]
    fn no_overlapping_positions() {
        let track = Track::generate(789, 12.0);
        let mut positions = Vec::new();

        for segment in &track.segments {
            positions.push(segment.transform.position);
        }
        positions.push(track.current_end.position);

        // Check all pairwise combinations
        for i in 0..positions.len() {
            for j in (i + 2)..positions.len() {
                let distance = positions[i].distance(positions[j]);
                assert!(
                    distance > 3.0,
                    "Potential overlap between positions {} and {} (distance: {})",
                    i,
                    j,
                    distance
                );
            }
        }
    }

    #[test]
    fn valid_rotations() {
        let track = Track::generate(321, 8.0);

        for segment in &track.segments {
            // Check rotation is normalized
            assert_relative_eq!(segment.transform.rotation.length(), 1.0, epsilon = EPSILON);

            // Check pitch doesn't exceed 45 degrees
            let (yaw, pitch, roll) = segment.transform.rotation.to_euler(EulerRot::YXZ);
            assert!(
                pitch.abs() < FRAC_PI_2 / 2.0,
                "Excessive pitch angle: {} radians",
                pitch
            );
        }
    }
}

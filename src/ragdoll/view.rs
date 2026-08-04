use bevy::prelude::*;

use super::components::RagdollPart;

pub fn update_ragdoll_colors(
    mut ragdoll_parts: Query<(&RagdollPart, &mut Sprite)>,
) {
    for (part, mut sprite) in ragdoll_parts.iter_mut() {
        sprite.color = part.part_type.color(10);
    }
}

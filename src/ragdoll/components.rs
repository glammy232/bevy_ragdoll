use bevy::prelude::*;

#[derive(Component)]
pub struct RagdollPart {
    pub part_type: PartType,
    pub health: u32,
    pub max_health: u32,
    pub collision_cooldown: Timer,
}

#[derive(Resource)]
pub struct RagdollSpawnTimer(pub Timer);

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PartType {
    Head,
    Torso,
    UpperArmLeft,
    LowerArmLeft,
    UpperArmRight,
    LowerArmRight,
    UpperLegLeft,
    LowerLegLeft,
    UpperLegRight,
    LowerLegRight,
}

impl PartType {
    pub fn size(&self) -> Vec2 {
        match self {
            PartType::Head => Vec2::new(30.0, 30.0),
            PartType::Torso => Vec2::new(40.0, 60.0),
            PartType::UpperArmLeft | PartType::UpperArmRight => Vec2::new(15.0, 40.0),
            PartType::LowerArmLeft | PartType::LowerArmRight => Vec2::new(12.0, 35.0),
            PartType::UpperLegLeft | PartType::UpperLegRight => Vec2::new(20.0, 45.0),
            PartType::LowerLegLeft | PartType::LowerLegRight => Vec2::new(18.0, 40.0),
        }
    }

    pub fn mass(&self) -> f32 {
        match self {
            PartType::Head => 1.0,
            PartType::Torso => 3.0,
            PartType::UpperArmLeft | PartType::UpperArmRight => 1.5,
            PartType::LowerArmLeft | PartType::LowerArmRight => 1.0,
            PartType::UpperLegLeft | PartType::UpperLegRight => 2.0,
            PartType::LowerLegLeft | PartType::LowerLegRight => 1.5,
        }
    }

    pub fn color(&self, health: u32) -> Color {
        let base_color = match self {
            PartType::Head => Color::srgb(1.0, 0.85, 0.7),
            PartType::Torso => Color::srgb(0.2, 0.4, 0.8),
            PartType::UpperArmLeft | PartType::UpperArmRight => Color::srgb(0.9, 0.3, 0.3),
            PartType::LowerArmLeft | PartType::LowerArmRight => Color::srgb(1.0, 0.6, 0.3),
            PartType::UpperLegLeft | PartType::UpperLegRight => Color::srgb(0.2, 0.7, 0.3),
            PartType::LowerLegLeft | PartType::LowerLegRight => Color::srgb(0.3, 0.8, 0.4),
        };
        
        let damage_factor = 10 - health;
        let base_rgba = base_color.to_srgba();
        Color::srgb(
            1.0, 1.0, 1.0,
           // (base_rgba.red + damage_factor * 0.5).min(1.0),
           // (base_rgba.green - damage_factor * 0.5).max(0.0),
           // (base_rgba.blue - damage_factor * 0.5).max(0.0),
        )
    }
}

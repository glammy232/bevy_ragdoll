use bevy::prelude::*;

use bevy_rapier2d::prelude::*;

use super::components::RagdollPart;
use super::components::PartType;

pub fn spawn_ragdoll(commands: &mut Commands, position: Vec2, scale: f32, mut time: ResMut<Time<Virtual>>) -> Entity {
    let torso = spawn_part(commands, PartType::Torso, position, 0.0, scale);

    let head = spawn_part(
        commands,
        PartType::Head,
        position + Vec2::new(0.0, 50.0 * scale),
        0.0,
        scale,
    );
    
    create_joint(commands, torso, head,
        Vec2::new(0.0, 25.0 * scale),
        Vec2::new(0.0, -10.0 * scale),
        -0.5, 0.5);

    /*let left_upper = spawn_part(
        commands,
        PartType::UpperArmLeft,
        position + Vec2::new(27.0 * scale, 15.0 * scale),
        -0.3,
        scale,
    );
    
    let left_lower = spawn_part(
        commands,
        PartType::LowerArmLeft,
        position + Vec2::new(27.0 * scale, -25.0 * scale),
        -0.3,
        scale,
    );
    
    create_joint(commands, torso, left_upper,
        Vec2::new(20.0 * scale, 20.0 * scale),
        Vec2::new(0.0, 20.0 * scale),
        -2.0, 1.0);
    
    create_joint(commands, left_upper, left_lower,
        Vec2::new(0.0, -20.0 * scale),
        Vec2::new(0.0, 17.5 * scale),
        -2.5, 0.1);*/

    let right_upper = spawn_part(
        commands,
        PartType::UpperArmRight,
        position + Vec2::new(-27.0 * scale, 15.0 * scale),
        0.3,
        scale,
    );
    
    let right_lower = spawn_part(
        commands,
        PartType::LowerArmRight,
        position + Vec2::new(-27.0 * scale, -25.0 * scale),
        0.3,
        scale,
    );
    
    create_joint(commands, torso, right_upper,
        Vec2::new(-20.0 * scale, 20.0 * scale),
        Vec2::new(0.0, 20.0 * scale),
        -1.0, 2.0);
    
    create_joint(commands, right_upper, right_lower,
        Vec2::new(0.0, -20.0 * scale),
        Vec2::new(0.0, 17.5 * scale),
        -0.1, 2.5);

    /*let left_upper_leg = spawn_part(
        commands,
        PartType::UpperLegLeft,
        position + Vec2::new(12.0 * scale, -55.0 * scale),
        -0.1,
        scale,
    );
    
    let left_lower_leg = spawn_part(
        commands,
        PartType::LowerLegLeft,
        position + Vec2::new(12.0 * scale, -100.0 * scale),
        -0.1,
        scale,
    );
    
    create_joint(commands, torso, left_upper_leg,
        Vec2::new(15.0 * scale, -30.0 * scale),
        Vec2::new(0.0, 22.5 * scale),
        -1.5, 0.5);
    
    create_joint(commands, left_upper_leg, left_lower_leg,
        Vec2::new(0.0, -22.5 * scale),
        Vec2::new(0.0, 20.0 * scale),
        -0.1, 1.5);*/

    let right_upper_leg = spawn_part(
        commands,
        PartType::UpperLegRight,
        position + Vec2::new(-12.0 * scale, -55.0 * scale),
        0.1,
        scale,
    );
    
    let right_lower_leg = spawn_part(
        commands,
        PartType::LowerLegRight,
        position + Vec2::new(-12.0 * scale, -100.0 * scale),
        0.1,
        scale,
    );
    
    create_joint(commands, torso, right_upper_leg,
        Vec2::new(-15.0 * scale, -30.0 * scale),
        Vec2::new(0.0, 22.5 * scale),
        -0.5, 1.5);
    
    create_joint(commands, right_upper_leg, right_lower_leg,
        Vec2::new(0.0, -22.5 * scale),
        Vec2::new(0.0, 20.0 * scale),
        -1.5, 0.1);

    time.pause();
    torso
}

pub fn spawn_part(
    commands: &mut Commands,
    part_type: PartType,
    position: Vec2,
    rotation: f32,
    scale: f32,
) -> Entity {
    let size = part_type.size() * scale;
    let color = part_type.color(1);

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(position.extend(0.0))
            .with_rotation(Quat::from_rotation_z(rotation)),
        RigidBody::Dynamic,
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        ColliderMassProperties::Mass(part_type.mass() * scale),
        Damping {
            linear_damping: 0.05,//0.1
            angular_damping: 0.1,//0.2
        },
        Restitution::coefficient(0.1),//0.3
        Friction::coefficient(0.3),//0.5
        RagdollPart {
            part_type,
            health: 10,
            max_health: 10,
            collision_cooldown: Timer::from_seconds(0.5, TimerMode::Once),
        },
        ExternalForce::default(),
        Velocity::default(),
    )).id()
}

fn create_joint(
    commands: &mut Commands,
    parent: Entity,
    child: Entity,
    anchor1: Vec2,
    anchor2: Vec2,
    min_angle: f32,
    max_angle: f32,
) {
    let joint = RevoluteJointBuilder::new()
        .local_anchor1(anchor1)
        .local_anchor2(anchor2)
        .limits([min_angle, max_angle]);
    
    commands.entity(child).insert(ImpulseJoint::new(parent, joint));
}

/*fn spawn_random_ragdoll(
    time: Res<Time>,
    mut timer: ResMut<RagdollSpawnTimer>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    timer.0.tick(time.delta());
    
    if keyboard.just_pressed(KeyCode::KeyR) || timer.0.just_finished() {
        let x = rand::random::<f32>() * 600.0 - 300.0;
        let y = rand::random::<f32>() * 200.0 + 100.0;
        let scale = rand::random::<f32>() * 1.0 + 0.5;
        
        spawn_ragdoll(&mut commands, Vec2::new(x, y), scale);
    }
}*/

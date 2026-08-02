use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy::window::WindowResolution;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ragdoll Playground 2D".into(),
                    resolution: WindowResolution::new(1280, 720),
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(RagdollSpawnTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            spawn_random_ragdoll,
            control_ragdoll,
            apply_explosion,
            handle_collisions,
            update_ragdoll_colors,
            update_particles,
        ))
        .run();
}

// Ресурсы
#[derive(Resource)]
struct RagdollSpawnTimer(Timer);

// Компоненты
#[derive(Component)]
struct RagdollPart {
    part_type: PartType,
    health: f32,
    max_health: f32,
    collision_cooldown: Timer,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum PartType {
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
    fn size(&self) -> Vec2 {
        match self {
            PartType::Head => Vec2::new(30.0, 30.0),
            PartType::Torso => Vec2::new(40.0, 60.0),
            PartType::UpperArmLeft | PartType::UpperArmRight => Vec2::new(15.0, 40.0),
            PartType::LowerArmLeft | PartType::LowerArmRight => Vec2::new(12.0, 35.0),
            PartType::UpperLegLeft | PartType::UpperLegRight => Vec2::new(20.0, 45.0),
            PartType::LowerLegLeft | PartType::LowerLegRight => Vec2::new(18.0, 40.0),
        }
    }

    fn mass(&self) -> f32 {
        match self {
            PartType::Head => 1.0,
            PartType::Torso => 3.0,
            PartType::UpperArmLeft | PartType::UpperArmRight => 1.5,
            PartType::LowerArmLeft | PartType::LowerArmRight => 1.0,
            PartType::UpperLegLeft | PartType::UpperLegRight => 2.0,
            PartType::LowerLegLeft | PartType::LowerLegRight => 1.5,
        }
    }

    fn color(&self, health: f32) -> Color {
        let base_color = match self {
            PartType::Head => Color::srgb(1.0, 0.85, 0.7),
            PartType::Torso => Color::srgb(0.2, 0.4, 0.8),
            PartType::UpperArmLeft | PartType::UpperArmRight => Color::srgb(0.9, 0.3, 0.3),
            PartType::LowerArmLeft | PartType::LowerArmRight => Color::srgb(1.0, 0.6, 0.3),
            PartType::UpperLegLeft | PartType::UpperLegRight => Color::srgb(0.2, 0.7, 0.3),
            PartType::LowerLegLeft | PartType::LowerLegRight => Color::srgb(0.3, 0.8, 0.4),
        };
        
        let damage_factor = 1.0 - health;
        let base_rgba = base_color.to_srgba();
        Color::srgb(
            (base_rgba.red + damage_factor * 0.5).min(1.0),
            (base_rgba.green - damage_factor * 0.5).max(0.0),
            (base_rgba.blue - damage_factor * 0.5).max(0.0),
        )
    }
}

#[derive(Component)]
struct Particle {
    lifetime: Timer,
    velocity: Vec2,
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);

    // ground
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(1280.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -300.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(640.0, 10.0),
    ));

    // left wall
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(20.0, 720.0)),
            ..default()
        },
        Transform::from_xyz(-640.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(10.0, 360.0),
    ));

    // right wall
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(20.0, 720.0)),
            ..default()
        },
        Transform::from_xyz(640.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(10.0, 360.0),
    ));

    spawn_ragdoll(&mut commands, Vec2::new(0.0, 200.0), 1.0);
}

fn spawn_ragdoll(commands: &mut Commands, position: Vec2, scale: f32) {
    let torso = spawn_part(commands, PartType::Torso, position, 0.0, scale);

    let head = spawn_part(
        commands,
        PartType::Head,
        position + Vec2::new(0.0, 50.0 * scale),
        0.0,
        scale,
    );
    
    create_joint(commands, torso, head,
        Vec2::new(0.0, 30.0 * scale),
        Vec2::new(0.0, -15.0 * scale),
        -0.5, 0.5);

    let left_upper = spawn_part(
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
        -2.5, 0.1);

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

    let left_upper_leg = spawn_part(
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
        -0.1, 1.5);

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
}

fn spawn_part(
    commands: &mut Commands,
    part_type: PartType,
    position: Vec2,
    rotation: f32,
    scale: f32,
) -> Entity {
    let size = part_type.size() * scale;
    let color = part_type.color(1.0);

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
            linear_damping: 0.1,
            angular_damping: 0.2,
        },
        Restitution::coefficient(0.3),
        Friction::coefficient(0.5),
        RagdollPart {
            part_type,
            health: 1.0,
            max_health: 1.0,
            collision_cooldown: Timer::from_seconds(0.1, TimerMode::Once),
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

fn control_ragdoll(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut ragdoll_parts: Query<(&mut ExternalForce, &Transform), With<RagdollPart>>,
) {
    let move_force = 50000.0;
    
    for (mut force, _) in ragdoll_parts.iter_mut() {
        let mut direction = Vec2::ZERO;
        
        if keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        
        force.force = direction.normalize_or_zero() * move_force;
    }

    // Click to punch
    if mouse.just_pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera.single() {
            if let Ok(window) = windows.single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                        for (mut force, transform) in ragdoll_parts.iter_mut() {
                            let distance = transform.translation.truncate() - world_pos;
                            let force_magnitude = 100000.0 / (1.0 + distance.length());
                            
                            if distance.length() < 100.0 {
                                let direction = distance.normalize_or_zero();
                                force.force = direction * force_magnitude;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn apply_explosion(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut ragdoll_parts: Query<(&mut ExternalForce, &Transform), With<RagdollPart>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let mut center = Vec2::ZERO;
        let mut count = 0;
        
        for (_, transform) in ragdoll_parts.iter() {
            center += transform.translation.truncate();
            count += 1;
        }
        
        if count > 0 {
            center /= count as f32;
            
            for (mut force, transform) in ragdoll_parts.iter_mut() {
                let direction = (transform.translation.truncate() - center).normalize_or_zero();
                let distance = (transform.translation.truncate() - center).length();
                let explosion_force = 5000.0 / (1.0 + distance * 0.1);
                force.force = direction * explosion_force;
            }
            
            for _ in 0..20 {
                let velocity = Vec2::new(
                    rand::random::<f32>() * 400.0 - 200.0,
                    rand::random::<f32>() * 400.0 - 200.0,
                );
                
                commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, rand::random::<f32>() * 0.5 + 0.3, 0.0),
                        custom_size: Some(Vec2::new(5.0, 5.0)),
                        ..default()
                    },
                    Transform::from_translation(center.extend(0.0)),
                    Particle {
                        lifetime: Timer::from_seconds(rand::random::<f32>() * 0.5 + 0.3, TimerMode::Once),
                        velocity,
                    },
                ));
            }
        }
    }
}

fn handle_collisions(
    mut commands: Commands,
    time: Res<Time>,
    mut collision_events: MessageReader<CollisionEvent>,
    mut ragdoll_parts: Query<(Entity, &mut RagdollPart)>,
) {
    let mut processed_pairs: Vec<(Entity, Entity)> = Vec::new();

    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _flags) = collision_event {
            let pair = if *entity1 < *entity2 {
                (*entity1, *entity2)
            } else {
                (*entity2, *entity1)
            };

            if !processed_pairs.contains(&pair) {
                processed_pairs.push(pair);

                for &entity in &[pair.0, pair.1] {
                    if let Ok((ragdoll_entity, mut part)) = ragdoll_parts.get_mut(entity) {
                        part.collision_cooldown.tick(time.delta());
                        
                        if part.collision_cooldown.is_finished() {
                            part.health -= 0.01;
                            part.collision_cooldown.reset();
                            
                            if part.health <= 0.0 {
                                commands.entity(ragdoll_entity).remove::<ImpulseJoint>();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_ragdoll_colors(
    mut ragdoll_parts: Query<(&RagdollPart, &mut Sprite)>,
) {
    for (part, mut sprite) in ragdoll_parts.iter_mut() {
        sprite.color = part.part_type.color(part.health);
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut Particle)>,
) {
    for (entity, mut transform, mut particle) in particles.iter_mut() {
        particle.lifetime.tick(time.delta());
        
        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity.extend(0.0) * time.delta_secs();
            particle.velocity *= 0.95;
            transform.scale *= 0.98;
        }
    }
}

fn spawn_random_ragdoll(
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
}

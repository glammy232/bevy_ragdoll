mod ragdoll;

use bevy::input::keyboard::keyboard_input_system;
use bevy::prelude::*;
use bevy::window::WindowEvent::KeyboardInput;
use bevy_rapier2d::prelude::*;
use bevy::window::WindowResolution;

use ragdoll::RagdollPlugin;
use ragdoll::components::RagdollSpawnTimer;
use ragdoll::functions::spawn_ragdoll;
use ragdoll::components::RagdollPart;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#bevy-canvas".to_string()),
                    fit_canvas_to_parent: true,
                    title: "Ragdoll Playground 2D".into(),
                    resolution: WindowResolution::new(1280, 720),
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            //RapierDebugRenderPlugin::default(),
        ))
        .add_plugins(RagdollPlugin)
        .insert_resource(RagdollSpawnTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            //spawn_random_ragdoll,
            control_ragdoll,
            //apply_explosion,
            //handle_collisions,
            //update_ragdoll_colors,
            update_particles,
            set_time,
        ))
        .run();
}

#[derive(Component)]
struct PlayerControlled;

#[derive(Event)]
struct ExplosionEvent {
    center: Vec2,
    force: f32,
    radius: f32,
}

#[derive(Component)]
struct Particle {
    lifetime: Timer,
    velocity: Vec2,
}

fn setup(
    mut commands: Commands,
    mut time: ResMut<Time<Virtual>>
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(1320.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -320.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(660.0, 10.0),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(20.0, 1000.0)),
            ..default()
        },
        Transform::from_xyz(-640.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(10.0, 500.0),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(20.0, 1000.0)),
            ..default()
        },
        Transform::from_xyz(640.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(10.0, 500.0),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(150.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(-555.0, 150.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(25.0, 20.0),
    ));

    spawn_ladder(&mut commands);

    let torso = spawn_ragdoll(&mut commands, Vec2::new(-445.0, 450.0), 1.0, time);
    
    commands.entity(torso).insert(PlayerControlled);
}

fn spawn_ladder(commands: &mut Commands) {
    for i in 0..12 {
        commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(50.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(-505.0 + i as f32 * 50.0, 150.0 - i as f32 * 40.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(25.0, 20.0),
        ));
    }
}

fn set_time(keyboard: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Virtual>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

fn spawn_explosion_particles(commands: &mut Commands, center: Vec2, radius: f32) {
    let num_particles = 30;
    let colors = [
        Color::srgb(1.0, 0.8, 0.0),  // Yello
        Color::srgb(1.0, 0.5, 0.0),  // Orange
        Color::srgb(1.0, 0.3, 0.0),  // Red
        Color::srgb(0.8, 0.2, 0.0),  // Dark-Red
    ];
    
    for _ in 0..num_particles {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let speed = rand::random::<f32>() * 300.0 + 100.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        let size = rand::random::<f32>() * 4.0 + 2.0;
        let lifetime = rand::random::<f32>() * 0.8 + 0.3;
        let color = colors[0];
        
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_translation(center.extend(0.0)),
            Particle {
                lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
                velocity,
            },
        ));
    }
}

fn control_ragdoll(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut ragdoll_parts: Query<(Entity, &mut Velocity, &Transform, &ColliderMassProperties), With<PlayerControlled>>,
    mut parts_without_control: Query<(Entity, &mut Velocity, &Transform), (With<RagdollPart>, Without<PlayerControlled>)>,
) {
    let control_force = 2000.0;
    let max_velocity = 50000.0;
    
    // Контроль WASD для помеченных частей
    if let Ok((camera, camera_transform)) = camera.single() {
        if let Ok(window) = windows.single() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    // Управление через следование за курсором
                    for (entity, mut velocity, transform, mass_props) in ragdoll_parts.iter_mut() {
                        let target = world_pos;
                        let current = transform.translation.truncate();
                        let direction = (target - current).normalize_or_zero();
                        let distance = (target - current).length();
                        
                        // Плавное ускорение к цели с ограничением скорости
                        let target_velocity = direction * (distance * 5.0).min(max_velocity);
                        let mass = match mass_props {
                            ColliderMassProperties::Mass(m) => *m,
                            ColliderMassProperties::MassProperties(mp) => mp.mass,
                            ColliderMassProperties::Density(_) => {
                                1.0
                            }
                        };
                        
                        let vel = velocity.linear.clone();
                        velocity.linear += (target_velocity - vel) * 0.1 * mass;
                    }
                }
            }
        }
    }
    
    // WASD как дополнительный контроль
    let mut move_dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { move_dir.y += 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { move_dir.y -= 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }
    
    if move_dir != Vec2::ZERO {
        let move_dir = move_dir.normalize();
        for (_, mut velocity, _, _) in ragdoll_parts.iter_mut() {
            velocity.linear += move_dir * control_force * 0.016; // Учитываем время кадра
        }
    }
}

/*fn apply_explosion(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut ragdoll_parts: Query<(Entity, &mut ExternalForce, &Transform, &ColliderMassProperties), With<RagdollPart>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        println!("Explosion");
        let mut center = Vec2::ZERO;
        let mut total_mass = 0.0;
        
        // Center of mass
        for (_, _, transform, mass_props) in ragdoll_parts.iter() {
            let mass = match mass_props {
                    ColliderMassProperties::Mass(m) => *m,
                    ColliderMassProperties::MassProperties(mp) => mp.mass,
                    ColliderMassProperties::Density(_) => {
                    1.0 
                }
            };
            center += transform.translation.truncate() * mass;
            total_mass += mass;
        }
        
        if total_mass > 0.0 {
            center /= total_mass;
            
            let explosion_radius = 2000.0;
            let max_force = 100000.0;
            
            for (entity, mut force, transform, mass_props) in ragdoll_parts.iter_mut() {
                let direction = (transform.translation.truncate() - center).normalize_or_zero();
                let distance = (transform.translation.truncate() - center).length();
                
                let falloff = (-distance / explosion_radius).exp();

                let mass = match mass_props {
                        ColliderMassProperties::Mass(m) => *m,
                        ColliderMassProperties::MassProperties(mp) => mp.mass,
                        ColliderMassProperties::Density(_) => {
                        1.0 
                    }
                };
                
                let explosion_force = max_force * falloff * mass;
                
                force.force = direction * explosion_force;
            }
            
            spawn_explosion_particles(&mut commands, center, explosion_radius);
        }
    }
}*/

/*fn handle_collisions(
    mut commands: Commands,
    time: Res<Time>,
    mut collision_events: MessageReader<CollisionEvent>,
    mut ragdoll_parts: Query<(Entity, &mut RagdollPart, &Velocity)>,
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
                
                let vel1 = ragdoll_parts.get(pair.0).map(|(_, _, v)| v.linear.length()).unwrap_or(0.0);
                let vel2 = ragdoll_parts.get(pair.1).map(|(_, _, v)| v.linear.length()).unwrap_or(0.0);
                let impact_force = (vel1 - vel2).abs();
                
                for &entity in &[pair.0, pair.1] {
                    if let Ok((ragdoll_entity, mut part, _)) = ragdoll_parts.get_mut(entity) {
                        part.collision_cooldown.tick(time.delta());
                        
                        if part.collision_cooldown.is_finished() {
                            let damage = (impact_force * 0.001).min(0.1);
                            part.health -= damage;
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
}*/

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

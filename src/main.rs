// Why on earth does everything rotate when physics is enabled

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_rapier3d::{
    dynamics::{ImpulseJoint, RevoluteJointBuilder, RigidBody},
    geometry::{Collider, CollisionGroups, Group},
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(
            Startup,
            (
                setup_scene,
                setup_camera,
                setup_bike,
                setup_physics,
                setup_ui,
            ),
        )
        .add_systems(Update, (pause_physics, pan_camera, enable_gravity))
        .run();
}

fn setup_scene(mut commands: Commands) {
    // ground
    commands.spawn((
        TransformBundle::from_transform(Transform::from_xyz(0., -1.0, 0.)),
        RigidBody::Fixed,
        Collider::cylinder(0.05, 5.0),
        CollisionGroups::new(
            Group::GROUP_1,
            Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_3,
        ),
    ));
}

fn setup_bike(mut commands: Commands) {
    let _bike = commands
        .spawn((
            Name::new("Bike"),
            TransformBundle::default(),
            RigidBody::Dynamic,
            Collider::cuboid(0.5, 0.5, 2.0),
            CollisionGroups::new(Group::GROUP_2, Group::GROUP_1),
        ))
        .with_children(|parent| {
            let parent_entity = parent.parent_entity();

            let joint = RevoluteJointBuilder::new(Vec3::X)
                .local_anchor1(Vec3::ZERO)
                .local_anchor2(Vec3::ZERO);

            parent.spawn((
                Name::new("FrontWheelCollider"),
                RigidBody::Dynamic,
                TransformBundle::from_transform(Transform::from_rotation(Quat::from_rotation_z(
                    std::f32::consts::FRAC_PI_2,
                ))),
                Collider::round_cylinder(0.025, 0.8, 0.025),
                CollisionGroups::new(Group::GROUP_3, Group::GROUP_1),
                ImpulseJoint::new(parent_entity, joint),
            ));
        })
        .id();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera3dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(3.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(2.0, 0.0, 0.0).looking_at(Vec3::new(0., 0.0, 0.), Vec3::Y),
        ..default()
    },));
}

fn setup_physics(mut rapier: ResMut<RapierConfiguration>) {
    rapier.physics_pipeline_active = false;
    rapier.gravity = Vec3::ZERO;
}

fn setup_ui(mut commands: Commands) {
    commands.spawn(
        TextBundle::from_section(
            "Press p to toggle physics\nPress g to toggle gravity",
            TextStyle::default(),
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

fn pause_physics(mut rapier: ResMut<RapierConfiguration>, buttons: Res<ButtonInput<KeyCode>>) {
    if buttons.just_pressed(KeyCode::KeyP) {
        rapier.physics_pipeline_active = !rapier.physics_pipeline_active;
    }
}

fn enable_gravity(mut rapier: ResMut<RapierConfiguration>, buttons: Res<ButtonInput<KeyCode>>) {
    if buttons.just_pressed(KeyCode::KeyG) {
        rapier.gravity = if rapier.gravity == Vec3::ZERO {
            Vec3::Y * -9.81
        } else {
            Vec3::ZERO
        };
    }
}

fn pan_camera(
    mut query: Query<&mut Transform, With<Camera>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let x = keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) as i8
        - keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) as i8;
    let y = keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) as i8
        - keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) as i8;
    let dir = Vec2::new(x as f32, y as f32).normalize_or_zero();

    if dir == Vec2::ZERO {
        return;
    }

    for mut transform in &mut query {
        let three = dir.extend(0.);
        let rotated = transform.rotation.mul_vec3(three);
        transform.translation += rotated * time.delta_seconds();
    }
}

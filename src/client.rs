use bevy::{
    core::FixedTimestep,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use rand::{self, Rng};
use std::collections::BTreeMap;

use rblock::score_client::ScoreClient;
use rblock::ScoreRequest;
use tonic::Request;

pub mod rblock {
    tonic::include_proto!("rblock");
}

struct ScoreBoard {
    score: usize,
}
#[derive(Component)]
struct Score;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum RunState {
    Start,
    End,
}

struct PauseControl {
    pause: bool,
}

#[derive(Component)]
struct FinishPicture;

#[derive(Component)]
struct BlockAlive {
    velocity: Vec3,
}

struct BlockCenter {
    center: Vec2,
}
#[derive(Component)]
struct BlockDead;
#[derive(Component)]
struct BlockNext;
#[derive(Component)]
struct BlockWall;

const NEXT_CENTER: (f32, f32) = (-425.0, -50.0); //next center(-425.0,-50.0); screen top center(0.0,280.0);//
const CURR_CENTER: (f32, f32) = (0.0, 280.0);
const COL_NUM: usize = 12;
const ROW_NUM: usize = 20;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ScoreBoard { score: 0 })
        .insert_resource(BlockCenter {
            center: Vec2::new(CURR_CENTER.0, CURR_CENTER.1),
        })
        .insert_resource(PauseControl { pause: false })
        .add_startup_system(setup)
        .add_state(RunState::Start)
        .add_system_set(SystemSet::on_exit(RunState::End).with_system(update_block_system))
        .add_system_set(
            SystemSet::on_update(RunState::Start)
                .with_run_criteria(FixedTimestep::step(1.0)) //每秒一次
                .with_system(alive_block_move_system),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0 / 10.0)) //每秒一次
                .with_system(pause_system)
                .with_system(game_over_system),
        )
        .add_system_set(
            SystemSet::on_update(RunState::Start)
                .with_run_criteria(FixedTimestep::step(1.0 / 16.0))
                .with_system(alive_key_move_system),
        )
        .add_system_set(
            SystemSet::on_update(RunState::Start)
                .with_run_criteria(FixedTimestep::step(1.0 / 60.0))
                .with_system(alive_collision_system),
        )
        .add_system_set(SystemSet::on_enter(RunState::End).with_system(dead_block_clear_system))
        .add_system(scoreboard_system)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    //background
    for i in 0..COL_NUM {
        for j in 0..ROW_NUM {
            commands.spawn_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(
                        35.0 * i as f32 - 35.0 * 5.5,
                        -35.0 * 9.5 + 35.0 * j as f32,
                        0.0,
                    ),
                    scale: Vec3::new(30.0, 30.0, 0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.8, 0.8),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }
    // Add walls
    let wall_color = Color::rgb(0.0, 0.0, 0.0);
    let wall_thickness = 5.0;
    let bounds = Vec2::new(35.0 * 12.0, 35.0 * 20.0);

    // left
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-bounds.x / 2.0, 0.0, 0.0),
                scale: Vec3::new(wall_thickness, bounds.y + wall_thickness, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: wall_color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockWall);
    // right
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(bounds.x / 2.0, 0.0, 0.0),
                scale: Vec3::new(wall_thickness, bounds.y + wall_thickness, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: wall_color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockWall);
    // bottom
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -bounds.y / 2.0, 0.0),
                scale: Vec3::new(bounds.x + wall_thickness, wall_thickness, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: wall_color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockWall);
    // top
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, bounds.y / 2.0, 0.0),
                scale: Vec3::new(bounds.x + wall_thickness, wall_thickness, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: wall_color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BlockWall);

    // scoreboard
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Score: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.5, 0.5, 1.0),
                        },
                    },
                    TextSection {
                        value: "0".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(1.0, 0.5, 0.5),
                        },
                    },
                ],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Score);
    //next text
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Next: ".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.5, 0.5, 1.0),
                },
            }],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(255.0), //255
                left: Val::Px(5.0),  //5
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
    //next block
    let block = rand_spawn_block();
    for s in block.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(NEXT_CENTER.0 + s.0, NEXT_CENTER.1 + s.1, 0.0),
                    scale: Vec3::new(30.0, 30.0, 0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.3, 0.7),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockNext);
    }
    let block = rand_spawn_block();
    for s in block.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(CURR_CENTER.0 + s.0, CURR_CENTER.1 + s.1, 0.0),
                    scale: Vec3::new(30.0, 30.0, 0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.5, 0.5),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockAlive {
                velocity: Vec3::new(35.0, 35.0, 0.0),
            });
    }
}

fn rand_spawn_block() -> Vec<(f32, f32)> {
    //围绕旋转中心构造5种基本图案
    let blocks: Vec<Vec<(f32, f32)>> = vec![
        vec![(17.5, 17.5), (-17.5, 17.5), (17.5, 52.5), (-17.5, -17.5)], //N
        vec![(17.5, 17.5), (-17.5, 17.5), (17.5, -17.5), (-17.5, 52.5)], //N
        vec![(17.5, 17.5), (-17.5, 17.5), (-17.5, -17.5), (-17.5, 52.5)], //T
        vec![(-17.5, 17.5), (-17.5, -17.5), (17.5, -17.5), (17.5, 17.5)], //O
        vec![(17.5, -17.5), (-17.5, 52.5), (-17.5, -17.5), (-17.5, 17.5)], //L
        vec![(-17.5, -17.5), (17.5, 52.5), (17.5, -17.5), (17.5, 17.5)], //L
        vec![(17.5, 17.5), (17.5, 52.5), (17.5, -17.5), (17.5, -52.5)],  //I
    ];
    let mut rng = rand::thread_rng();
    blocks[rng.gen_range(0, 7)].clone()
}

fn update_block_system(
    mut commands: Commands,
    pause: Res<PauseControl>,
    mut center: ResMut<BlockCenter>,
    next: Query<(Entity, &BlockNext, &Transform)>,
) {
    //随机生成元素块到next区，取原next块到屏幕顶端
    //读取old next，生成curr
    if pause.pause {
        return;
    }

    for (entity, _, transform) in next.iter() {
        let offset = transform.translation;
        commands
            .spawn_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(
                        CURR_CENTER.0 + offset[0] - NEXT_CENTER.0,
                        CURR_CENTER.1 + offset[1] - NEXT_CENTER.1,
                        0.0,
                    ),
                    scale: Vec3::new(30.0, 30.0, 0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.5, 0.5),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockAlive {
                velocity: Vec3::new(35.0, 35.0, 0.0),
            });
        commands.entity(entity).despawn();
    }
    center.center = Vec2::new(CURR_CENTER.0, CURR_CENTER.1);

    let block = rand_spawn_block();
    for s in block.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(NEXT_CENTER.0 + s.0, NEXT_CENTER.1 + s.1, 0.0),
                    scale: Vec3::new(30.0, 30.0, 0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.3, 0.7),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockNext);
    }
}

fn pause_system(key_input: ResMut<Input<KeyCode>>, mut state: ResMut<PauseControl>) {
    if key_input.pressed(KeyCode::Space) {
        if state.pause {
            state.pause = false;
        } else {
            state.pause = true;
        }
    }
}

//auto
fn alive_block_move_system(
    pause: Res<PauseControl>,
    mut center: ResMut<BlockCenter>,
    mut alive: Query<(&BlockAlive, &mut Transform)>,
) {
    if pause.pause {
        return;
    }
    let mut velocity = None;
    for (block, mut transform) in alive.iter_mut() {
        transform.translation.y -= block.velocity.y;
        velocity = Some(block.velocity);
    }
    if let Some(velocity) = velocity {
        center.center.y -= velocity.y;
    }
}
fn alive_key_move_system(
    key_input: Res<Input<KeyCode>>,
    pause: Res<PauseControl>,
    mut center: ResMut<BlockCenter>,
    mut alive: Query<(&BlockAlive, &mut Transform)>,
) {
    if pause.pause {
        return;
    }

    let mut velocity = None;
    for (block, transform) in alive.iter_mut() {
        if (transform.translation.x - block.velocity.x < -35.0 * 5.5
            && key_input.pressed(KeyCode::Left))
            || (transform.translation.x + block.velocity.x
                > 35.0 * (COL_NUM - 1) as f32 - 35.0 * 5.5
                && key_input.pressed(KeyCode::Right))
        {
            return;
        }
    }
    for (block, mut transform) in alive.iter_mut() {
        velocity = Some(block.velocity);
        if key_input.pressed(KeyCode::Left) {
            transform.translation.x -= block.velocity.x;
        } else if key_input.pressed(KeyCode::Right) {
            transform.translation.x += block.velocity.x;
        } else if key_input.pressed(KeyCode::Down) {
            transform.translation.y -= block.velocity.y;
        } else if key_input.pressed(KeyCode::Up) {
            //旋转操作
            let x1 = transform.translation.x;
            let y1 = transform.translation.y;
            let x2 = center.center.x;
            let y2 = center.center.y;
            transform.translation.x = -(y1 - y2) + x2;
            transform.translation.y = (x1 - x2) + y2;
        }
    } //最左、最右侧不碰壁，操作栈记录所有信息，失败后回滚
    if let Some(velocity) = velocity {
        if key_input.pressed(KeyCode::Left) {
            center.center.x -= velocity.x;
        } else if key_input.pressed(KeyCode::Right) {
            center.center.x += velocity.x;
        } else if key_input.pressed(KeyCode::Down) {
            center.center.y -= velocity.y;
        }
    }
}

fn alive_collision_system(
    mut commands: Commands,
    mut app_state: ResMut<State<RunState>>,
    alive: Query<(Entity, &BlockAlive, &Transform)>,
    dead: Query<(&BlockDead, &Transform)>,
    wall: Query<(&BlockWall, &Transform)>,
) {
    let mut is_collided = false;
    'first_for: for (_, _, alive_transform) in alive.iter() {
        for (_, dead_transform) in dead.iter() {
            let collision = collide(
                alive_transform.translation,
                alive_transform.scale.truncate() + Vec2::new(2.5, 5.5),
                dead_transform.translation,
                dead_transform.scale.truncate() + Vec2::new(2.5, 5.5),
            );
            if let Some(collision) = collision {
                match collision {
                    Collision::Top => {
                        is_collided = true;
                        break 'first_for;
                    }
                    _ => (),
                }
            }
        }
        for (_, wall_transform) in wall.iter() {
            let collision = collide(
                alive_transform.translation,
                alive_transform.scale.truncate() + Vec2::new(2.5, 2.5),
                wall_transform.translation,
                wall_transform.scale.truncate() + Vec2::new(2.5, 2.5),
            );
            if let Some(collision) = collision {
                match collision {
                    Collision::Top => {
                        is_collided = true;
                        break 'first_for;
                    }
                    _ => (),
                }
            }
        }
    }
    if is_collided {
        for (entity, _, alive_transform) in alive.iter() {
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform {
                        translation: alive_transform.translation,
                        scale: Vec3::new(30.0, 30.0, 0.0),
                        ..Default::default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.5, 0.7, 0.2),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(BlockDead);
            commands.entity(entity).despawn();
        }
        app_state.set(RunState::End).unwrap();
    }
}

fn dead_block_clear_system(
    mut commands: Commands,
    mut app_state: ResMut<State<RunState>>,
    mut board: ResMut<ScoreBoard>,
    mut dead: Query<(Entity, &BlockDead, &mut Transform)>,
) {
    let mut counter = BTreeMap::new();
    for (_, _, transform) in dead.iter() {
        let y = transform.translation.y as i32;
        if counter.contains_key(&y) {
            *counter.get_mut(&y).unwrap() += 1;
        } else {
            counter.insert(y, 1);
        }
    }
    let mut dead_entity = Vec::new();
    for (k, v) in counter.iter() {
        if *v != COL_NUM {
            continue;
        }
        for (entity, _, mut transform) in dead.iter_mut() {
            let y = transform.translation.y as i32;
            if y > *k {
                //coord add
                transform.translation.y -= 35.0;
            } else if y == *k {
                //delete entity
                dead_entity.push(entity);
            }
        }
    }
    board.score += dead_entity.len() * dead_entity.len() / COL_NUM / COL_NUM;
    for e in dead_entity.iter() {
        commands.entity(*e).despawn();
    }
    app_state.set(RunState::Start).unwrap();
}

fn scoreboard_system(scoreboard: Res<ScoreBoard>, mut query: Query<(&Score, &mut Text)>) {
    let (_, mut text) = query.single_mut();

    let request = Request::new(ScoreRequest {
        score: scoreboard.score as u32,
        topk: 10,
    });
    let rt=tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async{
        let mut client = ScoreClient::connect("http://127.0.0.1:8020").await.unwrap();
         let response = client.query_score(request).await.unwrap();
         text.sections[1].value = format!("{} rank:{}", scoreboard.score,response.into_inner().rank);
    });
}

fn game_over_system(
    key_input: Res<Input<KeyCode>>,
    mut scoreboard: ResMut<ScoreBoard>,
    mut pause: ResMut<PauseControl>,
    mut commands: Commands,
    mut center: ResMut<BlockCenter>,
    asset_server: Res<AssetServer>,
    alive: Query<(Entity, &BlockAlive)>,
    over_img: Query<(Entity, &Text, &FinishPicture)>,
    dead: Query<(Entity, &BlockDead, &Transform)>,
) {
    let mut finish = false;
    for (_, _, transform) in dead.iter() {
        if transform.translation.y >= -35.0 * 9.5 + 35.0 * (ROW_NUM - 2) as f32 {
            finish = true;
            break;
        }
    }
    if finish {
        commands
            .spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "GameOver!".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 80.0,
                            color: Color::rgb(0.5, 0.5, 1.0),
                        },
                    }],
                    ..Default::default()
                },
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(255.0),
                        left: Val::Px(555.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(FinishPicture);
        pause.pause = true;
    }
    if key_input.pressed(KeyCode::Return) {
        for (entity, _, _) in dead.iter() {
            commands.entity(entity).despawn();
        }
        for (entity, _) in alive.iter() {
            commands.entity(entity).despawn();
        }
        let block = rand_spawn_block();
        for s in block.iter() {
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(CURR_CENTER.0 + s.0, CURR_CENTER.1 + s.1, 0.0),
                        scale: Vec3::new(30.0, 30.0, 0.0),
                        ..Default::default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.5, 0.5, 0.5),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(BlockAlive {
                    velocity: Vec3::new(35.0, 35.0, 0.0),
                });
        }
        pause.pause = false;
        scoreboard.score = 0;
        center.center.x = CURR_CENTER.0;
        center.center.y = CURR_CENTER.1;
        for (entity, _, _) in over_img.iter() {
            commands.entity(entity).despawn();
        }
    }
}

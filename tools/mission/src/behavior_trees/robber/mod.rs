use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

use crate::components::robber::Robber;

pub mod robber_bribe;
pub mod robber_captured;
pub mod robber_rest;
pub mod robber_run;

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "99bd66e1-5e2e-40fb-a639-5fd56667b752"]
pub enum RobberBehavior {
    Debug(Debug),
    Delay(Delay),
    Selector(Selector),
    Sequencer(Sequencer),
    Repeater(Repeater),
    Inverter(Inverter),
    Any(Any),
    All(All),
    RobberRun(robber_run::RobberRunAction),
    RobberBribe(robber_bribe::RobberBribeAction),
    RobberCaptured(robber_captured::RobberCapturedAction),
    RobberRest(robber_rest::RobberRestAction),
}

impl Default for RobberBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorSpawner for RobberBehavior {
    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            RobberBehavior::Debug(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::Delay(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::Selector(selector) => BehaviorInfo::insert_with(commands, selector),
            RobberBehavior::Sequencer(sequence) => BehaviorInfo::insert_with(commands, sequence),
            RobberBehavior::Repeater(repeater) => BehaviorInfo::insert_with(commands, repeater),
            RobberBehavior::Inverter(inverter) => BehaviorInfo::insert_with(commands, inverter),
            RobberBehavior::Any(any) => BehaviorInfo::insert_with(commands, any),
            RobberBehavior::All(all) => BehaviorInfo::insert_with(commands, all),
            RobberBehavior::RobberRun(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::RobberBribe(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::RobberCaptured(action) => BehaviorInfo::insert_with(commands, action),
            RobberBehavior::RobberRest(action) => BehaviorInfo::insert_with(commands, action),
        }
    }
}

pub fn setup_behavior(
    mut commands: Commands,
    query: Query<Entity, (With<Robber>, Without<BehaviorTree>)>,
    asset_server: Res<AssetServer>,
) {
    for robber in query.iter() {
        let document: Handle<BehaviorAsset> = asset_server.load("behaviors/mission/robber.bht.ron");
        let behavior = BehaviorTree::from_asset::<RobberBehavior>(None, &mut commands, document);
        if let Some(root) = behavior.root {
            commands.entity(root).insert(BehaviorCursor);
        }
        commands
            .entity(robber)
            .push_children(&[behavior.root.unwrap()])
            .insert(behavior);
    }
}

pub struct RobberBehaviorPlugin;

impl Plugin for RobberBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app
            // behavior assets
            .add_system(behavior_loader::<RobberBehavior>)
            .add_system(setup_behavior)
            .add_system(robber_run::run)
            .add_system(robber_bribe::run)
            .add_system(robber_captured::run)
            .add_system(robber_rest::run);
    }
}

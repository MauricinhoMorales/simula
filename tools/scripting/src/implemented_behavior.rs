use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

pub struct ImplementedBehaviorPlugin;

impl Plugin for ImplementedBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BehaviorTreePlugin::<ImplementedBehavior>::default())
            .add_system(subtree::run::<ImplementedBehavior>) // Subtrees are typed, need to register them separately
            .register_type::<Subtree<ImplementedBehavior>>();
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ImplementedBehaviorAttributes {
    pub pos: Vec2,
}

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "7CFA1742-7725-416C-B167-95DA01750E1C"]
pub enum ImplementedBehavior {
    Debug(Debug),
    Selector(Selector),
    Sequencer(Sequencer),
    All(All),
    Any(Any),
    Repeater(Repeater),
    Inverter(Inverter),
    Succeeder(Succeeder),
    Wait(Wait),
    Delay(Delay),
    Guard(Guard),
    Timeout(Timeout),
    Subtree(Subtree<ImplementedBehavior>), // Substrees are typed, this loads same tree type
}

impl Default for ImplementedBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorNodeInspectable<ImplementedBehavior> for ImplementedBehaviorAttributes {
    fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    fn get_pos(&self) -> Vec2 {
        self.pos
    }
}

impl BehaviorInspectable for ImplementedBehavior {
    fn color(&self) -> Color {
        match self {
            ImplementedBehavior::Debug(_) => Color::hex("#235").unwrap(),
            ImplementedBehavior::Selector(_) => Color::hex("#522").unwrap(),
            ImplementedBehavior::Sequencer(_) => Color::hex("#252").unwrap(),
            ImplementedBehavior::All(_) => Color::hex("#252").unwrap(),
            ImplementedBehavior::Any(_) => Color::hex("#522").unwrap(),
            ImplementedBehavior::Repeater(_) => Color::hex("#440").unwrap(),
            ImplementedBehavior::Inverter(_) => Color::hex("#440").unwrap(),
            ImplementedBehavior::Succeeder(_) => Color::hex("#440").unwrap(),
            ImplementedBehavior::Wait(_) => Color::hex("#235").unwrap(),
            ImplementedBehavior::Delay(_) => Color::hex("#440").unwrap(),
            ImplementedBehavior::Guard(_) => Color::hex("#440").unwrap(),
            ImplementedBehavior::Timeout(_) => Color::hex("#440").unwrap(),
            ImplementedBehavior::Subtree(_) => Color::hex("#440").unwrap(),
        }
    }

    #[rustfmt::skip]
    fn categories(&self) -> Vec<&'static str> {
        match self {
            ImplementedBehavior::Debug(_) => vec![<Debug as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Selector(_) => vec![<Selector as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Sequencer(_) => vec![<Sequencer as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::All(_) => vec![<All as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Any(_) => vec![<Any as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Repeater(_) => vec![<Repeater as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Inverter(_) => vec![<Inverter as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Succeeder(_) => vec![<Succeeder as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Wait(_) => vec![<Wait as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Delay(_) => vec![<Delay as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Guard(_) => vec![<Guard as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Timeout(_) => vec![<Timeout as BehaviorInfo>::TYPE.as_ref()],
            ImplementedBehavior::Subtree(_) => vec![<Subtree<ImplementedBehavior> as BehaviorInfo>::TYPE.as_ref()],
        }
    }
}

impl BehaviorFactory for ImplementedBehavior {
    type Attributes = ImplementedBehaviorAttributes;

    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            ImplementedBehavior::Debug(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Selector(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Sequencer(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::All(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Any(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Repeater(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Inverter(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Succeeder(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Wait(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Delay(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Guard(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Timeout(data) => BehaviorInfo::insert_with(commands, data),
            ImplementedBehavior::Subtree(data) => BehaviorInfo::insert_with(commands, data),
        }
    }

    fn label(&self) -> &str {
        match self {
            ImplementedBehavior::Debug(_) => <Debug as BehaviorInfo>::NAME,
            ImplementedBehavior::Selector(_) => <Selector as BehaviorInfo>::NAME,
            ImplementedBehavior::Sequencer(_) => <Sequencer as BehaviorInfo>::NAME,
            ImplementedBehavior::All(_) => <All as BehaviorInfo>::NAME,
            ImplementedBehavior::Any(_) => <Any as BehaviorInfo>::NAME,
            ImplementedBehavior::Repeater(_) => <Repeater as BehaviorInfo>::NAME,
            ImplementedBehavior::Inverter(_) => <Inverter as BehaviorInfo>::NAME,
            ImplementedBehavior::Succeeder(_) => <Succeeder as BehaviorInfo>::NAME,
            ImplementedBehavior::Wait(_) => <Wait as BehaviorInfo>::NAME,
            ImplementedBehavior::Delay(_) => <Delay as BehaviorInfo>::NAME,
            ImplementedBehavior::Guard(_) => <Guard as BehaviorInfo>::NAME,
            ImplementedBehavior::Timeout(_) => <Timeout as BehaviorInfo>::NAME,
            ImplementedBehavior::Subtree(_) => <Subtree<ImplementedBehavior> as BehaviorInfo>::NAME,
        }
    }

    fn reflect(&self) -> &dyn Reflect {
        match self {
            ImplementedBehavior::Debug(data) => data,
            ImplementedBehavior::Selector(data) => data,
            ImplementedBehavior::Sequencer(data) => data,
            ImplementedBehavior::All(data) => data,
            ImplementedBehavior::Any(data) => data,
            ImplementedBehavior::Repeater(data) => data,
            ImplementedBehavior::Inverter(data) => data,
            ImplementedBehavior::Succeeder(data) => data,
            ImplementedBehavior::Wait(data) => data,
            ImplementedBehavior::Delay(data) => data,
            ImplementedBehavior::Guard(data) => data,
            ImplementedBehavior::Timeout(data) => data,
            ImplementedBehavior::Subtree(data) => data,
        }
    }

    fn reflect_mut(&mut self) -> &mut dyn Reflect {
        match self {
            ImplementedBehavior::Debug(data) => data,
            ImplementedBehavior::Selector(data) => data,
            ImplementedBehavior::Sequencer(data) => data,
            ImplementedBehavior::All(data) => data,
            ImplementedBehavior::Any(data) => data,
            ImplementedBehavior::Repeater(data) => data,
            ImplementedBehavior::Inverter(data) => data,
            ImplementedBehavior::Succeeder(data) => data,
            ImplementedBehavior::Wait(data) => data,
            ImplementedBehavior::Delay(data) => data,
            ImplementedBehavior::Guard(data) => data,
            ImplementedBehavior::Timeout(data) => data,
            ImplementedBehavior::Subtree(data) => data,
        }
    }

    #[rustfmt::skip]
    fn copy_from(&mut self, entity: Entity, world: &World) -> Result<(), BehaviorMissing> {
        match self {
            ImplementedBehavior::Debug(data) => *data = world.get::<Debug>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Selector(data) => *data = world.get::<Selector>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Sequencer(data) => *data = world.get::<Sequencer>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::All(data) => *data = world.get::<All>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Any(data) => *data = world.get::<Any>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Repeater(data) => *data = world.get::<Repeater>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Inverter(data) => *data = world.get::<Inverter>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Succeeder(data) => *data = world.get::<Succeeder>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Wait(data) => *data = world.get::<Wait>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Delay(data) => *data = world.get::<Delay>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Guard(data) => *data = world.get::<Guard>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Timeout(data) => *data = world.get::<Timeout>(entity).ok_or(BehaviorMissing)?.clone(),
            ImplementedBehavior::Subtree(data) => *data = world.get::<Subtree<ImplementedBehavior>>(entity).ok_or(BehaviorMissing)?.clone(),
        };
        Ok(())
    }

    fn typ(&self) -> BehaviorType {
        match self {
            ImplementedBehavior::Debug(_) => <Debug as BehaviorInfo>::TYPE,
            ImplementedBehavior::Selector(_) => <Selector as BehaviorInfo>::TYPE,
            ImplementedBehavior::Sequencer(_) => <Sequencer as BehaviorInfo>::TYPE,
            ImplementedBehavior::All(_) => <All as BehaviorInfo>::TYPE,
            ImplementedBehavior::Any(_) => <Any as BehaviorInfo>::TYPE,
            ImplementedBehavior::Repeater(_) => <Repeater as BehaviorInfo>::TYPE,
            ImplementedBehavior::Inverter(_) => <Inverter as BehaviorInfo>::TYPE,
            ImplementedBehavior::Succeeder(_) => <Succeeder as BehaviorInfo>::TYPE,
            ImplementedBehavior::Wait(_) => <Wait as BehaviorInfo>::TYPE,
            ImplementedBehavior::Delay(_) => <Delay as BehaviorInfo>::TYPE,
            ImplementedBehavior::Guard(_) => <Guard as BehaviorInfo>::TYPE,
            ImplementedBehavior::Timeout(_) => <Timeout as BehaviorInfo>::TYPE,
            ImplementedBehavior::Subtree(_) => <Subtree<ImplementedBehavior> as BehaviorInfo>::TYPE,
        }
    }

    fn list() -> Vec<Self> {
        vec![
            ImplementedBehavior::Debug(Default::default()),
            ImplementedBehavior::Selector(Default::default()),
            ImplementedBehavior::Sequencer(Default::default()),
            ImplementedBehavior::All(Default::default()),
            ImplementedBehavior::Any(Default::default()),
            ImplementedBehavior::Repeater(Default::default()),
            ImplementedBehavior::Inverter(Default::default()),
            ImplementedBehavior::Succeeder(Default::default()),
            ImplementedBehavior::Wait(Default::default()),
            ImplementedBehavior::Delay(Default::default()),
            ImplementedBehavior::Guard(Default::default()),
            ImplementedBehavior::Timeout(Default::default()),
            ImplementedBehavior::Subtree(Default::default()),
        ]
    }
}

use crate::{
    inspector::graph::{
        AllMyNodeTemplates, MyDataType, MyEditorState, MyGraphState, MyNodeTemplate, MyResponse,
    },
    BehaviorCursor, BehaviorFactory, BehaviorNode, BehaviorTree, BehaviorType,
};
use bevy::{ecs::system::SystemState, prelude::*, asset::HandleId, reflect::TypeRegistry};
use bevy_inspector_egui::{bevy_egui::EguiContexts, egui, bevy_inspector::hierarchy::{hierarchy_ui, SelectedEntities}};
use egui_node_graph::NodeResponse;
use core::any::TypeId;

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, HandleId),
}

#[derive(Resource)]
struct UiState {
    // tree: Tree<EguiWindow>,
    // viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
    // gizmo_mode: GizmoMode,
}

// fn select_resource(
//     ui: &mut egui::Ui,
//     type_registry: &TypeRegistry,
//     selection: &mut InspectorSelection,
// ) {
//     let mut resources: Vec<_> = type_registry
//         .iter()
//         .filter(|registration| registration.data::<ReflectResource>().is_some())
//         .map(|registration| (registration.short_name().to_owned(), registration.type_id()))
//         .collect();
//     resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

//     for (resource_name, type_id) in resources {
//         let selected = match *selection {
//             InspectorSelection::Resource(selected, _) => selected == type_id,
//             _ => false,
//         };

//         if ui.selectable_label(selected, &resource_name).clicked() {
//             *selection = InspectorSelection::Resource(type_id, resource_name);
//         }
//     }
// }

#[derive(Default, Clone, Resource)]
pub struct BehaviorInspectorAttributes;

#[derive(Default, Clone, Resource, Reflect)]
pub struct BehaviorInspector {
    pub selected: BehaviorInspectorItem,
}

impl BehaviorInspector {
    pub fn select(&mut self, entity: Entity, name: String) {
        self.selected = BehaviorInspectorItem {
            entity: Some(entity),
            name,
        };
    }

    pub fn unselect(&mut self) {
        self.selected = BehaviorInspectorItem::default();
    }
}

#[derive(Clone, PartialEq, Reflect)]
pub struct BehaviorInspectorItem {
    pub entity: Option<Entity>,
    pub name: String,
}

impl Default for BehaviorInspectorItem {
    fn default() -> Self {
        Self {
            entity: None,
            name: "Select Behavior".to_string(),
        }
    }
}

fn item_label(item: &BehaviorInspectorItem) -> String {
    let entity = if let Some(entity) = item.entity {
        format!("[{}]: ", entity.index())
    } else {
        "".to_string()
    };
    format!("{}{}", entity, item.name)
}

fn find_cursor(world: &mut World, tree_entity: Entity) -> Option<Entity> {
    let mut query = world.query_filtered::<(Entity, &BehaviorNode), With<BehaviorCursor>>();
    for (entity, node) in query.iter(world) {
        if node.tree == Some(tree_entity) {
            return Some(entity);
        }
    }
    None
}

pub fn behavior_graph_ui<T: BehaviorFactory>(world: &mut World) {
    let mut behavior_trees = world.query::<(
        Entity,
        Option<&Name>,
        &BehaviorTree,
        &mut MyGraphState,
        &mut MyEditorState<T>,
    )>();
    let behavior_inspector = world.resource_mut::<BehaviorInspector>().clone();
    let mut system_state: SystemState<EguiContexts> = SystemState::new(world);
    let mut context = system_state.get_mut(world).ctx_mut().clone();

    // egui::TopBottomPanel::top("top").show(&mut context, |ui| {
    //     egui::menu::bar(ui, |ui| {
    //         egui::menu::menu_button(ui, "File", |ui| {
    //             if ui.add(egui::Button::new("New")).clicked() {
    //                 println!("New");
    //             };
    //             if ui.add(egui::Button::new("Save")).clicked() {
    //                 println!("Save");
    //             };
    //             if ui.button("Save As...").clicked() {
    //                 println!("Save As...");
    //             };
    //         });

    //         egui::ComboBox::from_id_source("Behavior Inspector Selector")
    //         .selected_text(item_label(&behavior_inspector.selected))
    //         .show_ui(ui, |ui| {
    //             let mut selectable_behaviors: Vec<BehaviorInspectorItem> = {
    //                 behavior_trees
    //                     .iter(world)
    //                     .map(|(entity, name, _, _, _)| BehaviorInspectorItem {
    //                         entity: Some(entity),
    //                         name: name.unwrap_or(&Name::new("")).to_string(),
    //                     })
    //                     .collect::<Vec<_>>()
    //             };
    //             selectable_behaviors.insert(0, BehaviorInspectorItem::default());
    //             for selectable_behavior in selectable_behaviors {
    //                 if ui
    //                     .selectable_label(
    //                         behavior_inspector.selected == selectable_behavior,
    //                         item_label(&selectable_behavior),
    //                     )
    //                     .clicked()
    //                 {
    //                     let mut behavior_inspector = world.resource_mut::<BehaviorInspector>();
    //                     behavior_inspector.selected = selectable_behavior;
    //                 }
    //             }
    //         });
            
    //     });
    // });

    if let BehaviorInspectorItem {
        entity: Some(entity),
        name,
    } = &behavior_inspector.selected
    {
        let (tree_entity, tree_root) = {
            let Ok((tree_entity, _, behavior_tree, _, _)) = behavior_trees.get_mut(world, *entity) else {
            return;};
            let Some(tree_root) = behavior_tree.root else {return;};
            (tree_entity, tree_root)
        };

        // match find_cursor(world, tree_entity) {
        //     Some(_cursor) => {
        //         ui.label("Running");
        //     }
        //     None => {
        //         if ui.button("Run").clicked() {
        //             world.entity_mut(tree_root).insert(BehaviorCursor::Delegate);
        //         }
        //     }
        // }

        egui::Window::new(format!("Behavior: {}", name))
            .title_bar(true)
            .resizable(true)
            .collapsible(true)
            .scroll2([true, true])
            .show(&mut context, |ui| {

                
                let Ok((_, _, _, mut graph_state, mut editor_state)) = behavior_trees.get_mut(world, *entity) else {
                    return;};

                    let graph_response = editor_state.draw_graph_editor(
                                        ui,
                                        AllMyNodeTemplates::<T>::default(),
                                        &mut graph_state,
                                        Vec::default(),
                                    );

                    for response in graph_response.node_responses {
                        println!("response: {:?}", response);
                        match response {
                            NodeResponse::ConnectEventEnded { output: output_id, input: input_id } => {
                                // Check if output is already connected, and if so, remove the previous connection
                                let mut removes = vec![];
                                for (other_input, other_output) in editor_state.graph.connections.iter() {
                                    if *other_output == output_id {
                                        if other_input != input_id {
                                            removes.push(other_input);
                                        }
                                    }
                                }
                                for other_input in removes {
                                    editor_state.graph.connections.remove(other_input);
                                }

                                // If composite type, dynamically adjust outputs of node
                                let node_id = editor_state.graph.outputs[output_id].node;
                                let node = editor_state.graph.nodes.get(node_id).unwrap();
                                if let MyNodeTemplate::Behavior(behavior) = &node.user_data.data {
                                    if behavior.typ() == BehaviorType::Composite {
                                        // Get all unused outputs
                                        let mut unused_outputs = vec![];
                                        node.outputs.iter().for_each(|(_, output_id)| {
                                            let connected = editor_state.graph.connections.iter().filter(|(_, other_output)| {
                                                println!("Output: {:#?} == {:#?}", output_id, *other_output);
                                                *other_output == output_id
                                            }).count() > 0;
                                            if !connected {
                                                unused_outputs.push(*output_id);
                                            }
                                        });

                                        // If there are no unused outputs, add a new output
                                        if unused_outputs.len() == 0 {
                                            editor_state.graph.add_output_param(node_id, "B".into(), MyDataType::Flow);
                                        }

                                        // Remove all but one unused output
                                        while unused_outputs.len() > 1 {
                                            if let Some(output_id) = unused_outputs.pop() {
                                                editor_state.graph.remove_output_param(output_id);
                                            }
                                        }
                                    }
                                }
                            }
                            NodeResponse::User(MyResponse::NodeEdited(node_id, data)) => {
                                if let Some(node) = editor_state.graph.nodes.get_mut(node_id) {
                                    node.user_data.data = MyNodeTemplate::Behavior(data);
                                }
                            }
                            NodeResponse::User(MyResponse::NameEdited(node_id, name)) => {
                                if let Some(node) = editor_state.graph.nodes.get_mut(node_id) {
                                    node.user_data.name = name;
                                }
                            }
                            _ => {}
                        }
                    }
            });
    }

    // egui::Window::new("Behavior Inspector").show(&mut context, |ui| {



    //     egui::ComboBox::from_id_source("Behavior Inspector Selector")
    //         .selected_text(item_label(&behavior_inspector.selected))
    //         .show_ui(ui, |ui| {
    //             let mut selectable_behaviors: Vec<BehaviorInspectorItem> = {
    //                 behavior_trees
    //                     .iter(world)
    //                     .map(|(entity, name, _, _, _)| BehaviorInspectorItem {
    //                         entity: Some(entity),
    //                         name: name.unwrap_or(&Name::new("")).to_string(),
    //                     })
    //                     .collect::<Vec<_>>()
    //             };
    //             selectable_behaviors.insert(0, BehaviorInspectorItem::default());
    //             for selectable_behavior in selectable_behaviors {
    //                 if ui
    //                     .selectable_label(
    //                         behavior_inspector.selected == selectable_behavior,
    //                         item_label(&selectable_behavior),
    //                     )
    //                     .clicked()
    //                 {
    //                     let mut behavior_inspector = world.resource_mut::<BehaviorInspector>();
    //                     behavior_inspector.selected = selectable_behavior;
    //                 }
    //             }
    //         });

        
    // });
}

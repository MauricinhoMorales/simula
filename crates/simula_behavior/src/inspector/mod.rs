use crate::{
    inspector::graph::{
        BehaviorDataType, BehaviorEditorState, BehaviorGraphState, BehaviorNodeData,
        BehaviorNodeTemplate, BehaviorNodeTemplates, BehaviorResponse, BehaviorValueType,
    },
    protocol::{
        BehaviorClient, BehaviorFileData, BehaviorFileId, BehaviorFileName, BehaviorProtocolClient,
        BehaviorProtocolServer, BehaviorServer, BehaviorTelemetry,
    },
    Behavior, BehaviorFactory, BehaviorType,
};
use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use crossbeam_channel::unbounded;
use egui_node_graph::{Graph, NodeId, NodeResponse, NodeTemplateTrait};
use serde::{Deserialize, Serialize};
use simula_inspector::{egui, Inspector, Inspectors};
use std::borrow::Cow;

pub mod graph;

#[derive(Default)]
pub struct BehaviorInspectorPlugin<T: BehaviorFactory>(pub std::marker::PhantomData<T>);

impl<T> Plugin for BehaviorInspectorPlugin<T>
where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
{
    fn build(&self, app: &mut App) {
        // Setup bi-directional communication channels
        let (protocol_client_sender, protocol_client_receiver) = unbounded();
        let (protocol_server_sender, protocol_server_receiver) = unbounded();

        let client = BehaviorClient::<T> {
            sender: protocol_client_sender,
            receiver: protocol_server_receiver,
        };

        let server = BehaviorServer::<T> {
            sender: protocol_server_sender,
            receiver: protocol_client_receiver,
        };

        app.insert_resource(client)
            .insert_resource(server)
            .insert_resource(BehaviorInspector::<T>::default())
            .add_startup_system(setup::<T>)
            .add_system(update::<T>);
    }
}

#[derive(Clone, Copy)]
enum BehaviorInspectorState {
    New,
    Editing,
    Load,
    Loading,
    Save,
    Saving,
    Run,
    Starting,
    Running,
    Stop,
    Stopping,
}

#[derive(Clone)]
struct BehaviorInspectorItem<T: BehaviorFactory> {
    pub entity: Option<Entity>,
    pub name: BehaviorFileName,
    pub state: BehaviorInspectorState,
    pub collapsed: bool,
    pub behavior: Option<Behavior<T>>,
}

#[derive(Default, Clone, Resource)]
struct BehaviorInspector<T: BehaviorFactory> {
    pub selected: Option<BehaviorFileId>,
    pub behaviors: HashMap<BehaviorFileId, BehaviorInspectorItem<T>>,
}

fn get_label_from_id<T: BehaviorFactory>(
    file_id: &Option<BehaviorFileId>,
    behavior_inspector: &BehaviorInspector<T>,
) -> Cow<'static, str> {
    match file_id {
        None => Cow::Borrowed("None"),
        Some(behavior_file_id) => {
            let behavior_inspector_item = &behavior_inspector.behaviors[behavior_file_id];
            (*behavior_inspector_item.name).clone()
        }
    }
}

fn menu_ui<T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>>(
    ui: &mut egui::Ui,
    world: &mut World,
) {
    egui::menu::menu_button(ui, "🏃 Behaviors", |ui| {
        if ui.add(egui::Button::new("✚ New")).clicked() {
            let file_id = BehaviorFileId::new();
            let file_name = BehaviorFileName(format!("bt_{}", *file_id).into());
            let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
            behavior_inspector.behaviors.insert(
                file_id.clone(),
                BehaviorInspectorItem {
                    entity: None,
                    name: file_name,
                    state: BehaviorInspectorState::New,
                    collapsed: false,
                    behavior: None,
                },
            );
            behavior_inspector.selected = Some(file_id.clone());
        }

        let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
        if let Some(file_id) = behavior_inspector.selected.clone() {
            if let Some(behavior_inspector_item) = behavior_inspector.behaviors.get_mut(&file_id) {
                if let BehaviorInspectorState::Saving = behavior_inspector_item.state {
                } else {
                    if ui.add(egui::Button::new("💾 Save")).clicked() {
                        behavior_inspector_item.state = BehaviorInspectorState::Save;
                        warn!("Saving behavior {:?}", file_id);
                    }
                }
            }
        }
    });

    let behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
    let mut new_selected = behavior_inspector.selected.clone();

    egui::ComboBox::from_id_source("Behavior Inspector Selector")
        .selected_text(get_label_from_id(
            &behavior_inspector.selected,
            &behavior_inspector,
        ))
        .show_ui(ui, |ui| {
            let mut selectable_behaviors: Vec<Option<BehaviorFileId>> = {
                let mut keys: Vec<BehaviorFileId> =
                    behavior_inspector.behaviors.keys().cloned().collect();
                keys.sort_by(|a, b| {
                    let name_a = &behavior_inspector.behaviors[a].name;
                    let name_b = &behavior_inspector.behaviors[b].name;
                    name_a.cmp(&name_b)
                });
                keys.iter().map(|key| Some(key.clone())).collect()
            };
            selectable_behaviors.insert(0, None);
            for selectable_behavior in selectable_behaviors {
                ui.allocate_ui(egui::vec2(200.0, 1.0), |ui| {
                    if ui
                        .selectable_label(
                            behavior_inspector.selected == selectable_behavior,
                            get_label_from_id(&selectable_behavior, &behavior_inspector),
                        )
                        .clicked()
                    {
                        println!("Selected: {:?}", selectable_behavior);
                        new_selected = selectable_behavior.clone();
                    }
                });
            }
        });

    let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
    behavior_inspector.selected = new_selected;
}

fn window_ui<T: BehaviorFactory>(context: &mut egui::Context, world: &mut World) {
    let selected_behavior = world
        .resource_mut::<BehaviorInspector<T>>()
        .selected
        .clone();
    let Some(selected_behavior) = selected_behavior else { return;};
    let behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
    let Some((file_name, inspector_item_state, entity))
        = behavior_inspector.behaviors
        .get(&selected_behavior)
        .and_then(|item| Some((item.name.clone(), item.state, item.entity))) else { return;};
    match inspector_item_state {
        BehaviorInspectorState::Editing => {}
        BehaviorInspectorState::Save => {}
        BehaviorInspectorState::Saving => {}
        BehaviorInspectorState::Run => {}
        BehaviorInspectorState::Starting => {}
        BehaviorInspectorState::Running => {}
        BehaviorInspectorState::Stop => {}
        BehaviorInspectorState::Stopping => {}
        _ => return,
    }
    let Some(entity) = entity else { return;};

    let mut behavior_graphs = world.query::<(
        Entity,
        Option<&Name>,
        &mut BehaviorGraphState,
        &mut BehaviorEditorState<T>,
    )>();

    let window = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .single(world);
    let default_size = egui::vec2(window.width() * 0.7, window.height() * 0.7);

    let mut open = true;
    let mut window_name = format!("{}", *file_name);
    egui::Window::new(&format!("BT:[{}]", *selected_behavior))
        .default_size(default_size)
        .title_bar(false)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(22, 20, 25, 200))
                .inner_margin(3.0),
        )
        .show(context, |ui| {
            let mut pan_reset = false;
            let mut pan = egui::vec2(0.0, 0.0);
            context.input(|i| {
                pan = i.scroll_delta;
            });
            let mut pan_length = 0.0;
            if let Ok((_, _, _graph_state, editor_state)) = behavior_graphs.get(world, entity) {
                pan_length = editor_state.pan_zoom.pan.length_sq();
            }

            ui.vertical(|ui| {
                let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
                let inspector_item = behavior_inspector
                    .behaviors
                    .get_mut(&selected_behavior)
                    .unwrap();

                ui.horizontal(|ui| {
                    egui::menu::bar(ui, |ui| {
                        if inspector_item.collapsed {
                            if ui.add(egui::Button::new("▶").frame(false)).clicked() {
                                inspector_item.collapsed = false;
                            }
                        } else {
                            if ui.add(egui::Button::new("▼").frame(false)).clicked() {
                                inspector_item.collapsed = true;
                            }
                        }

                        if let BehaviorInspectorState::Saving = inspector_item_state {
                            ui.add_enabled(false, egui::Label::new("💾"));
                        } else if let BehaviorInspectorState::Editing = inspector_item_state {
                            if ui.add(egui::Button::new("💾")).clicked() {
                                inspector_item.state = BehaviorInspectorState::Save;
                            }
                        }

                        if ui
                            .add_enabled(pan_length > 1000.0, egui::Button::new("⨀").frame(true))
                            .clicked()
                        {
                            pan_reset = true;
                        }

                        ui.add_space(20.0);

                        if let BehaviorInspectorState::Editing = inspector_item_state {
                            if ui.add(egui::Button::new("⏵").frame(true)).clicked() {
                                inspector_item.state = BehaviorInspectorState::Run;
                            }
                        }

                        if let BehaviorInspectorState::Running = inspector_item_state {
                            if ui.add(egui::Button::new("⏹").frame(true)).clicked() {
                                inspector_item.state = BehaviorInspectorState::Stop;
                            }
                        }

                        ui.style_mut().visuals.extreme_bg_color =
                            egui::Color32::from_rgba_premultiplied(0, 0, 0, 100);
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut window_name)
                                    .desired_width(50.0)
                                    .clip_text(false),
                            )
                            .changed()
                        {
                            inspector_item.name = BehaviorFileName(window_name.clone().into());
                        }

                        // Space for the little cross icon
                        ui.add_space(8.0);
                        ui.add_space(ui.available_width() - 12.0);
                        if close_button(ui, ui.available_rect_before_wrap()).clicked() {
                            println!("Button clicked!");
                            open = false;
                        }
                    });
                });

                if !inspector_item.collapsed {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(52, 50, 55, 140))
                        .stroke(egui::Stroke::NONE)
                        .inner_margin(egui::Margin {
                            left: 10.0,
                            right: 10.0,
                            top: 10.0,
                            bottom: 10.0,
                        })
                        .outer_margin(egui::Margin {
                            left: -3.0,
                            right: -3.0,
                            top: 0.0,
                            bottom: -3.0,
                        })
                        .show(ui, |ui| {
                            let (mut graph_state, mut editor_state) =
                                if let Ok((_, _, graph_state, editor_state)) =
                                    behavior_graphs.get_mut(world, entity)
                                {
                                    (graph_state, editor_state)
                                } else {
                                    return;
                                };

                            // keep root node locked
                            if let Some(root_node) = graph_state.root_node {
                                let position =
                                    editor_state.node_positions.get_mut(root_node).unwrap();
                                *position = egui::pos2(0.0, 0.0);
                            } else {
                                for (nodeid, node) in &editor_state.graph.nodes {
                                    if let BehaviorNodeTemplate::Root = &node.user_data.data {
                                        graph_state.root_node = Some(nodeid);
                                        break;
                                    }
                                }
                            }

                            // handle pan
                            let scroll_rect = ui.available_rect_before_wrap();
                            if ui.rect_contains_pointer(scroll_rect) {
                                editor_state.pan_zoom.pan += pan;
                            }
                            if pan_reset {
                                editor_state.pan_zoom.pan = egui::vec2(0.0, 0.0);
                            }

                            let graph_response = editor_state.draw_graph_editor(
                                ui,
                                BehaviorNodeTemplates::<T>::default(),
                                &mut graph_state,
                                Vec::default(),
                            );

                            for response in graph_response.node_responses {
                                trace!("response: {:?}", response);
                                match response {
                                    NodeResponse::SelectNode(node_id) => {
                                        graph_state.active_node = Some(node_id);
                                    }
                                    NodeResponse::DeselectNode => {
                                        graph_state.active_node = None;
                                    }
                                    NodeResponse::ConnectEventEnded {
                                        output: output_id,
                                        input: input_id,
                                    } => {
                                        // Check if output is already connected, and if so, remove the previous connection
                                        let mut removes = vec![];
                                        for (other_input, other_output) in
                                            editor_state.graph.connections.iter()
                                        {
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
                                        if let BehaviorNodeTemplate::Behavior(behavior) =
                                            &node.user_data.data
                                        {
                                            if behavior.typ() == BehaviorType::Composite {
                                                // Get all unused outputs
                                                let mut unused_outputs = vec![];
                                                node.outputs.iter().for_each(|(_, output_id)| {
                                                    let connected = editor_state
                                                        .graph
                                                        .connections
                                                        .iter()
                                                        .filter(|(_, other_output)| {
                                                            println!(
                                                                "Output: {:#?} == {:#?}",
                                                                output_id, *other_output
                                                            );
                                                            *other_output == output_id
                                                        })
                                                        .count()
                                                        > 0;
                                                    if !connected {
                                                        unused_outputs.push(*output_id);
                                                    }
                                                });

                                                // If there are no unused outputs, add a new output
                                                if unused_outputs.len() == 0 {
                                                    editor_state.graph.add_output_param(
                                                        node_id,
                                                        "B".into(),
                                                        BehaviorDataType::Flow,
                                                    );
                                                }

                                                // Remove all but one unused output
                                                while unused_outputs.len() > 1 {
                                                    if let Some(output_id) = unused_outputs.pop() {
                                                        editor_state
                                                            .graph
                                                            .remove_output_param(output_id);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    NodeResponse::User(BehaviorResponse::NodeEdited(
                                        node_id,
                                        data,
                                    )) => {
                                        if let Some(node) =
                                            editor_state.graph.nodes.get_mut(node_id)
                                        {
                                            node.user_data.data =
                                                BehaviorNodeTemplate::Behavior(data);
                                        }
                                    }
                                    NodeResponse::User(BehaviorResponse::NameEdited(
                                        node_id,
                                        name,
                                    )) => {
                                        if let Some(node) =
                                            editor_state.graph.nodes.get_mut(node_id)
                                        {
                                            node.label = name;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        });
                }
            });
        });

    if !open {
        let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
        behavior_inspector.selected = None;
    }
}

fn setup<T>(mut inspectors: ResMut<Inspectors>)
where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
{
    inspectors.inspectors.push(Inspector {
        menu_ui: menu_ui::<T>,
        window_ui: window_ui::<T>,
    });
}

fn get_root_child<T: BehaviorFactory>(
    graph: &Graph<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>>,
) -> Option<NodeId> {
    for node in graph.nodes.values() {
        if let BehaviorNodeTemplate::Root = &node.user_data.data {
            // All this just to get first child of root node
            let root_child_id: Option<NodeId> = node
                .outputs
                .iter()
                .filter_map(|(_, output_id)| {
                    graph
                        .connections
                        .iter()
                        .find(|(input_id, rhs_output_id)| {
                            output_id == *rhs_output_id
                                && graph.inputs[*input_id].typ == BehaviorDataType::Flow
                        })
                        .and_then(|(input_id, _)| Some(graph.inputs[input_id].node))
                })
                .next();
            return root_child_id;
        }
    }
    None
}

fn update<T>(
    mut commands: Commands,
    time: Res<Time>,
    type_registry: Res<AppTypeRegistry>,
    mut behavior_inspector: ResMut<BehaviorInspector<T>>,
    behavior_client: Res<BehaviorClient<T>>,
    mut graph_states: Query<&mut BehaviorGraphState>,
    mut editor_states: Query<&mut BehaviorEditorState<T>>,
) where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
{
    // provide time for all graph states
    for mut graph_state in graph_states.iter_mut() {
        graph_state.time = time.clone();
    }

    if let Some(selected_behavior) = behavior_inspector.selected.clone() {
        if let Some(behavior_inspector_item) =
            behavior_inspector.behaviors.get_mut(&selected_behavior)
        {
            // If selected behavior is load, load it
            if let BehaviorInspectorState::Load = behavior_inspector_item.state {
                info!("Loading behavior: {}", *selected_behavior);
                behavior_inspector_item.state = BehaviorInspectorState::Loading;
                behavior_client
                    .sender
                    .send(BehaviorProtocolClient::LoadFile(selected_behavior))
                    .unwrap();
            }
            // if selected behavior is new, create it
            else if let BehaviorInspectorState::New = behavior_inspector_item.state {
                info!("Creating behavior: {}", *selected_behavior);

                let mut graph_state = BehaviorGraphState {
                    type_registry: type_registry.0.clone(),
                    ..Default::default()
                };
                let mut editor_state = BehaviorEditorState::<T>::default();

                // create the only root node allowed
                let root_node_data = BehaviorNodeData {
                    data: BehaviorNodeTemplate::Root,
                    state: None,
                };
                let root_node =
                    editor_state
                        .graph
                        .add_node("Root".into(), root_node_data, |graph, node_id| {
                            BehaviorNodeTemplate::Root.build_node(graph, &mut graph_state, node_id)
                        });

                editor_state
                    .node_positions
                    .insert(root_node, egui::Pos2::new(0.0, 0.0));
                editor_state.node_order.push(root_node);

                let entity = commands
                    .spawn(Name::new(format!("BT: {}", *selected_behavior)))
                    .insert(graph_state)
                    .insert(editor_state)
                    .id();
                behavior_inspector_item.entity = Some(entity);
                behavior_inspector_item.state = BehaviorInspectorState::Editing;
            }
            // if selected behavior is save, save it
            else if let BehaviorInspectorState::Save = behavior_inspector_item.state {
                info!("Saving behavior: {}", *selected_behavior);
                if let Some(entity) = behavior_inspector_item.entity {
                    behavior_inspector_item.state = BehaviorInspectorState::Saving;
                    if let Ok(editor_state) = editor_states.get(entity) {
                        if let Ok(file_data) = ron::ser::to_string_pretty(
                            &editor_state,
                            ron::ser::PrettyConfig::default(),
                        ) {
                            info!("Saving behavior: {}", *behavior_inspector_item.name);
                            behavior_inspector_item.state = BehaviorInspectorState::Saving;
                            behavior_client
                                .sender
                                .send(BehaviorProtocolClient::SaveFile(
                                    selected_behavior.clone(),
                                    behavior_inspector_item.name.clone(),
                                    BehaviorFileData(file_data.into()),
                                ))
                                .unwrap();
                        } else {
                            error!(
                                "Failed to serialize behavior: {}",
                                *behavior_inspector_item.name
                            );
                            behavior_inspector_item.state = BehaviorInspectorState::Editing;
                        };
                    } else {
                        error!(
                            "No graph editor state for behavior: {}",
                            *behavior_inspector_item.name
                        );
                        behavior_inspector_item.state = BehaviorInspectorState::New;
                    }
                } else {
                    error!("No entity for behavior: {}", *behavior_inspector_item.name);
                    behavior_inspector_item.state = BehaviorInspectorState::New;
                }
            }
            // if selected behavior should run, run it
            else if let BehaviorInspectorState::Run = behavior_inspector_item.state {
                info!("Run behavior: {}", *selected_behavior);
                if let Some(entity) = behavior_inspector_item.entity {
                    if let Ok(editor_state) = editor_states.get(entity) {
                        for node in editor_state.graph.nodes.values() {
                            if let BehaviorNodeTemplate::Root = &node.user_data.data {
                                let root_child_id = get_root_child(&editor_state.graph);
                                if let Some(root_child_id) = root_child_id {
                                    let behavior =
                                        graph_to_behavior(&editor_state.graph, root_child_id);
                                    println!("behavior: {:#?}", behavior);
                                    behavior_inspector_item.behavior = Some(behavior.clone());
                                    behavior_inspector_item.state =
                                        BehaviorInspectorState::Starting;
                                    behavior_client
                                        .sender
                                        .send(BehaviorProtocolClient::Run(
                                            selected_behavior.clone(),
                                            behavior,
                                        ))
                                        .unwrap();
                                } else {
                                    error!("No root child");
                                    behavior_inspector_item.state = BehaviorInspectorState::Editing;
                                }
                                break;
                            }
                        }
                    }
                }
            }
            // if selected behavior should stop, stop it
            else if let BehaviorInspectorState::Stop = behavior_inspector_item.state {
                info!("Stop behavior: {}", *selected_behavior);
                behavior_inspector_item.state = BehaviorInspectorState::Stopping;
                behavior_client
                    .sender
                    .send(BehaviorProtocolClient::Stop(selected_behavior))
                    .unwrap();
            }
        }
    }

    while let Ok(server_msg) = behavior_client.receiver.try_recv() {
        match server_msg {
            // Receive list of behaviors
            BehaviorProtocolServer::FileNames(behaviors) => {
                for (file_id, file_name) in &behaviors {
                    if !behavior_inspector.behaviors.contains_key(&file_id) {
                        behavior_inspector.behaviors.insert(
                            file_id.clone(),
                            BehaviorInspectorItem {
                                entity: None,
                                name: file_name.clone(),
                                state: BehaviorInspectorState::Load,
                                collapsed: false,
                                behavior: None,
                            },
                        );
                    }
                }
            }
            // Receive behavior data
            BehaviorProtocolServer::File(file_id, file_data) => {
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Loading = behavior_inspector_item.state {
                        let entity = commands
                            .spawn(Name::new(format!("BT: {}", *file_id)))
                            .insert(BehaviorGraphState {
                                type_registry: type_registry.0.clone(),
                                ..Default::default()
                            })
                            .insert(
                                ron::de::from_str::<BehaviorEditorState<T>>(&file_data).unwrap(),
                            )
                            .id();
                        behavior_inspector_item.entity = Some(entity);
                        behavior_inspector_item.state = BehaviorInspectorState::Editing;
                    }
                }
            }
            // Receive file saved
            BehaviorProtocolServer::FileSaved(file_id) => {
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Saving = behavior_inspector_item.state {
                        behavior_inspector_item.state = BehaviorInspectorState::Editing;
                    }
                } else {
                    error!("Unexpected file saved: {:?}", file_id);
                }
            }
            // Behavior started
            BehaviorProtocolServer::Started(file_id) => {
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Starting = behavior_inspector_item.state {
                        behavior_inspector_item.state = BehaviorInspectorState::Running;
                    }
                } else {
                    error!("Unexpected behavior started: {:?}", file_id);
                }
            }
            // Behavior stopped
            BehaviorProtocolServer::Stopped(file_id) => {
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let Some(behavior) = behavior_inspector_item.behavior.clone() {
                        if let Some(entity) = behavior_inspector_item.entity {
                            if let Ok(mut editor_state) = editor_states.get_mut(entity) {
                                if let Some(root_child_id) = get_root_child(&editor_state.graph) {
                                    behavior_to_graph(
                                        &mut editor_state.graph,
                                        root_child_id,
                                        &behavior,
                                    );
                                }
                            }
                        }
                    }
                    if let BehaviorInspectorState::Stopping = behavior_inspector_item.state {
                        behavior_inspector_item.state = BehaviorInspectorState::Editing;
                    }
                } else {
                    error!("Unexpected behavior stopped: {:?}", file_id);
                }
            }
            // Behavior telemetry
            BehaviorProtocolServer::Telemetry(file_id, telemetry) => {
                // println!("received telemetry: {:#?}", telemetry);
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Running = behavior_inspector_item.state {
                        if let Some(entity) = behavior_inspector_item.entity {
                            if let Ok(mut editor_state) = editor_states.get_mut(entity) {
                                let root_child_id = get_root_child(&editor_state.graph);
                                if let Some(root_child_id) = root_child_id {
                                    behavior_telemerty_to_graph(
                                        &mut editor_state.graph,
                                        root_child_id,
                                        &telemetry,
                                    );
                                } else {
                                    error!("No root child");
                                }
                            }
                        }
                    }
                } else {
                    error!("Unexpected behavior telemetry: {:?}", file_id);
                }
            }
        }
    }
}

fn close_button(ui: &mut egui::Ui, node_rect: egui::Rect) -> egui::Response {
    // Measurements
    let margin = 6.0;
    let size = 6.0;
    let stroke_width = 2.0;
    let offs = margin + size / 2.0;

    let position = egui::pos2(node_rect.right() - offs, node_rect.top() + offs);
    let rect = egui::Rect::from_center_size(position, egui::vec2(size, size));
    let resp = ui.allocate_rect(rect, egui::Sense::click());

    let color = if resp.clicked() {
        egui::Color32::WHITE
    } else if resp.hovered() {
        egui::Color32::GRAY
    } else {
        egui::Color32::DARK_GRAY
    };
    let stroke = egui::Stroke {
        width: stroke_width,
        color,
    };

    ui.painter()
        .line_segment([rect.left_top(), rect.right_bottom()], stroke);
    ui.painter()
        .line_segment([rect.right_top(), rect.left_bottom()], stroke);

    resp
}

// Recursively build behavior from graph
fn graph_to_behavior<T>(
    graph: &Graph<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>>,
    node_id: NodeId,
) -> Behavior<T>
where
    T: BehaviorFactory,
{
    let node: &egui_node_graph::Node<BehaviorNodeData<T>> = &graph.nodes[node_id];
    let BehaviorNodeTemplate::Behavior(behavior) = &node.user_data.data else { panic!("Expected behavior node") };
    let mut behavior = Behavior(node.label.clone(), behavior.clone(), vec![]);
    for (_, output_id) in node.outputs.iter() {
        let child_id = graph
            .connections
            .iter()
            .find(|(input_id, rhs_output_id)| {
                output_id == *rhs_output_id && graph.inputs[*input_id].typ == BehaviorDataType::Flow
            })
            .and_then(|(input_id, _)| Some(graph.inputs[input_id].node));
        if let Some(child_id) = child_id {
            let child = graph_to_behavior(graph, child_id);
            behavior.2.push(child);
        }
    }
    behavior
}

// Recursively update graph from behavior
fn behavior_to_graph<T>(
    graph: &mut Graph<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>>,
    node_id: NodeId,
    behavior: &Behavior<T>,
) where
    T: BehaviorFactory,
{
    // Update graph node with behavior telemetry
    let node: &mut egui_node_graph::Node<BehaviorNodeData<T>> = &mut graph.nodes[node_id];
    node.user_data.data = BehaviorNodeTemplate::Behavior(behavior.1.clone());
    node.user_data.state = None;

    // Get node children
    let node_children: Vec<NodeId> = node
        .outputs
        .iter()
        .filter_map(|(_, output_id)| {
            graph
                .connections
                .iter()
                .find(|(input_id, rhs_output_id)| {
                    output_id == *rhs_output_id
                        && graph.inputs[*input_id].typ == BehaviorDataType::Flow
                })
                .and_then(|(input_id, _)| Some(graph.inputs[input_id].node))
        })
        .collect();

    // Zip and iterate over children
    let node_children = node_children.iter().cloned();
    let behavior_children = behavior.2.iter();
    for (node_child, behavior_child) in node_children.zip(behavior_children) {
        behavior_to_graph(graph, node_child, behavior_child);
    }
}

// Recursively update graph from behavior telemetry
fn behavior_telemerty_to_graph<T>(
    graph: &mut Graph<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>>,
    node_id: NodeId,
    telemetry: &BehaviorTelemetry<T>,
) where
    T: BehaviorFactory,
{
    // Update graph node with behavior telemetry
    let node: &mut egui_node_graph::Node<BehaviorNodeData<T>> = &mut graph.nodes[node_id];
    if let BehaviorTelemetry(state, Some(behavior), _) = telemetry {
        node.user_data.data = BehaviorNodeTemplate::Behavior(behavior.clone());
        node.user_data.state = Some(*state);
    }

    // Get node children
    let node_children: Vec<NodeId> = node
        .outputs
        .iter()
        .filter_map(|(_, output_id)| {
            graph
                .connections
                .iter()
                .find(|(input_id, rhs_output_id)| {
                    output_id == *rhs_output_id
                        && graph.inputs[*input_id].typ == BehaviorDataType::Flow
                })
                .and_then(|(input_id, _)| Some(graph.inputs[input_id].node))
        })
        .collect();

    // Zip and iterate over children
    let node_children = node_children.iter().cloned();
    let telemetry_children = telemetry.2.iter();
    for (node_child, behavior_child) in node_children.zip(telemetry_children) {
        behavior_telemerty_to_graph(graph, node_child, behavior_child);
    }
}

// For use with world.get_entity_component_reflect
fn _components_of_entity(
    world: &mut World,
    entity: Entity,
) -> Vec<(String, bevy::ecs::component::ComponentId, core::any::TypeId)> {
    let entity_ref = world.get_entity(entity).unwrap();
    let archetype = entity_ref.archetype();
    let mut components: Vec<_> = archetype
        .components()
        .filter_map(|component_id| {
            let info = world.components().get_info(component_id).unwrap();
            let name = pretty_type_name::pretty_type_name_str(info.name());
            Some((name, component_id, info.type_id().unwrap()))
        })
        .collect();
    components.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    components
}

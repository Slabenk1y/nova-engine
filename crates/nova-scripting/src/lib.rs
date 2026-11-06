//! nova-scripting: Visual node-based scripting and sandboxed text-based execution.

use nova_core::Id;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Visual node connection socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketType {
    Trigger,
    Number,
    String,
    Boolean,
    EntityRef,
}

/// Node definition in the visual scripting graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VisualNodeKind {
    OnPlayerTouch,
    OnTimerTick { interval_seconds: f32 },
    TeleportPlayer { target_x: f32, target_y: f32, target_z: f32 },
    KillPlayer,
    AddScore { amount: i64 },
    SetVisible { visible: bool },
    PlaySound { sound_id: String },
    CompareNumber { operator: String, operand: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualNode {
    pub id: Id,
    pub kind: VisualNodeKind,
    pub next_node_id: Option<Id>,
}

/// Visual Script Graph representing user gameplay logic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualScriptGraph {
    pub id: Id,
    pub name: String,
    pub nodes: HashMap<Id, VisualNode>,
    pub entry_nodes: Vec<Id>,
}

impl VisualScriptGraph {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Id::new(),
            name: name.into(),
            nodes: HashMap::new(),
            entry_nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, kind: VisualNodeKind, is_entry: bool) -> Id {
        let id = Id::new();
        self.nodes.insert(id, VisualNode { id, kind, next_node_id: None });
        if is_entry {
            self.entry_nodes.push(id);
        }
        id
    }

    pub fn connect(&mut self, from: Id, to: Id) {
        if let Some(node) = self.nodes.get_mut(&from) {
            node.next_node_id = Some(to);
        }
    }
}

/// Context passed to script runtime execution.
#[derive(Debug, Default, Clone)]
pub struct ScriptExecutionContext {
    pub player_score: i64,
    pub player_respawn_requested: bool,
    pub teleport_target: Option<[f32; 3]>,
    pub entity_visibility: HashMap<Id, bool>,
    pub sound_events: Vec<String>,
}

/// Execution error with sandboxing diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptError {
    MaxInstructionsExceeded,
    InvalidOperation(String),
}

/// Sandboxed execution engine for visual and text scripts.
pub struct ScriptRuntime {
    pub max_instructions_per_tick: usize,
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self {
            max_instructions_per_tick: 5000,
        }
    }
}

impl ScriptRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_visual_graph(
        &self,
        graph: &VisualScriptGraph,
        entry_node_id: Id,
        ctx: &mut ScriptExecutionContext,
    ) -> Result<(), ScriptError> {
        let mut current_id = Some(entry_node_id);
        let mut steps = 0;

        while let Some(id) = current_id {
            steps += 1;
            if steps > self.max_instructions_per_tick {
                return Err(ScriptError::MaxInstructionsExceeded);
            }

            let node = graph.nodes.get(&id).ok_or_else(|| {
                ScriptError::InvalidOperation(format!("Node {} not found in graph", id))
            })?;

            match &node.kind {
                VisualNodeKind::KillPlayer => {
                    ctx.player_respawn_requested = true;
                }
                VisualNodeKind::TeleportPlayer { target_x, target_y, target_z } => {
                    ctx.teleport_target = Some([*target_x, *target_y, *target_z]);
                }
                VisualNodeKind::AddScore { amount } => {
                    ctx.player_score += amount;
                }
                VisualNodeKind::PlaySound { sound_id } => {
                    ctx.sound_events.push(sound_id.clone());
                }
                _ => {}
            }

            current_id = node.next_node_id;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_script_execution() {
        let mut graph = VisualScriptGraph::new("Obby Lava Trigger");
        let entry = graph.add_node(VisualNodeKind::OnPlayerTouch, true);
        let kill = graph.add_node(VisualNodeKind::KillPlayer, false);
        let add_score = graph.add_node(VisualNodeKind::AddScore { amount: 50 }, false);

        graph.connect(entry, kill);
        graph.connect(kill, add_score);

        let runtime = ScriptRuntime::new();
        let mut ctx = ScriptExecutionContext::default();

        let result = runtime.execute_visual_graph(&graph, entry, &mut ctx);
        assert!(result.is_ok());
        assert!(ctx.player_respawn_requested);
        assert_eq!(ctx.player_score, 50);
    }
}

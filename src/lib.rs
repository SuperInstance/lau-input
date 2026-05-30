//! `lau-input` — Low-level input handling for the game.
//!
//! Provides action mapping, gesture recognition, and voice command detection.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Vec2
// ---------------------------------------------------------------------------

/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn magnitude(&self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn normalize(&self) -> Self {
        let m = self.magnitude();
        if m == 0.0 {
            Self::zero()
        } else {
            Self {
                x: self.x / m,
                y: self.y / m,
            }
        }
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

// ---------------------------------------------------------------------------
// GestureType
// ---------------------------------------------------------------------------

/// Recognised gesture types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GestureType {
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,
    Pinch,
    Spread,
    Tap,
    DoubleTap,
    LongPress,
    Circle,
}

// ---------------------------------------------------------------------------
// InputAction
// ---------------------------------------------------------------------------

/// Actions that can be produced by input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputAction {
    Move(Vec2),
    Jump,
    Interact,
    Build,
    Destroy,
    OpenInventory,
    OpenMap,
    Pause,
    Chat,
    VoiceActivate,
    Emote(u8),
    Scroll(f64),
    SelectSlot(u8),
    ToggleMode,
}

// ---------------------------------------------------------------------------
// InputTrigger
// ---------------------------------------------------------------------------

/// The physical / logical input that fires an action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputTrigger {
    KeyPress(char),
    KeyHold(char),
    MouseButton(u8),
    MouseClick(u8),
    GamepadButton(u8),
    GamepadAxis(u8, f64),
    Gesture(GestureType),
}

// ---------------------------------------------------------------------------
// InputBinding
// ---------------------------------------------------------------------------

/// A mapping from trigger to action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputBinding {
    pub action: InputAction,
    pub trigger: InputTrigger,
}

// ---------------------------------------------------------------------------
// InputState
// ---------------------------------------------------------------------------

/// Current snapshot of all input devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputState {
    pub keys_held: HashSet<char>,
    pub mouse_pos: Vec2,
    pub mouse_delta: Vec2,
    pub mouse_buttons: [bool; 3],
    pub scroll_delta: f64,
    pub gamepad_axes: [f64; 8],
    pub gamepad_buttons: [bool; 16],
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys_held: HashSet::new(),
            mouse_pos: Vec2::zero(),
            mouse_delta: Vec2::zero(),
            mouse_buttons: [false; 3],
            scroll_delta: 0.0,
            gamepad_axes: [0.0; 8],
            gamepad_buttons: [false; 16],
        }
    }

    pub fn press_key(&mut self, key: char) {
        self.keys_held.insert(key);
    }

    pub fn release_key(&mut self, key: char) {
        self.keys_held.remove(&key);
    }

    pub fn is_key_held(&self, key: char) -> bool {
        self.keys_held.contains(&key)
    }

    pub fn move_mouse(&mut self, x: f64, y: f64) {
        self.mouse_delta = Vec2::new(x - self.mouse_pos.x, y - self.mouse_pos.y);
        self.mouse_pos = Vec2::new(x, y);
    }

    pub fn press_mouse(&mut self, button: u8) {
        if let Some(b) = self.mouse_buttons.get_mut(button as usize) {
            *b = true;
        }
    }

    pub fn release_mouse(&mut self, button: u8) {
        if let Some(b) = self.mouse_buttons.get_mut(button as usize) {
            *b = false;
        }
    }

    pub fn scroll(&mut self, delta: f64) {
        self.scroll_delta = delta;
    }

    pub fn set_gamepad_axis(&mut self, axis: u8, value: f64) {
        if let Some(a) = self.gamepad_axes.get_mut(axis as usize) {
            *a = value;
        }
    }

    pub fn press_gamepad(&mut self, button: u8) {
        if let Some(b) = self.gamepad_buttons.get_mut(button as usize) {
            *b = true;
        }
    }
}

// ---------------------------------------------------------------------------
// InputMapper
// ---------------------------------------------------------------------------

/// Maps physical triggers to logical actions via bindings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputMapper {
    pub bindings: Vec<InputBinding>,
}

impl InputMapper {
    pub fn new() -> Self {
        Self { bindings: Vec::new() }
    }

    pub fn bind(&mut self, trigger: InputTrigger, action: InputAction) {
        self.bindings.push(InputBinding { action, trigger });
    }

    pub fn unbind(&mut self, trigger: &InputTrigger) {
        self.bindings.retain(|b| &b.trigger != trigger);
    }

    /// Evaluate all bindings against the current `state`.
    ///
    /// - `KeyPress` fires when the key is held (simulates "just pressed" in a
    ///   real engine you'd pair with a previous-frame snapshot).
    /// - `KeyHold` fires every frame the key is held.
    /// - `MouseClick`/`MouseButton` fire when the mouse button is down.
    /// - `GamepadButton` fires when the gamepad button is down.
    /// - `GamepadAxis(idx, threshold)` fires when `axis ≥ threshold`.
    /// - `Gesture` is not state-based and is never emitted by `process`
    ///   (gestures are detected externally and injected as actions).
    pub fn process(&self, state: &InputState, _dt: f64) -> Vec<InputAction> {
        let mut actions = Vec::new();

        // WASD movement synthesis — if any WASD key is held, produce a single Move.
        let mut dir = Vec2::zero();
        if state.is_key_held('w') {
            dir.y -= 1.0;
        }
        if state.is_key_held('s') {
            dir.y += 1.0;
        }
        if state.is_key_held('a') {
            dir.x -= 1.0;
        }
        if state.is_key_held('d') {
            dir.x += 1.0;
        }
        let wasd_active = dir.magnitude() > 0.0;
        if wasd_active {
            actions.push(InputAction::Move(dir.normalize()));
        }

        for binding in &self.bindings {
            let fired = match &binding.trigger {
                InputTrigger::KeyHold(c) => state.is_key_held(*c),
                InputTrigger::KeyPress(c) => state.is_key_held(*c),
                InputTrigger::MouseButton(b) | InputTrigger::MouseClick(b) => {
                    state.mouse_buttons.get(*b as usize).copied().unwrap_or(false)
                }
                InputTrigger::GamepadButton(b) => {
                    state.gamepad_buttons.get(*b as usize).copied().unwrap_or(false)
                }
                InputTrigger::GamepadAxis(idx, threshold) => state
                    .gamepad_axes
                    .get(*idx as usize)
                    .map(|&v| v >= *threshold)
                    .unwrap_or(false),
                InputTrigger::Gesture(_) => false, // handled externally
            };

            if fired {
                // Skip Move bindings when WASD already produced one.
                if matches!(binding.action, InputAction::Move(_)) && wasd_active {
                    continue;
                }
                actions.push(binding.action.clone());
            }
        }

        if state.scroll_delta.abs() > 0.0 {
            actions.push(InputAction::Scroll(state.scroll_delta));
        }

        actions
    }

    /// Convenience: return the default WASD + common bindings.
    pub fn default_bindings() -> Self {
        let mut m = Self::new();
        m.bind(InputTrigger::KeyHold(' '), InputAction::Jump);
        m.bind(InputTrigger::KeyPress('e'), InputAction::Interact);
        m.bind(InputTrigger::KeyPress('\t'), InputAction::OpenInventory);
        m.bind(InputTrigger::KeyPress('m'), InputAction::OpenMap);
        m.bind(InputTrigger::KeyPress('\u{1b}'), InputAction::Pause); // ESC
        m
    }
}

// ---------------------------------------------------------------------------
// VoiceCommand
// ---------------------------------------------------------------------------

/// A recognised voice command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    pub text: String,
    pub confidence: f64,
    pub timestamp: u64,
}

impl VoiceCommand {
    pub fn is_valid(&self) -> bool {
        self.confidence > 0.7
    }
}

// ---------------------------------------------------------------------------
// VoiceCommandDetector
// ---------------------------------------------------------------------------

/// Maps voice phrases to game actions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoiceCommandDetector {
    pub commands: HashMap<String, InputAction>,
}

impl VoiceCommandDetector {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, phrase: &str, action: InputAction) {
        self.commands.insert(phrase.to_lowercase(), action);
    }

    /// Attempt to match a voice command to a registered phrase.
    ///
    /// Uses a simple normalised-Levenshtein-style fuzzy match: we check
    /// containment and simple prefix/suffix heuristics. Good enough for a
    /// game prototype — replace with a real fuzzy library in production.
    pub fn detect(&self, voice: &VoiceCommand) -> Option<InputAction> {
        if !voice.is_valid() {
            return None;
        }
        let spoken = voice.text.to_lowercase();
        let spoken_words: Vec<&str> = spoken.split_whitespace().collect();

        let mut best_score: f64 = 0.0;
        let mut best_action: Option<InputAction> = None;

        for (phrase, action) in &self.commands {
            let phrase_words: Vec<&str> = phrase.split_whitespace().collect();
            let phrase_lower = phrase.to_lowercase();

            // Exact match
            if spoken == phrase_lower {
                return Some(action.clone());
            }

            // Containment
            if spoken.contains(&phrase_lower) || phrase_lower.contains(&spoken) {
                let score = 0.9;
                if score > best_score {
                    best_score = score;
                    best_action = Some(action.clone());
                }
                continue;
            }

            // Word overlap ratio
            let overlap = phrase_words
                .iter()
                .filter(|w| spoken_words.iter().any(|s| s.contains(*w) || w.contains(s)))
                .count() as f64;
            let total = (phrase_words.len().max(spoken_words.len())) as f64;
            let score = overlap / total;
            if score > best_score && score >= 0.5 {
                best_score = score;
                best_action = Some(action.clone());
            }
        }

        best_action
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Vec2 ---
    #[test]
    fn vec2_new() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn vec2_zero() {
        let v = Vec2::zero();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
    }

    #[test]
    fn vec2_magnitude() {
        let v = Vec2::new(3.0, 4.0);
        assert!((v.magnitude() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn vec2_normalize() {
        let v = Vec2::new(3.0, 4.0).normalize();
        assert!((v.magnitude() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn vec2_normalize_zero() {
        let v = Vec2::zero().normalize();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
    }

    #[test]
    fn vec2_dot() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert!((a.dot(&b) - 11.0).abs() < 1e-9);
    }

    // --- InputState ---
    #[test]
    fn input_state_new_defaults() {
        let s = InputState::new();
        assert!(s.keys_held.is_empty());
        assert_eq!(s.mouse_pos, Vec2::zero());
        assert_eq!(s.scroll_delta, 0.0);
        assert_eq!(s.mouse_buttons, [false; 3]);
        assert_eq!(s.gamepad_axes, [0.0; 8]);
        assert_eq!(s.gamepad_buttons, [false; 16]);
    }

    #[test]
    fn press_release_key() {
        let mut s = InputState::new();
        s.press_key('w');
        assert!(s.is_key_held('w'));
        s.release_key('w');
        assert!(!s.is_key_held('w'));
    }

    #[test]
    fn mouse_move_updates_delta() {
        let mut s = InputState::new();
        s.move_mouse(10.0, 20.0);
        assert_eq!(s.mouse_pos, Vec2::new(10.0, 20.0));
        s.move_mouse(15.0, 25.0);
        assert_eq!(s.mouse_delta, Vec2::new(5.0, 5.0));
    }

    #[test]
    fn mouse_button_press_release() {
        let mut s = InputState::new();
        s.press_mouse(0);
        assert!(s.mouse_buttons[0]);
        s.release_mouse(0);
        assert!(!s.mouse_buttons[0]);
    }

    #[test]
    fn scroll_accumulates() {
        let mut s = InputState::new();
        s.scroll(3.5);
        assert_eq!(s.scroll_delta, 3.5);
    }

    #[test]
    fn gamepad_axis_and_button() {
        let mut s = InputState::new();
        s.set_gamepad_axis(0, 0.8);
        assert!((s.gamepad_axes[0] - 0.8).abs() < 1e-9);
        s.press_gamepad(5);
        assert!(s.gamepad_buttons[5]);
    }

    // --- InputMapper ---
    #[test]
    fn bind_and_process_jump() {
        let mut mapper = InputMapper::new();
        mapper.bind(InputTrigger::KeyHold(' '), InputAction::Jump);
        let mut state = InputState::new();
        state.press_key(' ');
        let actions = mapper.process(&state, 0.016);
        assert!(actions.contains(&InputAction::Jump));
    }

    #[test]
    fn unbind_removes_binding() {
        let mut mapper = InputMapper::new();
        mapper.bind(InputTrigger::KeyHold('x'), InputAction::Chat);
        mapper.unbind(&InputTrigger::KeyHold('x'));
        assert!(mapper.bindings.is_empty());
    }

    #[test]
    fn wasd_movement() {
        let mapper = InputMapper::new();
        let mut state = InputState::new();
        state.press_key('w');
        state.press_key('d');
        let actions = mapper.process(&state, 0.016);
        let move_action = actions.iter().find_map(|a| match a {
            InputAction::Move(v) => Some(*v),
            _ => None,
        });
        assert!(move_action.is_some());
        let v = move_action.unwrap();
        assert!(v.x > 0.0 && v.y < 0.0); // right + up
    }

    #[test]
    fn default_bindings_pause_on_esc() {
        let mapper = InputMapper::default_bindings();
        let mut state = InputState::new();
        state.press_key('\u{1b}');
        let actions = mapper.process(&state, 0.016);
        assert!(actions.contains(&InputAction::Pause));
    }

    #[test]
    fn default_bindings_interact_on_e() {
        let mapper = InputMapper::default_bindings();
        let mut state = InputState::new();
        state.press_key('e');
        let actions = mapper.process(&state, 0.016);
        assert!(actions.contains(&InputAction::Interact));
    }

    #[test]
    fn gamepad_button_binding() {
        let mut mapper = InputMapper::new();
        mapper.bind(InputTrigger::GamepadButton(0), InputAction::Jump);
        let mut state = InputState::new();
        state.press_gamepad(0);
        let actions = mapper.process(&state, 0.016);
        assert!(actions.contains(&InputAction::Jump));
    }

    #[test]
    fn gamepad_axis_binding() {
        let mut mapper = InputMapper::new();
        mapper.bind(InputTrigger::GamepadAxis(1, 0.5), InputAction::Build);
        let mut state = InputState::new();
        state.set_gamepad_axis(1, 0.8);
        let actions = mapper.process(&state, 0.016);
        assert!(actions.contains(&InputAction::Build));
    }

    #[test]
    fn mouse_button_binding() {
        let mut mapper = InputMapper::new();
        mapper.bind(InputTrigger::MouseButton(0), InputAction::Destroy);
        let mut state = InputState::new();
        state.press_mouse(0);
        let actions = mapper.process(&state, 0.016);
        assert!(actions.contains(&InputAction::Destroy));
    }

    // --- VoiceCommand ---
    #[test]
    fn voice_command_valid() {
        let vc = VoiceCommand {
            text: "jump".into(),
            confidence: 0.85,
            timestamp: 1,
        };
        assert!(vc.is_valid());
    }

    #[test]
    fn voice_command_invalid_low_confidence() {
        let vc = VoiceCommand {
            text: "jump".into(),
            confidence: 0.5,
            timestamp: 1,
        };
        assert!(!vc.is_valid());
    }

    // --- VoiceCommandDetector ---
    #[test]
    fn voice_detect_exact_match() {
        let mut det = VoiceCommandDetector::new();
        det.register("jump", InputAction::Jump);
        let vc = VoiceCommand {
            text: "jump".into(),
            confidence: 0.9,
            timestamp: 1,
        };
        assert_eq!(det.detect(&vc), Some(InputAction::Jump));
    }

    #[test]
    fn voice_detect_rejects_low_confidence() {
        let mut det = VoiceCommandDetector::new();
        det.register("jump", InputAction::Jump);
        let vc = VoiceCommand {
            text: "jump".into(),
            confidence: 0.3,
            timestamp: 1,
        };
        assert_eq!(det.detect(&vc), None);
    }

    #[test]
    fn voice_detect_no_match() {
        let det = VoiceCommandDetector::new();
        let vc = VoiceCommand {
            text: "fly".into(),
            confidence: 0.95,
            timestamp: 1,
        };
        assert_eq!(det.detect(&vc), None);
    }

    #[test]
    fn voice_detect_fuzzy_containment() {
        let mut det = VoiceCommandDetector::new();
        det.register("open inventory", InputAction::OpenInventory);
        let vc = VoiceCommand {
            text: "please open inventory now".into(),
            confidence: 0.85,
            timestamp: 1,
        };
        assert_eq!(det.detect(&vc), Some(InputAction::OpenInventory));
    }

    // --- Serde round-trip ---
    #[test]
    fn serde_input_action() {
        let action = InputAction::Emote(3);
        let json = serde_json::to_string(&action).unwrap();
        let decoded: InputAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn serde_input_state() {
        let mut s = InputState::new();
        s.press_key('a');
        s.move_mouse(5.0, 10.0);
        let json = serde_json::to_string(&s).unwrap();
        let decoded: InputState = serde_json::from_str(&json).unwrap();
        assert!(decoded.is_key_held('a'));
        assert_eq!(decoded.mouse_pos.x, 5.0);
    }

    #[test]
    fn scroll_action_emitted() {
        let mapper = InputMapper::new();
        let mut state = InputState::new();
        state.scroll(2.0);
        let actions = mapper.process(&state, 0.016);
        assert!(actions.contains(&InputAction::Scroll(2.0)));
    }
}

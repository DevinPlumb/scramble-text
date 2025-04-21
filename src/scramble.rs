use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub enum RangeOrCharCodes {
    Range(i32, i32),
    Codes(Vec<i32>),
}

#[derive(Clone)]
pub enum ScrambleControl {
    Char(char),
    Number(i32),
    Null,
}

impl fmt::Display for ScrambleControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScrambleControl::Char(c) => write!(f, "{}", c),
            ScrambleControl::Number(n) => write!(f, "{}", n),
            ScrambleControl::Null => write!(f, ""),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UseScrambleProps {
    /// When true, the animation will play automatically when a text input is first provided.
    pub play_on_mount: Option<bool>,

    /// Text input to be scrambled
    pub text: String,

    /// 0-1 range that determines the scramble speed. A speed of 1 will redraw 60 times a second.
    /// A speed of 0 will pause the animation.
    #[serde(default = "default_speed")]
    pub speed: f32,

    /// The controller will move forward along the text input and scramble more characters,
    /// at a pace of `tick` frames.
    #[serde(default = "default_tick")]
    pub tick: i32,

    /// Number of characters to step forward on every tick.
    #[serde(default = "default_step")]
    pub step: i32,

    /// Chance of scrambling a character (0-1)
    #[serde(default = "default_chance")]
    pub chance: f32,

    /// Number of characters to randomly scramble
    #[serde(default = "default_seed")]
    pub seed: i32,

    /// How many times to scramble each character
    #[serde(default = "default_scramble")]
    pub scramble: i32,

    /// Characters to avoid scrambling
    #[serde(default = "default_ignore")]
    pub ignore: Vec<String>,

    /// Unicode character range for scrambler
    #[serde(default = "default_range")]
    pub range: RangeOrCharCodes,

    /// When true, enables overdrive mode
    #[serde(default)]
    pub overdrive: bool,

    /// When true, animation starts from empty string
    #[serde(default)]
    pub overflow: bool,

    /// When true, enables hover-to-replay functionality
    #[serde(default)]
    pub hover_replay: bool,
}

fn default_speed() -> f32 {
    1.0
}
fn default_tick() -> i32 {
    1
}
fn default_step() -> i32 {
    1
}
fn default_chance() -> f32 {
    1.0
}
fn default_seed() -> i32 {
    1
}
fn default_scramble() -> i32 {
    1
}
fn default_ignore() -> Vec<String> {
    vec![" ".to_string()]
}
fn default_range() -> RangeOrCharCodes {
    RangeOrCharCodes::Range(65, 125)
}

impl UseScrambleProps {
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.speed) {
            return Err("Speed must be between 0 and 1".to_string());
        }
        if self.tick <= 0 {
            return Err("Tick must be greater than 0".to_string());
        }
        if self.step <= 0 {
            return Err("Step must be greater than 0".to_string());
        }
        if !(0.0..=1.0).contains(&self.chance) {
            return Err("Chance must be between 0 and 1".to_string());
        }
        if self.seed < 0 {
            return Err("Seed must be greater than or equal to 0".to_string());
        }
        if self.scramble < 0 {
            return Err("Scramble must be greater than or equal to 0".to_string());
        }

        match &self.range {
            RangeOrCharCodes::Range(min, max) => {
                if *min < 0 || *max < *min {
                    return Err("Invalid range values".to_string());
                }
                if !char::from_u32(*min as u32).is_some() || !char::from_u32(*max as u32).is_some()
                {
                    return Err("Range values must be valid Unicode scalar values".to_string());
                }
            }
            RangeOrCharCodes::Codes(codes) => {
                if codes.is_empty() {
                    return Err("Codes vector cannot be empty".to_string());
                }
                if codes
                    .iter()
                    .any(|&code| !char::from_u32(code as u32).is_some())
                {
                    return Err("All codes must be valid Unicode scalar values".to_string());
                }
            }
        }

        Ok(())
    }
}

impl Default for UseScrambleProps {
    fn default() -> Self {
        UseScrambleProps {
            play_on_mount: None,
            text: String::new(),
            speed: default_speed(),
            tick: default_tick(),
            step: default_step(),
            chance: default_chance(),
            seed: default_seed(),
            scramble: default_scramble(),
            ignore: default_ignore(),
            range: default_range(),
            overdrive: false,
            overflow: false,
            hover_replay: false,
        }
    }
}

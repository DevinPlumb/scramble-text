use rand::Rng;
use scramble_text::random_int;
use scramble_text::scramble::ScrambleControl;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

fn main() {
    // This is a library crate, main is just for documentation
    println!("This is a WebAssembly library. Please use it from JavaScript.");
}

#[derive(Clone)]
enum RangeOrCharCodes {
    Range(i32, i32),
    Codes(Vec<i32>),
}

impl RangeOrCharCodes {
    fn get_random_value(&self) -> Option<i32> {
        match self {
            RangeOrCharCodes::Range(min, max) => Some(random_int(*min, *max)),
            RangeOrCharCodes::Codes(codes) if !codes.is_empty() => {
                let idx = random_int(0, codes.len() as i32 - 1);
                codes.get(idx as usize).copied()
            }
            _ => None,
        }
    }
}

fn get_random_char(range: &RangeOrCharCodes) -> String {
    range
        .get_random_value()
        .and_then(|code| char::from_u32(code as u32))
        .map(|c| c.to_string())
        .unwrap_or_default()
}

#[derive(Default)]
pub struct UseScrambleProps {
    /// When true, the animation will play automatically when a text input is first provided.
    /// When false, animation must be triggered manually.
    pub play_on_mount: Option<bool>,

    /// Optional text input to be scrambled
    pub text: Option<String>,

    /// 0-1 range that determines the scramble speed. A speed of 1 will redraw 60 times a second.
    /// A speed of 0 will pause the animation.
    /// Must be between 0 and 1.
    pub speed: Option<f32>,

    /// The controller will move forward along the text input and scramble more characters,
    /// at a pace of `tick` frames. Combined with the `speed` prop, you can control the animation rate.
    /// Must be greater than 0.
    pub tick: Option<i32>,

    /// Number of characters to step forward on every tick.
    /// Must be greater than 0.
    pub step: Option<i32>,

    /// Chance of scrambling a character, range from 0 to 1, 0 being no chance, and 1 being 100% chance.
    /// Must be between 0 and 1.
    pub chance: Option<f32>,

    /// Number of characters to randomly scramble at random text positions.
    /// Must be greater than or equal to 0.
    pub seed: Option<i32>,

    /// How many times to scramble each character.
    /// Must be greater than or equal to 0.
    pub scramble: Option<i32>,

    /// Characters to avoid scrambling. Each string should be a single character.
    pub ignore: Option<Vec<String>>,

    /// Unicode character range for scrambler.
    /// For Range(min, max): min and max must be valid Unicode scalar values.
    /// For Codes(vec): each value must be a valid Unicode scalar value.
    pub range: Option<RangeOrCharCodes>,

    /// When true, enables overdrive mode which uses a specific Unicode character for animation.
    pub overdrive: Option<bool>,

    /// When true, animation always starts from an empty string.
    /// When false, animation starts with the full text and scrambles it.
    pub overflow: Option<bool>,

    /// Callback invoked when animation starts drawing
    pub on_animation_start: Option<Box<dyn Fn()>>,

    /// Callback invoked when animation finishes
    pub on_animation_end: Option<Box<dyn Fn()>>,

    /// Callback invoked on each animation frame with the current text state
    pub on_animation_frame: Option<Box<dyn Fn(String)>>,
}

impl UseScrambleProps {
    fn validate(&self) -> Result<(), String> {
        if let Some(speed) = self.speed {
            if !(0.0..=1.0).contains(&speed) {
                return Err("Speed must be between 0 and 1".to_string());
            }
        }

        if let Some(tick) = self.tick {
            if tick <= 0 {
                return Err("Tick must be greater than 0".to_string());
            }
        }

        if let Some(step) = self.step {
            if step <= 0 {
                return Err("Step must be greater than 0".to_string());
            }
        }

        if let Some(chance) = self.chance {
            if !(0.0..=1.0).contains(&chance) {
                return Err("Chance must be between 0 and 1".to_string());
            }
        }

        if let Some(seed) = self.seed {
            if seed < 0 {
                return Err("Seed must be greater than or equal to 0".to_string());
            }
        }

        if let Some(scramble) = self.scramble {
            if scramble < 0 {
                return Err("Scramble must be greater than or equal to 0".to_string());
            }
        }

        if let Some(range) = &self.range {
            match range {
                RangeOrCharCodes::Range(min, max) => {
                    if *min < 0 || *max < *min {
                        return Err("Invalid range values".to_string());
                    }
                    if !char::from_u32(*min as u32).is_some()
                        || !char::from_u32(*max as u32).is_some()
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
        }

        Ok(())
    }

    pub fn use_scramble(
        &self,
    ) -> Result<
        (
            Rc<RefCell<Option<web_sys::Element>>>,
            Box<dyn Fn()>,
            Box<dyn Fn()>,
        ),
        String,
    > {
        self.validate()?;
        let text = self.text.clone().unwrap_or_default();
        let speed = self.speed.unwrap_or(1.0) as f64;
        let seed = self.seed.unwrap_or(1);
        let step = self.step.unwrap_or(1);
        let tick = self.tick.unwrap_or(1);
        let scramble = self.scramble.unwrap_or(1);
        let chance = self.chance.unwrap_or(1.0) as f64;
        let overflow = self.overflow.unwrap_or(true);
        let range = self
            .range
            .clone()
            .unwrap_or(RangeOrCharCodes::Range(65, 125));
        let overdrive = self.overdrive.unwrap_or(true);
        let ignore = self.ignore.clone().unwrap_or_else(|| vec![" ".to_string()]);

        let prefers_reduced_motion = web_sys::window()
            .and_then(|window| window.match_media("(prefers-reduced-motion: reduce)").ok())
            .flatten()
            .map(|media| media.matches())
            .unwrap_or(false);

        let (step, chance, overdrive) = if prefers_reduced_motion {
            (text.len() as i32, 0.0, false)
        } else {
            (step, chance, overdrive)
        };

        let fps_interval = (1000.0 / (60.0 * speed)) as f64;

        // Text node ref
        let node_ref: Rc<RefCell<Option<web_sys::Element>>> = Rc::new(RefCell::new(None));

        // Animation frame request
        let raf_ref = Rc::new(RefCell::new(0));

        // Compute
        let elapsed_ref = Rc::new(RefCell::new(0.0));

        // Scramble step
        let step_ref = Rc::new(RefCell::new(0));

        // Current character index ref
        let scramble_index_ref = Rc::new(RefCell::new(0));

        // Scramble controller
        let control_ref: Rc<RefCell<Vec<Option<ScrambleControl>>>> =
            Rc::new(RefCell::new(Vec::new()));

        // Overdrive control index
        let overdrive_ref = Rc::new(RefCell::new(0));

        let ignore = ignore.clone();
        let set_if_not_ignored =
            Rc::new(move |value: &ScrambleControl, replace: ScrambleControl| {
                if ignore.contains(&value.to_string()) {
                    value.clone()
                } else {
                    replace
                }
            });

        // pick random character ahead in the string, and add them to the randomizer
        let seed_forward = {
            let scramble_index = scramble_index_ref.clone();
            let control = control_ref.clone();
            let text = text.clone();
            let set_if_not_ignored = set_if_not_ignored.clone();

            move || {
                let scramble_index = *scramble_index.borrow();
                let control_len = control.borrow().len();

                if scramble_index >= text.len() || control_len == 0 {
                    return;
                }

                for _ in 0..seed {
                    let index = random_int(scramble_index as i32, (control_len - 1) as i32);
                    if index < 0 || index as usize >= control_len {
                        continue;
                    }

                    let mut control = control.borrow_mut();
                    if let Some(value) = control[index as usize].as_ref() {
                        if !matches!(value, ScrambleControl::Number(_)) {
                            control[index as usize] = Some((set_if_not_ignored)(
                                value,
                                ScrambleControl::Number(
                                    if random_int(0, 10) >= ((1.0 - chance) * 10.0) as i32 {
                                        scramble.max(seed)
                                    } else {
                                        0
                                    },
                                ),
                            ));
                        }
                    }
                }
            }
        };

        // add `step` characters to the randomizer, and increase the scrambleIndexRef pointer
        let step_forward = {
            let scramble_index = scramble_index_ref.clone();
            let control = control_ref.clone();
            let text = text.clone();
            let set_if_not_ignored = set_if_not_ignored.clone();

            move || {
                let current_index = *scramble_index.borrow();
                if current_index >= text.len() {
                    return;
                }

                for _ in 0..step {
                    if current_index >= text.len() {
                        break;
                    }

                    let should_scramble = random_int(0, 10) >= ((1.0 - chance) * 10.0) as i32;

                    if let Some(c) = text.chars().nth(current_index) {
                        let mut control = control.borrow_mut();
                        if control.len() <= current_index {
                            control.resize(current_index + 1, None);
                        }

                        control[current_index] = Some((set_if_not_ignored)(
                            &ScrambleControl::Char(c),
                            ScrambleControl::Number(if should_scramble {
                                scramble + random_int(0, (scramble as f32 / 2.0).ceil() as i32)
                            } else {
                                0
                            }),
                        ));
                    }

                    *scramble_index.borrow_mut() += 1;
                }
            }
        };

        let resize_control = {
            let control = control_ref.clone();
            let text = text.clone();

            move || {
                let mut control = control.borrow_mut();
                if text.len() < control.len() {
                    control.truncate(text.len());
                } else if control.len() < text.len() {
                    control.resize(text.len(), None);
                }
            }
        };

        let overdrive_fn = {
            let overdrive = overdrive.clone();
            let control = control_ref.clone();
            let text = text.clone();
            let overdrive_index = overdrive_ref.clone();

            move || {
                if !overdrive {
                    return;
                }

                for _ in 0..step {
                    let max = control.borrow().len().max(text.len());
                    if *overdrive_index.borrow() < max {
                        let current_index = *overdrive_index.borrow();
                        let mut control = control.borrow_mut();
                        control[current_index] = Some((set_if_not_ignored)(
                            &ScrambleControl::Char(
                                text.chars().nth(current_index).unwrap_or_default(),
                            ),
                            ScrambleControl::Char(
                                char::from_u32(match overdrive {
                                    true => 95,
                                    false => 0,
                                    _ => overdrive as u32,
                                })
                                .unwrap_or('_'),
                            ),
                        ));
                        *overdrive_index.borrow_mut() += 1;
                    }
                }
            }
        };

        let on_tick = {
            let step_forward = step_forward.clone();
            let resize_control = resize_control.clone();
            let seed_forward = seed_forward.clone();

            move || {
                step_forward();
                resize_control();
                seed_forward();
            }
        };

        let draw = {
            let node_ref = node_ref.clone();
            let control = control_ref.clone();
            let text = text.clone();
            let scramble_index = scramble_index_ref.clone();
            let step_ref = step_ref.clone();
            let range = range.clone();

            move || {
                if node_ref.borrow().is_none() {
                    return;
                }

                let mut result = String::new();
                let scramble_index = *scramble_index.borrow();
                {
                    let mut control = control.borrow_mut();
                    for i in 0..control.len() {
                        match &control[i] {
                            Some(ScrambleControl::Number(n)) if *n > 0 => {
                                result.push_str(&get_random_char(&range));

                                if i <= scramble_index {
                                    if let Some(ScrambleControl::Number(n)) = control[i] {
                                        control[i] = Some(ScrambleControl::Number(n - 1));
                                    }
                                }
                            }

                            Some(ScrambleControl::Char(c))
                                if i >= text.len() || i >= scramble_index =>
                            {
                                result.push(*c);
                            }

                            Some(ScrambleControl::Char(c)) if i < scramble_index => {
                                if let Some(text_char) = text.chars().nth(i) {
                                    if text_char == *c {
                                        result.push(*c);
                                    } else {
                                        result.push(' ');
                                    }
                                }
                            }

                            Some(ScrambleControl::Number(0)) if i < text.len() => {
                                if let Some(c) = text.chars().nth(i) {
                                    result.push(c);
                                    control[i] = Some(ScrambleControl::Char(c));
                                }
                            }

                            _ => result.push(' '),
                        }
                    }

                    if result == text {
                        control.truncate(text.len());
                    }
                }

                if let Some(node) = node_ref.borrow().as_ref() {
                    node.set_inner_html(&result);
                }

                *step_ref.borrow_mut() += 1;
            }
        };

        let animate = {
            let speed = speed.clone();
            let overdrive_fn = overdrive_fn.clone();
            let elapsed = elapsed_ref.clone();
            let step_ref = step_ref.clone();
            let tick = tick.clone();
            let on_tick = on_tick.clone();
            let draw = draw.clone();
            let fps_interval = fps_interval.clone();

            Closure::wrap(Box::new(move |time: f64| {
                if speed == 0.0 {
                    return;
                }

                overdrive_fn();

                let time_elapsed = time - *elapsed.borrow();
                if time_elapsed > fps_interval {
                    *elapsed.borrow_mut() = time;

                    if *step_ref.borrow() % tick == 0 {
                        on_tick();
                    }

                    draw();
                }
            }) as Box<dyn FnMut(f64)>)
        };

        let reset = {
            let step_ref = step_ref.clone();
            let scramble_index = scramble_index_ref.clone();
            let overdrive_index = overdrive_ref.clone();
            let control = control_ref.clone();
            let text = text.clone();
            let overflow = overflow.clone();

            move || {
                *step_ref.borrow_mut() = 0;
                *scramble_index.borrow_mut() = 0;
                *overdrive_index.borrow_mut() = 0;

                if !overflow {
                    *control.borrow_mut() = vec![None; text.len()];
                }
            }
        };

        let raf_ref_for_play = raf_ref.clone();
        let play = Box::new(move || {
            reset();

            let window = web_sys::window().expect("no global window exists");

            // Cancel any existing animation frame
            if *raf_ref_for_play.borrow() != 0 {
                let _ = window.cancel_animation_frame(*raf_ref_for_play.borrow());
                *raf_ref_for_play.borrow_mut() = 0;
            }

            // Start new animation
            if let Ok(id) = window.request_animation_frame(animate.as_ref().unchecked_ref()) {
                *raf_ref_for_play.borrow_mut() = id;
            }
        }) as Box<dyn Fn()>;

        // Create cleanup function
        let cleanup = {
            let raf_ref = raf_ref.clone();

            Box::new(move || {
                if let Some(window) = web_sys::window() {
                    if *raf_ref.borrow() != 0 {
                        let _ = window.cancel_animation_frame(*raf_ref.borrow());
                        *raf_ref.borrow_mut() = 0;
                    }
                }
            }) as Box<dyn Fn()>
        };

        let node_ref_clone = node_ref.clone();

        Ok((node_ref_clone, play, cleanup))
    }
}

pub fn draw() {
    let window = web_sys::window().expect("no global window exists");
    let document = window.document().expect("no document exists");

    if let Some(element) = document.get_element_by_id("scramble-text") {
        let mut rng = rand::thread_rng();
        let text = element.text_content().unwrap_or_default();
        let scrambled: String = text
            .chars()
            .map(|c| {
                if c.is_whitespace() {
                    c
                } else {
                    // Generate a random ASCII character between '!' and '~'
                    char::from_u32(rng.gen_range(33..127)).unwrap_or(c)
                }
            })
            .collect();
        element.set_text_content(Some(&scrambled));
    }
}

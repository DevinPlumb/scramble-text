use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Element;

pub mod scramble;
pub use scramble::*;

#[wasm_bindgen]
pub fn random_int(min: i32, max: i32) -> i32 {
    rand::thread_rng().gen_range(min..=max)
}

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
}

fn get_random_char(range: &RangeOrCharCodes) -> char {
    match range {
        RangeOrCharCodes::Range(min, max) => {
            let code = random_int(*min, *max);
            char::from_u32(code as u32).unwrap_or('_')
        }
        RangeOrCharCodes::Codes(codes) if !codes.is_empty() => {
            let idx = random_int(0, (codes.len() - 1) as i32);
            char::from_u32(codes[idx as usize] as u32).unwrap_or('_')
        }
        _ => '_',
    }
}

// Re-export the main types with wasm-bindgen
#[wasm_bindgen]
pub struct ScrambleText {
    element: Element,
    props: UseScrambleProps,
    animation_frame_id: i32,
    animation_closure: Option<Closure<dyn FnMut(f64)>>,
    on_animation_start: Option<js_sys::Function>,
    on_animation_end: Option<js_sys::Function>,
    on_animation_frame: Option<js_sys::Function>,
    frame_count: i32,
    scramble_counts: Vec<i32>,
}

#[wasm_bindgen]
impl ScrambleText {
    #[wasm_bindgen(constructor)]
    pub fn new(element: Element, props: JsValue) -> Result<ScrambleText, JsError> {
        let props: UseScrambleProps = serde_wasm_bindgen::from_value(props)?;
        props.validate().map_err(|e| JsError::new(&e))?;

        Ok(ScrambleText {
            element,
            props: props.clone(),
            animation_frame_id: 0,
            animation_closure: None,
            on_animation_start: None,
            on_animation_end: None,
            on_animation_frame: None,
            frame_count: 0,
            scramble_counts: vec![props.scramble; props.text.len()],
        })
    }

    #[wasm_bindgen]
    pub fn set_on_animation_start(&mut self, callback: js_sys::Function) {
        self.on_animation_start = Some(callback);
    }

    #[wasm_bindgen]
    pub fn set_on_animation_end(&mut self, callback: js_sys::Function) {
        self.on_animation_end = Some(callback);
    }

    #[wasm_bindgen]
    pub fn set_on_animation_frame(&mut self, callback: js_sys::Function) {
        self.on_animation_frame = Some(callback);
    }

    pub fn start(&mut self) -> Result<(), JsError> {
        // Clean up any existing animation
        self.stop()?;

        let window = web_sys::window().ok_or_else(|| JsError::new("No window found"))?;

        // Call the start callback if it exists
        if let Some(callback) = &self.on_animation_start {
            let this = JsValue::null();
            let _ = callback.call0(&this);
        }

        // Reset animation state
        self.frame_count = 0;
        self.scramble_counts = vec![self.props.scramble; self.props.text.len()];

        // Create the animation closure
        let element = self.element.clone();
        let text = self.props.text.clone();
        let ignore = self.props.ignore.clone();
        let range = self.props.range.clone();
        let tick = self.props.tick;
        let step = self.props.step;
        let chance = self.props.chance;
        let overdrive = self.props.overdrive;
        let on_frame = self.on_animation_frame.clone();
        let scramble_counts = Rc::new(RefCell::new(self.scramble_counts.clone()));
        let frame_count = Rc::new(RefCell::new(self.frame_count));
        let animation_id = Rc::new(RefCell::new(0));
        let animation_id_clone = animation_id.clone();

        let animation_closure = Closure::wrap(Box::new(move |_time: f64| {
            let mut current_text = String::with_capacity(text.len());
            let mut rng = rand::thread_rng();

            // Update frame count
            *frame_count.borrow_mut() += 1;
            let current_frame = *frame_count.borrow();

            // On each tick, decrease scramble counts for some characters
            if current_frame % tick == 0 {
                let mut counts = scramble_counts.borrow_mut();
                for i in 0..counts.len() {
                    if rng.gen::<f32>() <= chance {
                        if let Some(count) = counts.get_mut(i) {
                            *count = count.saturating_sub(1);
                        }
                    }
                    // Break if we've processed enough characters for this step
                    if i >= step as usize - 1 {
                        break;
                    }
                }
            }

            // Build the current frame's text
            for (i, ch) in text.chars().enumerate() {
                let counts = scramble_counts.borrow();
                if i < counts.len() && counts[i] > 0 {
                    // Character is still being scrambled
                    if overdrive {
                        current_text.push('_');
                    } else if ignore.contains(&ch.to_string()) {
                        current_text.push(ch);
                    } else {
                        current_text.push(get_random_char(&range));
                    }
                } else {
                    // Character has finished scrambling
                    current_text.push(ch);
                }
            }

            // Update the DOM
            element.set_text_content(Some(&current_text));

            // Call the frame callback if it exists
            if let Some(callback) = &on_frame {
                let this = JsValue::null();
                let text_js = JsValue::from_str(&current_text);
                let _ = callback.call1(&this, &text_js);
            }

            // Stop the interval if animation is complete
            if !scramble_counts.borrow().iter().any(|&count| count > 0) {
                if let Some(window) = web_sys::window() {
                    let id = *animation_id_clone.borrow();
                    if id != 0 {
                        window.clear_interval_with_handle(id);
                        *animation_id_clone.borrow_mut() = 0;
                    }
                }
            }
        }) as Box<dyn FnMut(f64)>);

        // Start the animation with setInterval
        let speed = self.props.speed;
        let interval = if speed == 0.0 {
            0
        } else {
            (1000.0 / (60.0 * speed as f64)) as i32
        };

        let id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                animation_closure.as_ref().unchecked_ref(),
                interval,
            )
            .map_err(|_| JsError::new("Failed to start animation interval"))?;

        // Store the interval ID
        *animation_id.borrow_mut() = id;
        self.animation_frame_id = id;

        // Store the closure for cleanup
        self.animation_closure = Some(animation_closure);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), JsError> {
        if let Some(window) = web_sys::window() {
            if self.animation_frame_id != 0 {
                window.clear_interval_with_handle(self.animation_frame_id);
                self.animation_frame_id = 0;

                // Call the end callback if it exists
                if let Some(callback) = &self.on_animation_end {
                    let this = JsValue::null();
                    let _ = callback.call0(&this);
                }
            }
        }
        // Drop the existing closure if any
        self.animation_closure.take();
        Ok(())
    }
}

impl Drop for ScrambleText {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

use js_sys::Promise;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

use scramble_text::{ScrambleText, UseScrambleProps};

async fn sleep(ms: f64) {
    let promise = Promise::new(&mut |resolve, _| {
        let window = web_sys::window().unwrap();
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .unwrap();
    });
    JsFuture::from(promise).await.unwrap();
}

fn setup_test_element(document: &Document) -> Element {
    let element = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    element
}

#[wasm_bindgen_test]
async fn test_basic_scramble() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = setup_test_element(&document);
    let original_text = "Hello World";

    // Create scramble instance
    let props = JsValue::from_serde(&UseScrambleProps {
        text: original_text.to_string(),
        speed: 1.0,
        ..Default::default()
    })
    .unwrap();

    let mut scramble = ScrambleText::new(element.clone(), props).unwrap();

    // Test animation callbacks
    let mut start_called = false;
    let mut frame_called = false;
    let mut end_called = false;

    scramble.set_on_animation_start(js_sys::Function::new_no_args("start_called = true;"));
    scramble.set_on_animation_frame(js_sys::Function::new_with_args(
        "text",
        "frame_called = true;",
    ));
    scramble.set_on_animation_end(js_sys::Function::new_no_args("end_called = true;"));

    // Start animation
    scramble.start().unwrap();

    // Wait a short time to let animation run
    sleep(100.0).await;

    // Text should be different during animation
    assert_ne!(element.text_content().unwrap(), original_text);

    // Wait for animation to complete
    sleep(1000.0).await;
    scramble.stop().unwrap();

    // Verify callbacks were called
    assert!(js_sys::eval("start_called").unwrap().as_bool().unwrap());
    assert!(js_sys::eval("frame_called").unwrap().as_bool().unwrap());
    assert!(js_sys::eval("end_called").unwrap().as_bool().unwrap());
}

#[wasm_bindgen_test]
fn test_props_validation() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.create_element("div").unwrap();

    // Test various invalid props
    let invalid_cases = vec![
        (2.0, -1, 0.0, "speed > 1"), // Invalid speed
        (0.5, 0, 0.5, "tick <= 0"),  // Invalid tick
        (0.5, 1, 1.5, "chance > 1"), // Invalid chance
    ];

    for (speed, tick, chance, case) in invalid_cases {
        let props = JsValue::from_serde(&UseScrambleProps {
            text: "Test".to_string(),
            speed,
            tick,
            chance,
            ..Default::default()
        })
        .unwrap();

        assert!(
            ScrambleText::new(element.clone(), props).is_err(),
            "Failed to catch {}",
            case
        );
    }

    // Test valid props
    let valid_props = JsValue::from_serde(&UseScrambleProps {
        text: "Test".to_string(),
        speed: 0.5,
        chance: 0.8,
        tick: 1,
        ..Default::default()
    })
    .unwrap();

    assert!(
        ScrambleText::new(element, valid_props).is_ok(),
        "Valid props should work"
    );
}

#[wasm_bindgen_test]
async fn test_overdrive_mode() {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = setup_test_element(&document);
    let original_text = "Test Overdrive";

    let props = JsValue::from_serde(&UseScrambleProps {
        text: original_text.to_string(),
        overdrive: true,
        speed: 1.0,
        ..Default::default()
    })
    .unwrap();

    let mut scramble = ScrambleText::new(element.clone(), props).unwrap();

    // Start animation and wait briefly
    scramble.start().unwrap();
    sleep(100.0).await;

    // In overdrive mode, text should be different
    let current_text = element.text_content().unwrap();
    assert_ne!(
        current_text, original_text,
        "Text should change in overdrive mode"
    );
    assert!(
        current_text.len() == original_text.len(),
        "Text length should remain the same"
    );

    scramble.stop().unwrap();
}

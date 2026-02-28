// Phase 3: NativeRenderer foreign trait for callback-based rendering.
// This will allow native code to implement a rendering interface
// that Rust can call back into.
//
// Example (future):
// #[uniffi::export(callback_interface)]
// pub trait NativeRenderer: Send + Sync {
//     fn clear(&self, r: f32, g: f32, b: f32, a: f32);
//     fn draw_path(&self, segments: Vec<FfiPathSegment>, r: f32, g: f32, b: f32, a: f32);
//     fn save_state(&self);
//     fn restore_state(&self);
// }

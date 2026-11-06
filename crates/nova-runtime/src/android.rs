//! Android JNI entry points and lifecycle bindings for NOVA Engine.

use crate::RuntimeSession;
use nova_core::Id;
use nova_math::Vec2;
use std::sync::Mutex;

static GLOBAL_SESSION: Mutex<Option<RuntimeSession>> = Mutex::new(None);

pub fn init_android_session() {
    let mut guard = GLOBAL_SESSION.lock().unwrap();
    let place = experience_nova_obby::build_obby_world(Id::new());
    *guard = Some(RuntimeSession::new(place));
}

pub fn resize_viewport(_width: i32, height: i32) {
    let mut guard = GLOBAL_SESSION.lock().unwrap();
    if let Some(session) = guard.as_mut() {
        session.touch_controls.joystick.center = Vec2::new(140.0, height as f32 - 140.0);
        session.touch_controls.joystick.current = session.touch_controls.joystick.center;
    }
}

pub fn update_touch_input(stick_x: f32, stick_y: f32, jump_pressed: bool) {
    let mut guard = GLOBAL_SESSION.lock().unwrap();
    if let Some(session) = guard.as_mut() {
        session.touch_controls.joystick.is_active = stick_x.abs() > 0.05 || stick_y.abs() > 0.05;
        session.touch_controls.joystick.current = Vec2::new(
            session.touch_controls.joystick.center.x + stick_x * session.touch_controls.joystick.radius,
            session.touch_controls.joystick.center.y + stick_y * session.touch_controls.joystick.radius,
        );
        session.touch_controls.jump_button_pressed = jump_pressed;
    }
}

pub struct RenderBufferPacket {
    pub count: usize,
    pub float_data: Vec<f32>, // [px, py, pz, sx, sy, sz, r, g, b, a] * count
}

pub fn step_simulation_and_get_render_data(dt: f32) -> RenderBufferPacket {
    let mut guard = GLOBAL_SESSION.lock().unwrap();
    if let Some(session) = guard.as_mut() {
        session.tick(dt);

        let mut data = Vec::new();
        let mut count = 0;

        // 1. Player Character (Cube / Rig)
        let c_pos = session.character.position;
        let c_size = nova_math::Vec3::new(session.character.radius, session.character.height * 0.5, session.character.radius);
        data.extend_from_slice(&[
            c_pos.x, c_pos.y + session.character.height * 0.5, c_pos.z,
            c_size.x, c_size.y, c_size.z,
            0.15, 0.55, 0.95, 1.0, // Blue character
        ]);
        count += 1;

        // 2. World Platforms and Obstacles
        for entity in session.place.entities.values() {
            if let Some(rb) = &entity.rigid_body {
                let pos = entity.transform.position;
                let half_ext = match rb.shape {
                    nova_physics::ColliderShape::Box { half_extents } => half_extents,
                    _ => nova_math::Vec3::new(1.0, 1.0, 1.0),
                };

                let color = if entity.tags.contains(&"checkpoint".to_string()) {
                    [0.2, 0.85, 0.3, 1.0] // Green checkpoint
                } else if entity.tags.contains(&"finish".to_string()) {
                    [1.0, 0.84, 0.0, 1.0] // Gold finish
                } else if entity.tags.contains(&"lava_kill".to_string()) {
                    [0.9, 0.15, 0.1, 0.9] // Red lava
                } else if entity.tags.contains(&"hazard".to_string()) {
                    [0.95, 0.45, 0.1, 1.0] // Orange moving obstacle
                } else {
                    [0.85, 0.85, 0.9, 1.0] // Grey stone platform
                };

                data.extend_from_slice(&[
                    pos.x, pos.y, pos.z,
                    half_ext.x, half_ext.y, half_ext.z,
                    color[0], color[1], color[2], color[3],
                ]);
                count += 1;
            }
        }

        RenderBufferPacket { count, float_data: data }
    } else {
        RenderBufferPacket { count: 0, float_data: Vec::new() }
    }
}

// Raw JNI function pointer types matching standard jni.h
#[repr(C)]
pub struct JNIEnv {
    functions: *const JNINativeInterface,
}

#[repr(C)]
struct JNINativeInterface {
    _reserved0: *mut std::ffi::c_void,
    _reserved1: *mut std::ffi::c_void,
    _reserved2: *mut std::ffi::c_void,
    _reserved3: *mut std::ffi::c_void,
    _funcs: [*mut std::ffi::c_void; 211],
    get_direct_buffer_address: unsafe extern "C" fn(*mut JNIEnv, *mut std::ffi::c_void) -> *mut std::ffi::c_void,
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_com_nova_engine_NovaMainActivity_nativeInit(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
) {
    init_android_session();
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_com_nova_engine_NovaMainActivity_nativeResize(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    width: i32,
    height: i32,
) {
    resize_viewport(width, height);
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_com_nova_engine_NovaMainActivity_nativeTouchInput(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    stick_x: f32,
    stick_y: f32,
    jump_pressed: u8, // JNI jboolean
) {
    update_touch_input(stick_x, stick_y, jump_pressed != 0);
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_com_nova_engine_NovaMainActivity_nativeGetPlayerPos(
    env: *mut JNIEnv,
    _class: *mut std::ffi::c_void,
    out_direct_buffer: *mut std::ffi::c_void, // DirectByteBuffer
) {
    if env.is_null() || out_direct_buffer.is_null() {
        return;
    }
    unsafe {
        let get_addr = (*(*env).functions).get_direct_buffer_address;
        let addr = get_addr(env, out_direct_buffer) as *mut f32;
        if !addr.is_null() {
            let guard = GLOBAL_SESSION.lock().unwrap();
            if let Some(session) = guard.as_ref() {
                *addr.offset(0) = session.character.position.x;
                *addr.offset(1) = session.character.position.y;
                *addr.offset(2) = session.character.position.z;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_com_nova_engine_NovaMainActivity_nativeStep(
    env: *mut JNIEnv,
    _class: *mut std::ffi::c_void,
    dt: f32,
    out_direct_buffer: *mut std::ffi::c_void, // DirectByteBuffer
    max_floats: i32,
) -> i32 {
    let packet = step_simulation_and_get_render_data(dt);
    if !env.is_null() && !out_direct_buffer.is_null() && max_floats > 0 {
        unsafe {
            let get_addr = (*(*env).functions).get_direct_buffer_address;
            let addr = get_addr(env, out_direct_buffer) as *mut f32;
            if !addr.is_null() {
                let copy_len = packet.float_data.len().min(max_floats as usize);
                std::ptr::copy_nonoverlapping(packet.float_data.as_ptr(), addr, copy_len);
            }
        }
    }
    packet.count as i32
}

use std::{
    cell::Cell,
    os::raw::c_int,
    ptr,
    rc::Rc,
    time::{Duration, Instant},
};

use classicube_helpers::events::input;
use classicube_sys::{
    Entities, IGameComponent, InputButtons_CCKEY_W, LocalPlayer, ENTITIES_SELF_ID,
};

const DOUBLE_TAP_MILLIS: u64 = 400;

extern "C" fn init() {
    println!(
        "Init {}",
        concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"))
    );
}

extern "C" fn free() {}

extern "C" fn reset() {}

extern "C" fn on_new_map() {}

extern "C" fn on_new_map_loaded() {
    thread_local!(
        static EVENT_HANDLER: (input::Down2EventHandler, input::Up2EventHandler) = {
            let mut down_handler = input::Down2EventHandler::new();
            let mut up_handler = input::Up2EventHandler::new();

            // TODO keybinds instead of hardcoded W
            // TODO pressing S turns it off?

            let entity_ptr = unsafe { Entities.List[ENTITIES_SELF_ID as usize] };
            let local_player = entity_ptr as *mut LocalPlayer;

            let sprinting = Rc::new(Cell::new(false));
            let mut last_forward_down = None;
            down_handler.on({
                let sprinting = sprinting.clone();
                move |input::Down2Event { key, repeating, .. }| {
                    if key != &InputButtons_CCKEY_W || *repeating {
                        return;
                    }

                    let now = Instant::now();
                    if let Some(last) = last_forward_down {
                        if (last + Duration::from_millis(DOUBLE_TAP_MILLIS)) >= now {
                            let local_player = unsafe { &mut *local_player };
                            local_player.Hacks.HalfSpeeding = 1;
                            last_forward_down = None;
                            sprinting.set(true);
                            return;
                        }
                    }

                    last_forward_down = Some(now);
                }
            });
            up_handler.on({
                let sprinting = sprinting.clone();
                move |input::Up2Event { key, .. }| {
                    if key != &InputButtons_CCKEY_W {
                        return;
                    }

                    if sprinting.get() {
                        let local_player = unsafe { &mut *local_player };
                        local_player.Hacks.HalfSpeeding = 0;
                        sprinting.set(false);
                    }
                }
            });

            (down_handler, up_handler)
        };
    );

    EVENT_HANDLER.with(|_| {});
}

#[no_mangle]
pub static Plugin_ApiVersion: c_int = 1;

#[no_mangle]
pub static mut Plugin_Component: IGameComponent = IGameComponent {
    // Called when the game is being loaded.
    Init: Some(init),
    // Called when the component is being freed. (e.g. due to game being closed)
    Free: Some(free),
    // Called to reset the component's state. (e.g. reconnecting to server)
    Reset: Some(reset),
    // Called to update the component's state when the user begins loading a new map.
    OnNewMap: Some(on_new_map),
    // Called to update the component's state when the user has finished loading a new map.
    OnNewMapLoaded: Some(on_new_map_loaded),
    // Next component in linked list of components.
    next: ptr::null_mut(),
};

// ideas
//
// custom tilemap code? or hand write? custom tilemap code is preferrable.

#[cfg(feature = "buddy-alloc")]
mod alloc;
mod spritesheet;
mod wasm4;
use std::cell::RefCell;
use wasm4::*;
use num;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash)]
enum kitty_states {
    idle,
    moving_1,
    moving_2,
    jump
}
impl Character {
    fn new() -> Character {
        let kitty_state_to_anim_map = HashMap::from([
            (kitty_states::idle, 0),
            (kitty_states::moving_1, 1),
            (kitty_states::moving_2, 2),
            (kitty_states::jump, 3)
        ]);
        Character {
            x_pos: 0.0, y_pos: 0.0, x_vel: 0.0, y_vel: 0.0,
            x_vel_cap: 2.0, y_vel_cap: 7.0, count: 0, facing_right: true,
            state: kitty_states::idle, state_timer: 0, state_to_anim_map: kitty_state_to_anim_map
        }
    }
}


struct Character {
    x_pos: f32,
    y_pos: f32,
    x_vel: f32,
    y_vel: f32,
    x_vel_cap: f32,
    y_vel_cap: f32,
    count: i32,
    facing_right: bool,
    state: kitty_states,
    state_timer: i32,
    state_to_anim_map: HashMap<kitty_states, usize>
}

thread_local!(static BOB: RefCell<Character> = RefCell::new(Character::new()));
thread_local!(static KITTY_SPRITES: RefCell<Vec<spritesheet::Sprite<150, 4>>> = RefCell::new(vec![
    spritesheet::get_sprite_from_spritesheet([1, 2, 3, 4], 0, 0, 2, 2),
    spritesheet::get_sprite_from_spritesheet([1, 2, 3, 4], 2, 0, 2, 2),
    spritesheet::get_sprite_from_spritesheet([1, 2, 3, 4], 0, 2, 2, 2),
    spritesheet::get_sprite_from_spritesheet([1, 2, 3, 4], 2, 2, 2, 2),
    spritesheet::get_sprite_from_spritesheet([1, 2, 3, 4], 4, 0, 2, 2),
    spritesheet::get_sprite_from_spritesheet([1, 2, 3, 4], 4, 2, 2, 2),
]));

fn get_pos(bob: &Character) {
    let _ = bob.x_pos;
}

#[no_mangle]
fn start() {
    unsafe {
        *PALETTE = spritesheet::KITTY_SS_PALLETE;
    }
}

#[no_mangle]
fn update() {
    BOB.with(|bob_cell| {
        KITTY_SPRITES.with(|kitty_sprites_cell| {
            let mut bob = bob_cell.borrow_mut();
            let kitty_sprites = kitty_sprites_cell.borrow();
            unsafe { *DRAW_COLORS = spritesheet::KITTY_SS_DRAW_COLORS }
            // text("Hello from Rust!", 10, 10);

            let gamepad = unsafe { *GAMEPAD1 };
            
            let btn_accel = 0.6;
            let hop_v: f32 = -3.0;
            let h_decay = 0.8;
            if gamepad & BUTTON_LEFT != 0 {
                bob.x_vel -= btn_accel;
                bob.facing_right = false;
                bob.state = kitty_states::moving_1;
            } else if gamepad & BUTTON_RIGHT != 0 {
                bob.x_vel += btn_accel;
                bob.facing_right = true;
                bob.state = kitty_states::moving_2;
            } else {
                bob.x_vel *= h_decay;
                bob.state = kitty_states::idle;
            }
            if gamepad & BUTTON_UP != 0 {
                bob.y_vel = hop_v;
                bob.state = kitty_states::jump;
            } else if gamepad & BUTTON_DOWN != 0 {
                
            }

            bob.x_pos += bob.x_vel;
            bob.y_pos += bob.y_vel;
            
            let gravity = 0.1;
            bob.y_vel += gravity;
            get_pos(&bob);
            bob.x_pos = num::clamp(bob.x_pos, 0.0, 159.0);
            bob.y_pos = num::clamp(bob.y_pos, 0.0, 149.0);
            bob.x_vel = num::clamp(bob.x_vel, -bob.x_vel_cap, bob.x_vel_cap);
            bob.y_vel = num::clamp(bob.y_vel, -bob.y_vel_cap, bob.y_vel_cap);
            bob.count += 1;
            let i: usize = ((bob.count / 30) % 4) as usize;
            // blit(&SMILEY, bob.x_pos as i32, bob.y_pos as i32, 8, 8, BLIT_1BPP);
            // text("Press X to blink", 16, 90);
            // [blit(&spritesheet::KITTY_SS, 0, 0, spritesheet::KITTY_SS_WIDTH, spritesheet::KITTY_SS_HEIGHT, spritesheet::KITTY_SS_FLAGS);
            blit(
                &kitty_sprites[3].frames[*bob.state_to_anim_map.get(&bob.state).unwrap()].img,
                bob.x_pos as i32,
                bob.y_pos as i32,
                kitty_sprites[0].frames[i].width as u32,
                kitty_sprites[0].frames[i].height as u32,
                spritesheet::KITTY_SS_FLAGS | if bob.facing_right {0} else {BLIT_FLIP_X},
            );
            blit(
                &kitty_sprites[1].frames[i].img,
                130,
                100,
                kitty_sprites[1].frames[i].width as u32,
                kitty_sprites[1].frames[i].height as u32,
                spritesheet::KITTY_SS_FLAGS,
            );
            blit(
                &kitty_sprites[2].frames[i].img,
                90,
                120,
                kitty_sprites[2].frames[i].width as u32,
                kitty_sprites[2].frames[i].height as u32,
                spritesheet::KITTY_SS_FLAGS,
            );
            blit(
                &kitty_sprites[3].frames[i].img,
                40,
                100,
                kitty_sprites[3].frames[i].width as u32,
                kitty_sprites[3].frames[i].height as u32,
                spritesheet::KITTY_SS_FLAGS,
            );
        });
    });
}

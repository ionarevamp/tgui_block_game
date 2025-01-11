
use rayon::prelude::*;

use tgui::{
    utils::{
        Color,
        Vec2
    },
    widgets::{
        label::TextView,
        View
    },
    TGui,
    AF
};

use interpolator::{format, Formattable, iwrite};

use rand::{
    thread_rng,
    seq::SliceRandom,
};

use image::{Rgb, RgbImage};

use std::{
    sync::{
        Arc,
        Mutex,
        mpsc::channel,
    },
    thread::{
        self,
        sleep,
    },
    time::{
        Duration,
        Instant,
    },
    option::Option,
    path::Path,
    fs::File,
    fs::create_dir,
    io::Write,
};

#[derive(Debug, Clone)]
enum Action {
    Projectile(f64),
}

#[derive(Debug, Clone)]
enum EnemyType {
    Weak,
    Medium,
    Strong
}
#[derive(Debug, Clone)]
enum ObjectName {
    Player,
    Enemy(EnemyType),
}

#[derive(Debug, Clone)]
struct Object {
    pub name: ObjectName,
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub action: Action,
    pub recharge: f64,
    pub hp: f64,
    pub max_hp: f64,
}

impl Object {
    fn new(name: ObjectName, x: f64, y: f64, size: f64, max_hp: f64, action: Action) -> Self {
        Object {
            name,
            x,
            y,
            size,
            action,
            recharge: 0.0,
            hp: max_hp,
            max_hp,
        }
    }
    fn up(&mut self, amount: f64) -> &mut Self {
        self.y -= amount;
        self
    }
    fn down(&mut self, amount: f64) -> &mut Self {
        self.y += amount;
        self
    }
    fn left(&mut self, amount: f64) -> &mut Self {
        self.x -= amount;
        self
    }
    fn right(&mut self, amount: f64) -> &mut Self {
        self.x += amount;
        self
    }
    fn as_overlay(&self, buffer_height: usize, buffer_width: usize) -> Overlay {
        // TODO: match on self and write different rendering functions for different objects given
        // their position and state
        let mut overlay = Overlay::new(buffer_height, buffer_width);

        use ObjectName::*;

        println!("Drawing at {}, {}", self.x, self.y);

        for x in ((self.x-self.size/2.0) as usize)..((self.x+self.size/2.0) as usize) {
            for y in ((self.y-self.size/2.0) as usize)..((self.y+self.size/2.0) as usize) {
                let _ = overlay.set_pixel(
                    x,
                    y,
                    match self.name {
                        Player =>   &[0,0,0],
                        Enemy(_) => &[255,30,30],
                        _ =>        &[150,150,150],
                    }
                );
            }
        }

        overlay
    }
    fn collides_with(&self, object: Object) -> bool {
        let margin_self = self.size / 2.0;
        let margin_other = object.size / 2.0;
        
        for player_corner in 
        [
            (self.x-margin_self, self.y-margin_self),
            (self.x+margin_self, self.y-margin_self),
            (self.x-margin_self, self.y+margin_self),
            (self.x+margin_self, self.y+margin_self)
        ].iter() {
            if
                player_corner.0 > object.x-margin_other &&
                player_corner.1 > object.y-margin_other &&
                player_corner.0 < object.x+margin_other &&
                player_corner.1 < object.y+margin_other

            {
                return true;
            }
        }

        false
    }
    fn use_ability(&mut self, objects: &mut [Object]) -> &mut Self {

        let mut closest = &mut Object::new(ObjectName::Enemy(EnemyType::Weak), 10000.0, 10000.0, 0.0, 10000.0, Action::Projectile(0.0));
        let mut mag = 0.0;

        for object in objects.iter_mut() {
            use Action::*;

            #[allow(clippy::single_match)]
            match self.action {
                Projectile(magnitude) => {
                    let current_distance =  ((self.x - object.x).powf(2.0) + (self.y - object.y).powf(2.0)).sqrt();
                    let closest_distance = ((self.x - closest.x).powf(2.0) + (self.y - object.y).powf(2.0)).sqrt();
                        if current_distance <= (magnitude * 1.5) + 19.5f64 && current_distance < closest_distance {
                            println!("Found closer enemy");
                            closest = &mut *object;
                            mag = magnitude;
                        }
                    },
                    _ => {},
            }
        }
        
        println!("Damaging enemy");
        closest.hp -= mag;
        
        self
    }

}

mod anim_data;
use anim_data::{Overlay, Alter};

pub fn black_background() -> RgbImage {
    let mut bg = RgbImage::new(WIDTH as u32, HEIGHT as u32);

    for pixel in bg.pixels_mut() {
        *pixel = [0,0,0].into();
    }

    bg
}
pub fn white_background() -> RgbImage {
    let mut bg = RgbImage::new(WIDTH as u32, HEIGHT as u32);

    for pixel in bg.pixels_mut() {
        *pixel = [255,255,255].into();
    }

    bg
}

pub fn render_string(image: RgbImage) -> String {
    let mut buff = Vec::new();

    image.write_to(&mut std::io::Cursor::new(&mut buff), image::ImageOutputFormat::Jpeg(100)).unwrap();


    let mut res_base64 = base64::encode(&buff);
    res_base64.shrink_to_fit();
    res_base64
}


#[derive(PartialEq)]
enum WhichData {
    FramesAhead(usize),
}

#[derive(PartialEq)]
enum ChannelEvent {
    Input(tgui::event::Event),
    Frame(String),
    Data(WhichData),
    Ready,
    Done
}

use ChannelEvent::*;
use WhichData::*;


pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

fn main() {

    let tgui = Arc::new(TGui::new());
    let t_gui = tgui.clone();
    let _tgui = t_gui.clone();

    let mut flags = AF::empty();
    let ui = t_gui.ui(None, flags);
    let layout = ui.linear_layout(None, true);

    let image_frame = ui.frame_layout(Some(&layout));
    let image = ui.image_view(Some(&image_frame));
    image.set_image_string(&render_string(white_background()));
    let _image = Arc::new(image);
    
    let controls_section = ui.linear_layout(Some(&layout), true);
    let top_controls = ui.linear_layout(Some(&controls_section), false);
    let middle_controls = ui.linear_layout(Some(&controls_section), false);
    let bottom_controls = ui.linear_layout(Some(&controls_section), false);

    let _ = ui.space(Some(&top_controls));
//    let _ = ui.space(Some(&top_controls));
    let up_left_arrow = ui.button(Some(&top_controls), "┌");
    let up_arrow = ui.button(Some(&top_controls), "^");
    let up_right_arrow = ui.button(Some(&top_controls), "┐");
//    let _ = ui.space(Some(&top_controls));
    let _ = ui.space(Some(&top_controls));

    let _ = ui.space(Some(&middle_controls));
    let left_arrow = ui.button(Some(&middle_controls), "<");
    let action_button = ui.button(Some(&middle_controls), "o");
    let right_arrow = ui.button(Some(&middle_controls), ">");
    let _ = ui.space(Some(&middle_controls));
    
    let _ = ui.space(Some(&bottom_controls));
    let down_left_arrow = ui.button(Some(&bottom_controls), "└");
    let down_arrow = ui.button(Some(&bottom_controls), "v");
    let down_right_arrow = ui.button(Some(&bottom_controls), "┘");
    let _ = ui.space(Some(&bottom_controls));

    let bottom_line = ui.linear_layout(Some(&layout), false);
    let exit_button = ui.button(Some(&bottom_line), "EXIT");
    exit_button.set_background_color(Color::from_rgb(255, 30, 30));


    let (eventtx, eventrx) = channel::<ChannelEvent>();
    let _eventtx = eventtx.clone();
    let (imagerqtx, imagerqrx) = channel::<ChannelEvent>();


    rayon::spawn(move || {
        loop {
            let result = eventtx.send(Input(tgui.event()));
            //println!("Tgui event: {result:?}");
        }
    });


    let mut player = Object::new(
        ObjectName::Player,
        (WIDTH/2) as f64,
        (HEIGHT/2) as f64,
        10.0,
        20.0,
        Action::Projectile(2.0),
    );

    let mut enemy_list = Vec::with_capacity(10);

    for i in 9..40 {
        enemy_list.push( Object::new(
            ObjectName::Enemy(EnemyType::Weak),
            ((WIDTH/2 + (i+1) * 10) % WIDTH) as f64,
            ((HEIGHT/2 + (i+1) * 10) % HEIGHT) as f64,
            10.0,
            5.0,
            Action::Projectile(0.5)
        ));
    }

    let enemy_list = Arc::new(Mutex::new(enemy_list));

    let player = Arc::new(Mutex::new(player.clone()));

    let _player = player.clone();
    let _enemy_list = enemy_list.clone();
    rayon::spawn(move || {
        // TODO: make the player a mutex as well as the game state and send image over a channel to
        // handle rendering
        loop {
            if let Ok(ev) = imagerqrx.recv() {
                println!("Rendering thread received event");
                let player = loop {
                    if let Ok(lock) = _player.lock() {
                        break lock;
                    }
                };
                let mut enemy_list = loop {
                    if let Ok(lock) = _enemy_list.lock() {
                        break lock;
                    }
                };
                if ev == ChannelEvent::Ready {
                    let mut background = white_background();

                    let player_overlay = player.as_overlay(WIDTH, HEIGHT);

                    background.overlay(0,0,1.0,1.0, &player_overlay);

                    for (i, pix) in player_overlay.pixels.iter().enumerate() {
                        if let Some(val) = pix {
                            //println!("first valid value is {:?}, idx {i}", val);
                        }
                    }
                    let mut cached_delete = Vec::new();
                    for (i, enemy) in enemy_list.iter_mut().enumerate() {
                        let distance_to_player = ((player.x-enemy.x).powf(2.0) + (player.y-enemy.y).powf(2.0)).sqrt();
                        if player.collides_with(enemy.clone()) {
                            println!("Game over.");
                            let _ = _eventtx.send(ChannelEvent::Done);
                        }

                        let distance_to_player = distance_to_player * if let ObjectName::Enemy(enemy_type) = &enemy.name {
                            use EnemyType::*;
                            match enemy_type {
                                Weak => 1.8,
                                Medium => 1.5,
                                Strong => 1.2,
                                _ => 1.0,
                            }
                        } else { 1.0 };

                        enemy.x += (player.x - enemy.x) / distance_to_player;
                        enemy.y += (player.y - enemy.y) / distance_to_player;

                        if enemy.hp <= 0.0 {
                            cached_delete.push(i);
                        } else {
                            background.overlay(0,0,1.0,1.0, &enemy.as_overlay(WIDTH, HEIGHT));
                        }
                    }

                    for idx in cached_delete.iter().rev() {
                        let _ = enemy_list.remove(*idx);
                    }

                    let mut base64 = render_string(background);
                    base64.shrink_to_fit();

                    let _ = _eventtx.send(ChannelEvent::Frame(base64));
                }
                    
            }
        }
    });

    
    let mut running = true;
    while running {
        if let Ok(event) = eventrx.try_recv() {
            #[allow(clippy::single_match)]
            match event {
                Input(event) => {
                    println!("Got Input event {:?}", event.value);
                    if event.id == exit_button.get_id() {
                        running = false;
                    }
                    if event.id == action_button.get_id() {
                        let mut player = loop {
                            if let Ok(lock) = player.lock() {
                                break lock;
                            }
                        };
                        let mut enemy_list = loop {
                            if let Ok(lock) = enemy_list.lock() {
                                break lock;
                            }
                        };
                        player.use_ability(&mut enemy_list);
                    }
                    if event.id == up_left_arrow.get_id() {
                        let mut player = loop {
                            if let Ok(lock) = player.lock() {
                                break lock;
                            }
                        };
                        player.up(5.0).left(5.0);
                    }

                    if event.id == up_arrow.get_id() {
                        let mut player = loop {
                            if let Ok(lock) = player.lock() {
                                break lock;
                            }
                        };
                        player.up(5.0);
                    }
                    if event.id == up_right_arrow.get_id() {
                        let mut player = loop {
                            if let Ok(lock) = player.lock() {
                                break lock;
                            }
                        };
                        player.up(5.0).right(5.0);
                    }
                    if event.id == down_arrow.get_id() {
                        let mut player = loop {
                            if let Ok(lock) = player.lock() {
                                break lock;
                            }
                        };
                        player.down(5.0);
                    }
                    if event.id == down_left_arrow.get_id() {
                        let mut player = loop {
                            if let Ok(lock) = player.lock() {
                                break lock;
                            }
                        };
                        player.down(5.0).left(5.0);
                    }
                    if event.id == down_right_arrow.get_id() {
                        let mut player = loop {
                            if let Ok(lock) = player.lock() {
                                break lock;
                            }
                        };
                        player.down(5.0).right(5.0);
                    }
                    if event.id == left_arrow.get_id() {
                        let mut player = loop {
                            if let Ok(lock) = player.lock() {
                                break lock;
                            }
                        };
                        player.left(5.0);
                    }
                    if event.id == right_arrow.get_id() {
                        let mut player = loop {
                            if let Ok(lock) = player.lock() {
                                break lock;
                            }
                        };
                        player.right(5.0);
                    }

                },
                Frame(string) => {
                    image.set_image_string(&string);
                    continue;
                },
                Done => {
                    running = false;
                    continue;
                }
                _ => {},
            }
        }

        let _ = imagerqtx.send(ChannelEvent::Ready);
        sleep(Duration::from_millis((1000.0 / 30.0) as u64));
    }
}

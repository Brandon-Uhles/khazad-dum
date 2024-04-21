use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator, Rltk, RGB};
use std::cmp::{max, min};
use specs::prelude::*;

#[derive(Default)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}

impl Map {
    fn apply_room_to_map(&mut self, room : &Rect) {
        for y in room.y1 + 1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
    
    fn apply_tunnel_h(&mut self, x1:i32, x2:i32, y:i32) {
        for x in min(x1, x2) ..= max(x1,x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize  {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
    
    fn apply_tunnel_v(&mut self, y1:i32, y2:i32, x:i32) {
        for y in min(y1, y2) ..= max(y1,y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    pub fn new_map_room_and_corridors() -> Map {    
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;
        
        let mut map = Map {
            tiles : vec![TileType::Wall; 80*50],
            rooms : Vec::new(),
            width : 80,
            height : 50,
            revealed_tiles : vec![false; 80*50],
            visible_tiles : vec![false; 80*50],
        };
    
        let mut rng = RandomNumberGenerator::new();
    
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                map.apply_room_to_map(&new_room);
    
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        map.apply_tunnel_h(prev_x, new_x, prev_y);
                        map.apply_tunnel_v(prev_y, new_y, new_x);
                    } else {
                        map.apply_tunnel_h(prev_y, new_y, prev_x);
                        map.apply_tunnel_v(prev_x, new_x, new_y);
                    }
               }
               map.rooms.push(new_room);
            }
        }
        map
    }

    /// returns index of 2d map location mapped to a linear array
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Rect {
    pub x1 : i32,
    pub x2 : i32,
    pub y1 : i32,
    pub y2 : i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect{x1:x, y1:y, x2:x+w, y2:y+h}
    }

    pub fn intersect(&self, other:&Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2)/2, (self.y1 + self.y2)/2)
    }
}


/// draw function for map
pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
 
    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                        fg = RGB::from_f32(0.0, 0.5, 0.5);
                        glyph = rltk::to_cp437('.');
                }
                TileType::Wall => {
                        fg = RGB::from_f32(0., 1.0, 0.);                           
                        glyph = rltk::to_cp437('#');
                    
                }
            }
            if !map.visible_tiles[idx] { fg = fg.to_greyscale() }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}


// makes map with solid boundaries and 400 random tiles
/*pub fn _new_map_test() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80 * 50];

    // generate boundary wall
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // test spew of a bunch of walls
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}*/

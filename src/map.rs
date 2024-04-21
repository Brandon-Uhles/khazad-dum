use rltk::{RandomNumberGenerator, Rltk, Tile, RGB};
use std::cmp::{max, min};

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

fn apply_room_to_map(room : &Rect, map: &mut [TileType]) {
    for y in room.y1 + 1 ..= room.y2 {
        for x in room.x1 + 1 ..= room.x2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn apply_tunnel_h(map: &mut [TileType], x1:i32, x2:i32, y:i32) {
    for x in min(x1, x2) ..= max(x1,x2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < 80*50 {
            map[idx as usize] = TileType::Floor;
        }
    }
}

fn apply_tunnel_v(map: &mut [TileType], y1:i32, y2:i32, x:i32) {
    for y in min(y1, y2) ..= max(y1,y2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < 80*50 {
            map[idx as usize] = TileType::Floor;
        }
    }
}


/// draw function for map
pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // render tile depending on tile type
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('#'),
                );
            }
        }

        // move coordinates after tile render
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

pub fn new_map_room_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; 80*50];
    
    let mut rooms : Vec<Rect> = Vec::new();
    const MAX_ROOMS : i32 = 30;
    const MIN_SIZE : i32 = 6;
    const MAX_SIZE : i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, 80 - w - 1) - 1;
        let y = rng.roll_dice(1, 50 - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in rooms.iter() {
            if new_room.intersect(other_room) { ok = false }
        }
        if ok {
            apply_room_to_map(&new_room, &mut map);

            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len()-1].center();
                if rng.range(0,2) == 1 {
                    apply_tunnel_h(&mut map, prev_x, new_x, prev_y);
                    apply_tunnel_v(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_tunnel_h(&mut map, prev_y, new_y, prev_x);
                    apply_tunnel_v(&mut map, prev_x, new_x, new_y);
                }
           }
           rooms.push(new_room);
        }
    }
    (rooms, map)
}

/// returns index of 2d map location mapped to a linear array
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

/// makes map with solid boundaries and 400 random tiles
pub fn _new_map_test() -> Vec<TileType> {
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
}

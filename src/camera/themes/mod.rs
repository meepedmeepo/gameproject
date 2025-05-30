use bracket_lib::{color::{DARKSLATEGRAY, GHOST_WHITE, GRAY, GREENYELLOW, GREY40, LIGHT_GRAY, PINK2, PINK3, RGB, SLATE_BLUE}, prelude::{to_cp437, FontCharType}};

use crate::{Map, TileType};




pub fn tile_glyph(idx : usize, map : &Map) -> (String, RGB, RGB)
{
    let (glyph, mut fg, bg) = match map.depth 
    {
        _=> {get_tile_glyph_default(idx, map)}
    };

    if !map.visible_tiles[idx]
    {
        fg = fg.to_greyscale();
    }

    (glyph, fg, bg)
}

fn get_tile_glyph_default(idx : usize, map : &Map) -> (String, RGB, RGB)
{
    let glyph;
    let mut fg;
    let mut bg = RGB::from_f32(0., 0., 0.);

    match map.map[idx]
    {
        TileType::Floor =>  {glyph = ".".to_string(); fg = RGB::from_f32(0., 0.5, 0.5);}
        TileType::Wall => 
        {
            let x = idx as i32 % map.map_width;
            let y = idx as i32 / map.map_width;
            glyph = bracket_lib::terminal::to_char(wall_glyph(map, x, y)).to_string();
            fg = RGB::from_f32(0., 1., 0.);
        }
        TileType::DownStairs => {glyph = '>'.to_string(); fg = RGB::from_f32(0., 1., 1.);}
        TileType::Road => {glyph = '▓'.to_string(); fg = RGB::named(SLATE_BLUE);}
        TileType::MetalGrate => {glyph = '≡'.to_string(); fg = RGB::named(LIGHT_GRAY);}
        TileType::Concrete => {glyph = ".".to_string(); fg = RGB::named(PINK2);}
        TileType::Footpath => {glyph = '.'.to_string(); fg = RGB::named(GREENYELLOW);}
        TileType::RustedMetalFloor => {glyph = '~'.to_string(); fg = RGB::from_hex("#e04300").unwrap();}
    }

    

    (glyph, fg, bg)
}


fn wall_glyph(map : &Map, x: i32, y: i32) -> u8
{
    if x < 1 || x > map.map_width-2 || y < 1 || y > map.map_height-2 as i32 { return 35; }
    let mut mask : u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) { mask +=1; }
    if is_revealed_and_wall(map, x, y + 1) { mask +=2; }
    if is_revealed_and_wall(map, x - 1, y) { mask +=4; }
    if is_revealed_and_wall(map, x + 1, y) { mask +=8; }

    match mask {
        0 => { 9 } // Pillar because we can't see neighbors
        1 => { 186 } // Wall only to the north
        2 => { 186 } // Wall only to the south
        3 => { 186 } // Wall to the north and south
        4 => { 205 } // Wall only to the west
        5 => { 188 } // Wall to the north and west
        6 => { 187 } // Wall to the south and west
        7 => { 185 } // Wall to the north, south and west
        8 => { 205 } // Wall only to the east
        9 => { 200 } // Wall to the north and east
        10 => { 201 } // Wall to the south and east
        11 => { 204 } // Wall to the north, south and east
        12 => { 205 } // Wall to the east and west
        13 => { 202 } // Wall to the east, west, and south
        14 => { 203 } // Wall to the east, west, and north
        15 => { 206 }  // ╬ Wall on all sides
        _ => { 35 } // We missed one?
    }
}

fn is_revealed_and_wall(map : &Map, x: i32, y: i32) -> bool
{
    let idx = map.xy_idx(x, y);

    map.map[idx] == TileType::Wall && map.revealed_tiles[idx]
}
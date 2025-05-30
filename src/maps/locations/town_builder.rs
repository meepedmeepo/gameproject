use std::{collections::HashSet, hash::Hash};

use bracket_lib::{prelude::{a_star_search, console, DijkstraMap, DistanceAlg, Point}, random::RandomNumberGenerator};

use crate::{maps::{border_wall::BorderWall, voronoi::{DistanceAlgorithm, VoronoiCellBuilder}, AreaStartingPosition, DistantExitBuilder, MetaMapBuilder}, raws::RAWS, BuilderChain, BuilderMap, InitialMapBuilder, TileType};

use super::utils::{find_entity_spawn_locations, spawn_building_contents, vec_of_str};


const MAX_W : i32 = 13;
const MAX_H : i32 = 10;


pub struct Building
{
    pub name : String,
    pub to_spawn : Vec<String>,
}

//enum showing which wall the door will be placed on
#[derive(PartialEq, Eq, Clone, Copy)]
enum Orientation {N,E,S,W}

impl Orientation
{
    fn get_random(rng : &mut RandomNumberGenerator) -> Orientation
    {
        let roll = rng.roll_dice(1, 4);

        match roll
        {
            1 => {Orientation::N},
            2 => {Orientation::E},
            3 => {Orientation::S},
            _ => {Orientation::W}
        }
    }
}

pub fn starting_town() -> BuilderChain
{
    let mut builder = BuilderChain::new(0, 80, 80);
    // builder.start_with(VoronoiCellBuilder::new_advanced(300, DistanceAlgorithm::Manhattan
    //    , TileType::Road));
    builder.start_with(TownBuilder::new());
    //builder.with(BorderWall::new());
    //builder.with(AreaStartingPosition::new(crate::maps::XStart::CENTER, crate::maps::YStart::CENTER));
    //builder.with(DistantExitBuilder::new());

    builder
}


pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder 
{
    fn build_map(&mut self, rng: &mut bracket_lib::prelude::RandomNumberGenerator, build_data: &mut crate::BuilderMap) 
    {
        self.build(rng, build_data);
    }

}

impl MetaMapBuilder for TownBuilder
{
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl TownBuilder
{
    pub fn new() -> Box<TownBuilder>
    {
        Box::new(TownBuilder {})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap)
    {
        

        //creates a HashSet of available tiles to build that are within certain coordinate bounds
        let mut available_building_tiles: HashSet<usize> = HashSet::new();
        
        for y in 2..build_data.map.map_height-4
        {
            for x in 2..build_data.map.map_width - 2
            {
                if y < build_data.map.map_height - 15 && x < build_data.map.map_width - 4
                {
                    available_building_tiles.insert(build_data.map.xy_idx(x, y));
                }
            }
        }
        self.lay_concrete(build_data, &mut available_building_tiles);
        
        let road = self.paint_road(rng, build_data, &mut available_building_tiles);

        console::log(format!("available tiles {}", available_building_tiles.len()));

        let buildings = self.buildings(rng, build_data, &mut available_building_tiles);

        let doors = self.place_doors(rng, build_data, &buildings);

        self.building_content(build_data, &buildings, rng);

        //let road = self.paint_road(rng, build_data);

        //self.draw_footpaths(build_data, rng, &doors, road);

        self.spawn_townsfolk(rng, build_data);

    }


    //todo completely rework this, it looks like absolute dogshit to say the least oof
    fn draw_footpaths(&self, build_data : &mut BuilderMap,rng : &mut RandomNumberGenerator, doors : &Vec<usize>, road_pos : Point)
    {
        let mut mapclone = build_data.map.clone();
        mapclone.populate_blocked();
        for door in doors.iter()
        {
            mapclone.map[*door] = TileType::Wall
        }
        let mut footpath_start = road_pos;
        footpath_start.x += rng.roll_dice(1, 10) +5;
        footpath_start.y -= 2;

        let f_idx  = build_data.map.xy_idx(footpath_start.x, footpath_start.y);

        for door in doors.iter()
        {
            let path = a_star_search(f_idx, *door, &mapclone).steps;

            for step in path.iter()
            {
                build_data.map.map[*step] = TileType::Footpath;
            }
                
        }
    }
    fn lay_concrete(&mut self, build_data : &mut BuilderMap, available_building_tiles : &mut HashSet<usize>)
    {
        for y in 1..build_data.map.map_height-1
        {
            for x in 1..build_data.map.map_width-1
            {
                let idx = build_data.map.xy_idx(x, y);
                
                if build_data.map.map[idx] != TileType::Road
                {
                    build_data.map.map[idx] = TileType::Concrete;
                } else 
                {
                    available_building_tiles.remove(&idx);
                }
            }
        }
    }

    fn paint_road(&self,rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap, available_building_tiles : &mut HashSet<usize>) -> Point
    {
        let w = build_data.map.map_width as usize;
        let roll = rng.range(build_data.map.map_height-14, build_data.map.map_height-5);
        let start_pos = Point::new(1, roll);
        let start_idx = build_data.map.xy_idx(start_pos.x, start_pos.y);
        let end_roll = rng.range(10, build_data.map.map_height-3);
        let end_pos = Point::new(build_data.map.map_width-2, end_roll);
        let end_idx = build_data.map.xy_idx(end_pos.x, end_pos.y);

        let mut p1 = a_star_search(start_idx, end_idx, &build_data.map);
        let mut p2 = a_star_search(start_idx-w, end_idx-w, &build_data.map);
        let mut p3 = a_star_search(start_idx+w, end_idx+w, &build_data.map);

        let mut p4 = a_star_search(start_idx+w+w, end_idx+w+w, &build_data.map);
        let mut p5 = a_star_search(start_idx-w-w, end_idx-w-w, &build_data.map);
        p4.steps.append(&mut p5.steps);

        p1.steps.append(&mut p2.steps);
        p1.steps.append(&mut p3.steps);

        for step in p1.steps.iter()
        {
            available_building_tiles.remove(step);
            build_data.map.map[*step] = TileType::Road;
        }

        for step in p4.steps.iter()
        {
            available_building_tiles.remove(step);
        }


        
        start_pos
    }

    fn buildings(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap
        , available_building_tiles : &mut HashSet<usize>) -> Vec<(i32, i32, i32, i32)>
    {
        let mut n_buildings = 0;
        let mut buildings : Vec<(i32,i32,i32,i32)> = Vec::new();
        //create 10 lots for buildings and fill the inside with rusted metal flooring
        while n_buildings < 7
        {
            //console::log("trying to build hehe");
            let bx = rng.roll_dice(4, build_data.map.map_width-10);
            let by = rng.roll_dice(5, build_data.map.map_height-10);
            let bw = rng.roll_dice(1, MAX_W+1) +5;
            let bh = rng.roll_dice(1, MAX_H+1) + 5;

            let mut possible = true;
            for y in by..by+bh
            {
                for x in bx..bx+bw
                {
                    if x < 0 || x > build_data.map.map_width-1 || y < 0 || y > build_data.map.map_height-1
                    {
                        possible = false;
                    } else 
                    {
                        let idx = build_data.map.xy_idx(x, y);
                        if !available_building_tiles.contains(&idx) 
                        {
                            possible = false;
                        }    
                    }
                }
            }

            if possible
            {
                n_buildings += 1;
                console::log(format!("Successfully build {} buildings!", n_buildings));
                buildings.push((bx, by, bw, bh));
                
                for y in by..by+bh
                {
                    for x in bx..bx+bw
                    {
                        let idx = build_data.map.xy_idx(x, y);
                        build_data.map.map[idx] = TileType::RustedMetalFloor;

                        available_building_tiles.remove(&idx);
                        available_building_tiles.remove(&(idx+1));
                        available_building_tiles.remove(&(idx+build_data.map.map_width as usize));
                        available_building_tiles.remove(&(idx-1));
                        available_building_tiles.remove(&(idx-build_data.map.map_width as usize));
                    }
                }
            }
        }


        //put walls around buildings
        let mut map = build_data.map.clone();
        for y in 2..build_data.map.map_height-2
        {
            for x in 2..build_data.map.map_width-2
            {
                let idx= map.xy_idx(x, y);

                if build_data.map.map[idx] == TileType::RustedMetalFloor
                {
                    let mut neighbors = 0;

                    if build_data.map.map[idx-1] != TileType::RustedMetalFloor {neighbors+=1;}
                    if build_data.map.map[idx+1] != TileType::RustedMetalFloor {neighbors+=1;}
                    if build_data.map.map[idx- build_data.map.map_width as usize] != TileType::RustedMetalFloor {neighbors+=1;}
                    if build_data.map.map[idx+ build_data.map.map_width as usize] != TileType::RustedMetalFloor {neighbors+=1;}

                    if neighbors > 0
                    {
                        map.map[idx] = TileType::Wall;
                    }
                }
            }
        }

        build_data.map = map;


        buildings
    }

    fn building_content(&mut self, build_data : &mut BuilderMap, buildings : &Vec<(i32,i32,i32,i32)>,  rng : &mut RandomNumberGenerator)
    {
        let mut building_list = vec!["pub", "abandonedhouse", "ripperdocoffice", "drugden"];
        let building_list_clone = building_list.clone();
        building_list.push("accessroom");
        let mut spawn_any = false;
        for (i, build) in buildings.iter().enumerate()
        {
            let mut building_name : &str;
            if i == 0
            {
                building_name = "home";

                build_data.starting_position = Some(Point::new(build.0 + (build.2/2), build.1 + (build.3/2)))
            }
            else 
            {
                let mut index = 0;
                if building_list.len() == 1
                {
                    index = 0;
                } else if building_list.len() > 1
                {
                    index = rng.range(0,building_list.len());
                }
                else {
                    spawn_any = true;
                }

                
                if !spawn_any
                {
                    building_name = building_list[index];
                    building_list.remove(index);
                }
                else {
                    building_name = "";
                }
            }
            let building : crate::raws::Building;
            if !spawn_any
            {
                building = RAWS.lock().unwrap().get_building_from_name(building_name.to_string());
                if building_name == "accessroom"
                {
                    let point = Point::new(build.0 + (build.2/2), build.1 + (build.3/2));
                    let p_idx = build_data.map.xy_idx(point.x, point.y);
                    build_data.map.map[p_idx] = TileType::DownStairs;
                }
            }
            else {
                let index = rng.range(0, building_list_clone.len());
                building = RAWS.lock().unwrap().get_building_from_name(building_list_clone[index].to_string());
            }
                let mut tiles = HashSet::new();
                for y in build.1+1 .. build.1+build.3-1
                {
                    for x in build.0+1 .. build.0+build.2-1
                    {
                        tiles.insert(build_data.map.xy_idx(x, y));
                    }
                }
                console::log(format!("Length of tiles set {}", tiles.len()));
                spawn_building_contents(build_data, &building.contents, &mut tiles);
            
        }
    }

    fn place_doors(&self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap
        , buildings : &Vec<(i32,i32,i32,i32)>) -> Vec<usize>
    {
        let mut doors = Vec::new();

        //loop through every building to find door locations
        for (bx, by, bw, bh) in buildings.iter()
        {
            let side = Orientation::get_random(rng);
            let mut choices: Vec<usize> = Vec::new();
            match side 
            {
                Orientation::N => 
                {
                    choices = ((*bx .. *bx+*bw))
                        .into_iter()
                        .skip(1)
                        .map(|a| build_data.map.xy_idx(a, *by))
                        .collect();
                }
                Orientation::E => 
                {
                    choices = ((*by .. *by+*bh))
                        .into_iter()
                        .skip(1)
                        .map(|a| build_data.map.xy_idx((*bx+*bw)-1, a))
                        .collect();
                }
                Orientation::S => 
                {
                    choices = ((*bx .. *bx+*bw))
                        .into_iter()
                        .skip(1)
                        .map(|a| build_data.map.xy_idx(a, (*by+*bh)-1))
                        .collect();
                }
                Orientation::W => 
                {
                    choices = ((*by .. *by+*bh))
                        .into_iter()
                        .skip(1)
                        .map(|a| build_data.map.xy_idx(*bx, a))
                        .collect();
                }
            }

            let roll = rng.range(0, (choices.len()-1) as i32) as usize;

            let idx = choices[roll];

            build_data.map.map[idx] = TileType::RustedMetalFloor;
            build_data.spawn_list.push((idx, "Door".to_string()));

            doors.push(idx);
        }

        doors
    }

    fn spawn_townsfolk(&self, rng : &mut RandomNumberGenerator,build_data : &mut BuilderMap )
    {
        let mut valid_locations = find_entity_spawn_locations(build_data);
        let roll = rng.roll_dice(2, 6) +4;

        for i in 0..roll
        {
            if let Some(idx) = valid_locations.iter().next().cloned()
            {
                build_data.spawn_list.push((idx, "Citizen".to_string()));

                valid_locations.remove(&idx);
            }
            //let idx = valid_locations.iter().take(1).
        }
    }
}
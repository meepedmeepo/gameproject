use bracket_lib::{prelude::{DistanceAlg, Point}, random::RandomNumberGenerator};

use super::{BuilderMap, InitialMapBuilder, Map, TileType};


#[derive(PartialEq, Copy, Clone)]
pub enum DistanceAlgorithm {Pythagoras, Manhattan, Chebyshev}

pub struct VoronoiCellBuilder
{
    n_seeds : usize,
    distance_algorithm : DistanceAlgorithm,
    tile_to_draw : TileType
}

impl InitialMapBuilder for VoronoiCellBuilder
{
    fn build_map(&mut self, rng: &mut bracket_lib::prelude::RandomNumberGenerator, build_data: &mut super::BuilderMap) 
    {
        self.build(rng, build_data);
    }
}

impl VoronoiCellBuilder
{
    pub fn new() -> Box<VoronoiCellBuilder>
    {
        Box::new(VoronoiCellBuilder 
                {
                n_seeds: 64
                , distance_algorithm: DistanceAlgorithm::Chebyshev
                , tile_to_draw : TileType::Floor
                })
    }
    
    pub fn pythagoras() -> Box<VoronoiCellBuilder>
    {
        Box::new(VoronoiCellBuilder 
        {
            n_seeds: 64
            , distance_algorithm: DistanceAlgorithm::Pythagoras
            , tile_to_draw : TileType::Floor 
        })
    }

    pub fn manhattan() -> Box<VoronoiCellBuilder>
    {
        Box::new(VoronoiCellBuilder 
        {
            n_seeds: 64
            , distance_algorithm: DistanceAlgorithm::Manhattan,
            tile_to_draw : TileType::Floor 
        })
    }

    pub fn new_advanced(seeds : usize, algorithm : DistanceAlgorithm, tile : TileType) ->Box<VoronoiCellBuilder>
    {
        Box::new(VoronoiCellBuilder{
            n_seeds : seeds,
            distance_algorithm : algorithm,
            tile_to_draw : tile
        })
    }


    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap)
    {
        //Creates a voronoi diagram
        let mut voronoi_seeds : Vec<(usize, Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds
        {
            let vx = rng.roll_dice(1, build_data.map.map_width-1);
            let vy = rng.roll_dice(2, build_data.map.map_height-1);
            let vidx = build_data.map.xy_idx(vx, vy);
            let candidate = (vidx, Point::new(vx,vy));

            if !voronoi_seeds.contains(&candidate)
            {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance = vec![(0, 0.0f32) ; self.n_seeds];
        let mut voronoi_membership : Vec<i32> = vec![0 ; (build_data.map.map_width*build_data.map.map_height) as usize];

        for (i, vid) in voronoi_membership.iter_mut().enumerate()
        {
            let x = i as i32 % build_data.map.map_width;
            let y = i as i32 / build_data.map.map_width;

            for (seed, pos) in voronoi_seeds.iter().enumerate()
            {
                let distance;
                match self.distance_algorithm
                {
                    DistanceAlgorithm::Pythagoras =>
                    {
                        distance = DistanceAlg::PythagorasSquared.distance2d(Point::new(x, y), pos.1);
                    }
                    DistanceAlgorithm::Manhattan =>
                    {
                        distance = DistanceAlg::Manhattan.distance2d(Point::new(x, y), pos.1);
                    }
                    DistanceAlgorithm::Chebyshev =>
                    {
                        distance = DistanceAlg::Chebyshev.distance2d(Point::new(x, y), pos.1);
                    }
                }
                voronoi_distance[seed] = (seed,distance);
            }
            voronoi_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());
            
            *vid = voronoi_distance[0].0 as i32;
        }

        for y in 1.. build_data.map.map_height-1
        {
            for x in 1.. build_data.map.map_width-1
            {
                let mut neighbors = 0;
                let my_idx = build_data.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];

                if voronoi_membership[build_data.map.xy_idx(x-1, y)] != my_seed {neighbors += 1;}
                if voronoi_membership[build_data.map.xy_idx(x+1, y)] != my_seed {neighbors += 1;}
                if voronoi_membership[build_data.map.xy_idx(x, y-1)] != my_seed {neighbors += 1;}
                if voronoi_membership[build_data.map.xy_idx(x, y+1)] != my_seed {neighbors += 1;}

                if neighbors < 2
                {
                    build_data.map.map[my_idx] = self.tile_to_draw;
                }
            }
        }

    }
}
use std::slice::Chunks;

use bevy::prelude::*;
use bevy::ui::debug;
use rand::Rng;
use bevy::render::render_resource::PrimitiveTopology;
use libnoise::prelude::*;
use bevy::utils::HashMap;



use bevy::render::mesh::Indices;
pub struct WorldPlugin;


// BLOCK TYPES
const BLOCK_AIR : i32 = 0;
const BLOCK_SOLID : i32 = 1;

// CHUNK VARIABLES
const CHUNK_WIDTH : i32 = 24;
const CHUNK_HEIGHT : i32 = 256;

// TERRAIN VARIABLES
const GROUND_LEVEL : i32 = 100;
const AMPLITUDE : i32 = 10;
const SCALE : f64 = 0.02;
const RENDER_DISTANCE : i32 = 5;


struct Block {
    id: i32,
    block_type: i32,
}

impl Block {
    pub fn new(id: i32, block_type: i32) -> Self {
        Block { id, block_type }
    }
}

struct Chunk {
    id: i32,
    blocks: Vec<Block>,
    position: IVec2,
}

impl Chunk {
    pub fn new(id: i32, size: IVec3, position: IVec2) -> Self {
        let num_voxels: i32 = size.x * size.y * size.z;
        let mut blocks: Vec<Block> = Vec::with_capacity(num_voxels as usize);

        let mut block_ids : i32 = 0; 
    
        let mut noise: Simplex<2> = Source::simplex(2);

        for i in 0..num_voxels {
            let x: i32 = i % CHUNK_WIDTH;
            let z: i32 = (i % (CHUNK_WIDTH * CHUNK_WIDTH)) / CHUNK_WIDTH;
            let y: i32 = i / (CHUNK_WIDTH * CHUNK_WIDTH);
        
            blocks.push(Block::new(block_ids, get_block(x + position.x, y, z + position.y, &mut noise)));
            block_ids += 1;
        }

        Self { id, blocks, position }
    }
}

// Get the value of the given 2D noise at x, z and choose the corresponding block type
fn get_block(x: i32, y: i32, z: i32, noise: &mut Simplex<2>) -> i32 {
    let val : f64 = noise.sample([x as f64 * SCALE, z as f64 * SCALE]);

    let surface_y : i32 = (GROUND_LEVEL as f64 + (val * AMPLITUDE as f64)) as i32;
    
    if y < surface_y {
        return BLOCK_SOLID;
    } else {
        return BLOCK_AIR;
    }
}


// Generate all chunks in render distance
// A chunk combines multiple voxels and turns them into one mesh
fn spawn_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut chunks: Vec<Chunk> = Vec::new();

    let mut chunk_ids : i32 = 0;


    for x in 0..RENDER_DISTANCE {
        for z in 0..RENDER_DISTANCE {
            let position = IVec2::new(x as i32 * CHUNK_WIDTH, z as i32 * CHUNK_WIDTH);
            let size: IVec3 = IVec3::new(CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_WIDTH);
            let chunk = Chunk::new(chunk_ids, size, position);
            chunk_ids += 1;
            
            chunks.push(chunk);
        }
    }

    for chunk in &chunks{
        // Find neighboring chunks via position (distance from each other) and put it into a HashMap
        let mut neighbors_by_direction: HashMap<&'static str, &Chunk> = HashMap::new();

        chunks.iter().for_each(|other_chunk| {
        let distance_x = chunk.position.x - other_chunk.position.x;
        let distance_y = chunk.position.y - other_chunk.position.y;
        
        if !(distance_x == 0 && distance_y == 0) {

            if distance_x == CHUNK_WIDTH && distance_y == 0 {
                neighbors_by_direction.entry("left").or_insert(other_chunk);
            } else 
            if distance_x == 0 && distance_y == CHUNK_WIDTH {
                neighbors_by_direction.entry("top").or_insert(other_chunk);
            }else 
            if distance_x == -1*CHUNK_WIDTH && distance_y == 0 {
                neighbors_by_direction.entry("right").or_insert(other_chunk);
            }else 
            if distance_x == 0 && distance_y <= -1*CHUNK_WIDTH {
                neighbors_by_direction.entry("down").or_insert(other_chunk);
            }
        }
    });

        let cube_mesh: Handle<Mesh> = create_cube_mesh(&mut meshes, &chunk, &mut neighbors_by_direction);
        let random_hue: f32 = rand::thread_rng().gen_range(0.5..=1.0);
        let material = materials.add(Color::rgb(0.0, 0.0, random_hue).into());
        let cube = PbrBundle {
            mesh: cube_mesh,
            material: material.clone(),
            transform: Transform::from_xyz(chunk.position.x as f32 , 0 as f32, chunk.position.y as f32),
            ..default()
        };
        commands.spawn(cube);
    }

}


fn create_cube_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    chunk: &Chunk,
    neighbors_by_direction: &mut HashMap<&'static str, &Chunk>,
) -> Handle<Mesh> {
    
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut vfaces: Vec<usize> = Vec::new();
    let mut colors: Vec<Vec4> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();

    let num_voxels: i32 = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;
    let num_voxel_per_row : i32 = CHUNK_WIDTH * CHUNK_WIDTH;


    // Check for neighboring voxels, to hide faces
    for i in 0..num_voxels {
        let x: i32 = i % CHUNK_WIDTH;
        let z: i32 = (i % (CHUNK_WIDTH * CHUNK_WIDTH)) / CHUNK_WIDTH;
        let y: i32 = i / (CHUNK_WIDTH * CHUNK_WIDTH);

        if chunk.blocks[i as usize].block_type == BLOCK_AIR { continue; }

        // X Direction------------------------------
        if (i + 1 < num_voxels && (i + 1) % CHUNK_WIDTH != 0) && chunk.blocks[(i + 1) as usize].block_type == BLOCK_AIR {
            vfaces.push(0);
        }

        // X Direction right chunk neighbor if necessary
        else if (i + 1) % CHUNK_WIDTH == 0 && neighbors_by_direction.contains_key("right") {

            if neighbors_by_direction.get("right").unwrap().blocks[(i - CHUNK_WIDTH + 1) as usize].block_type == BLOCK_AIR{

                vfaces.push(0);
            }
        }
        
        // -X Direction------------------------------
        if (i > 0 && i % CHUNK_WIDTH != 0) && chunk.blocks[(i - 1) as usize].block_type == BLOCK_AIR{
            vfaces.push(1);
        }

        // X Direction left chunk neighbor if necessary
        else if (i % CHUNK_WIDTH == 0) && neighbors_by_direction.contains_key("left") {

            if neighbors_by_direction.get("left").unwrap().blocks[(i + CHUNK_WIDTH - 1) as usize].block_type == BLOCK_AIR{

                vfaces.push(1);
            }
        }
        


        // Y Direction ------------------------------
        // (not necessary to check for neighbor because no chunk is on top of each other)
        if ((i + num_voxel_per_row < num_voxels) && chunk.blocks[(i + num_voxel_per_row) as usize].block_type == BLOCK_AIR)
        || (i + num_voxel_per_row >= num_voxels) {
            vfaces.push(2);
        }
        
        // -Y Direction ------------------------------
        if (i - num_voxel_per_row >= 0) && chunk.blocks[(i - num_voxel_per_row) as usize].block_type == BLOCK_AIR {
            vfaces.push(3);
        }

        


        // Z Direction------------------------------
        if (i + CHUNK_WIDTH < num_voxels && i / num_voxel_per_row == (i + CHUNK_WIDTH) / num_voxel_per_row) && chunk.blocks[(i + CHUNK_WIDTH) as usize].block_type == BLOCK_AIR {
            vfaces.push(4);
        }
   
        // Z Direction down chunk neighbor if necessary
        else if (i / num_voxel_per_row != (i + CHUNK_WIDTH) / num_voxel_per_row) && neighbors_by_direction.contains_key("down") {

            if neighbors_by_direction.get("down").unwrap().blocks[(i - CHUNK_WIDTH * (CHUNK_WIDTH - 1)) as usize].block_type == BLOCK_AIR{

                vfaces.push(4);
            }
        }


        // -Z Direction------------------------------
        if (i - CHUNK_WIDTH >= 0 && i / num_voxel_per_row == (i - CHUNK_WIDTH) / num_voxel_per_row) && chunk.blocks[(i - CHUNK_WIDTH) as usize].block_type == BLOCK_AIR {
            vfaces.push(5);
        }

        // Z Direction top chunk neighbor if necessary
        else if (i / num_voxel_per_row != (i - CHUNK_WIDTH) / num_voxel_per_row) && neighbors_by_direction.contains_key("top") {

            if neighbors_by_direction.get("top").unwrap().blocks[(i + CHUNK_WIDTH * (CHUNK_WIDTH - 1)) as usize].block_type == BLOCK_AIR{

                vfaces.push(5);
            }
        }




        // Generate geometry data of 1 voxel which is part of the chunk 
        generate_cube(&mut vertices, &mut indices, &mut vfaces, &mut normals, &mut colors, 
            x as usize, y as usize, z as usize);

        vfaces.clear();
    }
    

    // Generate chunk-mesh with the vertices and indices provided by the voxel data
    meshes.add(Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute( Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute( Mesh::ATTRIBUTE_NORMAL, normals)
        // .with_inserted_attribute( Mesh::ATTRIBUTE_COLOR, colors)
        .with_indices(Some(Indices::U32(indices)))
    )
}

// Generate one voxel with visible faces of the chunk-mesh
fn generate_cube(
    vertices: &mut Vec<Vec3>,
    indices: &mut Vec<u32>,
    vfaces: &mut Vec<usize>,
    normals: &mut Vec<Vec3>,
    colors: &mut Vec<Vec4>,
    x: usize,
    y: usize,
    z: usize,
) {

    // Array for all possible vertex position of a Voxel
    let positions: [[Vec3; 4];6]  = 
    [[
         // X Direction Position
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),             
        Vec3::new(0.5, 0.5, 0.5),             
        Vec3::new(0.5, -0.5, 0.5), 
        ],[
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, -0.5), 
        Vec3::new(-0.5, -0.5, 0.5),
        ],[
        // Y Direction Position
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5), 
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        ],[
        Vec3::new(-0.5, -0.5, -0.5), 
        Vec3::new(0.5, -0.5, -0.5),  
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        ],[
        // Z Direction Position
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5), 
        ],[
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5), 
        Vec3::new(-0.5, -0.5, -0.5),
    ]];

    // Array for all possible normal direction of a Voxel 
    // TODO: Simplify normal data 
    let normal: [[Vec3; 4];6]  = 
    [[
        // X Direction Normals
        Vec3::new( 1.0 , 0.0 , 0.0),
        Vec3::new( 1.0 , 0.0 , 0.0),       
        Vec3::new( 1.0 , 0.0 , 0.0),            
        Vec3::new( 1.0, 0.0 , 0.0 ),
        ],[
        Vec3::new( -1.0 , 0.0 , 0.0),
        Vec3::new( -1.0 , 0.0 , 0.0),       
        Vec3::new( -1.0 , 0.0 , 0.0),            
        Vec3::new( -1.0, 0.0 , 0.0 ),
        ],[
        // Y Direction Normals
        Vec3::new( 0.0 , 1.0 , 0.0),
        Vec3::new( 0.0, 1.0 , 0.0),       
        Vec3::new( 0.0, 1.0 , 0.0),            
        Vec3::new( 0.0, 1.0 , 0.0 ),
        ],[
        Vec3::new( 0.0 , -1.0 , 0.0),
        Vec3::new( 0.0 , -1.0  , 0.0),       
        Vec3::new( 0.0 , -1.0  , 0.0),            
        Vec3::new( 0.0, -1.0  , 0.0 ),
        ],[
        // Z Direction Normals
        Vec3::new( 0.0 , 0.0 , 1.0),
        Vec3::new( 0.0 , 0.0 , 1.0),       
        Vec3::new( 0.0 , 0.0 , 1.0),            
        Vec3::new( 0.0 , 0.0 , 1.0),
        ],[
        Vec3::new( 0.0 , 0.0 , -1.0),
        Vec3::new( 0.0 , 0.0 , -1.0),       
        Vec3::new( 0.0 , 0.0 , -1.0),            
        Vec3::new( 0.0 , 0.0 , -1.0),
    ]];


    let random_hue: f32 = rand::thread_rng().gen_range(0.0..=1.0);
    let mut colorcube : Vec4 = Vec4::new(0.0,0.0,random_hue,1.0);

    // For loop for each face to render
    for i in vfaces {
        
        let base_index: u32 = vertices.len() as u32;

        // Push all corresponded vertices of the face
        vertices.extend_from_slice(
            &mut positions[*i]
                .iter()
                .map(|&position| position + Vec3::new(x as f32, y as f32, z as f32))
                .collect::<Vec<_>>(),
        );

        // Push all corresponded Normals  of the face
        normals.extend_from_slice(&normal[*i]);
        
        // Push all indices of the face
        indices.append(&mut vec![base_index + 0 as u32, base_index + 1 as u32, base_index + 2 as u32, base_index + 2 as u32, base_index + 3 as u32, base_index + 0]);
        colors.push(colorcube);
        colors.push(colorcube);
        colors.push(colorcube);
        colors.push(colorcube);
        }
    }




impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_chunks);
    }
}

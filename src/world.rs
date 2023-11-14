use std::slice::Chunks;

use bevy::prelude::*;
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
const CHUNK_WIDTH : i32 = 50;
const CHUNK_HEIGHT : i32 = 100;

// TERRAIN VARIABLES
const GROUND_LEVEL : i32 = 100;
const AMPLITUDE : i32 = 20;
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
    size: IVec3,
    blocks: Vec<Vec<Vec<Block>>>,
    position: IVec2,
}

impl Chunk {
    pub fn new(id: i32, size: IVec3, position: IVec2) -> Self {
        let num_voxels_x: i32 = size.x;
        let num_voxels_y: i32 = size.y;
        let num_voxels_z: i32 = size.z;

        let mut blocks: Vec<Vec<Vec<Block>>> = Vec::with_capacity(num_voxels_x as usize);
        let mut block_ids : i32 = 0; 


        let mut noise: Simplex<2> = Source::simplex(4);

        for _x in 0..num_voxels_x {
            let mut row: Vec<Vec<Block>> = Vec::with_capacity(num_voxels_y as usize);
            
            for _y in 0..num_voxels_y {
                let mut col: Vec<Block> = Vec::with_capacity(num_voxels_z as usize);

                for _z in 0..num_voxels_z {
                    col.push(Block::new(block_ids, get_block(_x + position.x, _y, _z + position.y, &mut noise)));
                }

                row.push(col);
            }

            blocks.push(row);
        }

        Self { id, size, blocks, position }
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

    // Check for neighboring voxels, to hide faces
    for (x, row) in chunk.blocks.iter().enumerate() {
        for (y, col) in row.iter().enumerate() {
            for (z, voxel) in col.iter().enumerate() {
                if voxel.block_type == BLOCK_AIR { continue; }

                // X Direction------------------------------
                if (x + 1 < CHUNK_WIDTH as usize && chunk.blocks[x+1][y][z].block_type == BLOCK_AIR){

                    vfaces.push(0);
                }
                // X Direction right chunk neighbor if necessary
                else if x + 1 ==  CHUNK_WIDTH as usize && neighbors_by_direction.contains_key("right") {

                    if neighbors_by_direction.get("right").unwrap().blocks[0][y][z].block_type == BLOCK_AIR{

                        vfaces.push(0);
                    }
                }

                // -X Direction------------------------------
                if x != 0 && chunk.blocks[x-1][y][z].block_type == BLOCK_AIR{

                    vfaces.push(1);
                }
                // X Direction left chunk neighbor if necessary
                else if x == 0 && neighbors_by_direction.contains_key("left") {

                    if neighbors_by_direction.get("left").unwrap().blocks[CHUNK_WIDTH as usize-1][y][z].block_type == BLOCK_AIR{

                        vfaces.push(1);
                    }
                }
                
                // Y Direction ------------------------------
                // (not necessary to check for neighbor because no chunk is on top of each other)
                if (y + 1 < CHUNK_HEIGHT as usize && chunk.blocks[x][y+1][z].block_type == BLOCK_AIR) 
                    || y == CHUNK_HEIGHT as usize - 1 {

                    vfaces.push(2);
                }
                // -Y Direction ------------------------------
                if (y != 0 && chunk.blocks[x][y-1][z].block_type == BLOCK_AIR) 
                    || y == 0 {

                    vfaces.push(3);
                }
                
                // Z Direction------------------------------
                if (z + 1 < CHUNK_WIDTH as usize && chunk.blocks[x][y][z+1].block_type == BLOCK_AIR) {

                    vfaces.push(4);
                }
                // Z Direction down chunk neighbor if necessary
                else if z + 1 ==  CHUNK_WIDTH as usize && neighbors_by_direction.contains_key("down") {

                    if neighbors_by_direction.get("down").unwrap().blocks[x][y][0].block_type == BLOCK_AIR{

                        vfaces.push(4);
                    }
                }
                // -Z Direction------------------------------
                if z != 0 && chunk.blocks[x][y][z-1].block_type == BLOCK_AIR {
                        
                    vfaces.push(5);
                }
                // Z Direction top chunk neighbor if necessary
                else if z == 0 && neighbors_by_direction.contains_key("top") {

                    if neighbors_by_direction.get("top").unwrap().blocks[x][y][CHUNK_WIDTH as usize-1].block_type == BLOCK_AIR{

                        vfaces.push(5);
                    }
                }

                // Generate geometry data of 1 voxel which is part of the chunk 
                generate_cube(&mut vertices, &mut indices, &mut vfaces,&mut normals,&mut colors,x,y,z);
                vfaces.clear();
                
            }      
        }
    }
    

    // Generate chunk-mesh with the vertices and indices provided by the voxel data
    meshes.add(Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute( Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute( Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute( Mesh::ATTRIBUTE_COLOR, colors)
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
        let base_index: u32 = vertices.len() as u32;
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

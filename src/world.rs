use bevy::prelude::*;
use rand::Rng;
use bevy::render::render_resource::PrimitiveTopology;

use bevy::render::mesh::Indices;
pub struct WorldPlugin;

const BLOCK_AIR : i32 = 0;
const BLOCK_SOLID : i32 = 1;

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
}

impl Chunk {
    pub fn new(id: i32, size: IVec3) -> Self {
        let num_voxels_x = size.x;
        let num_voxels_y = size.y;
        let num_voxels_z = size.z;
        
        let mut blocks = Vec::with_capacity(num_voxels_x as usize);
        let mut block_ids : i32 = 0; 
        
        let mut rng = rand::thread_rng();

        for _x in 0..num_voxels_x {
            let mut row = Vec::with_capacity(num_voxels_y as usize);
            
            for _y in 0..num_voxels_y {
                let mut col = Vec::with_capacity(num_voxels_z as usize);

                for _z in 0..num_voxels_z {
                    
                    if rng.gen_range(0..=1) == 0 {
                        col.push(Block::new(block_ids, BLOCK_AIR));
                    } else {
                        col.push(Block::new(block_ids, BLOCK_SOLID));
                    }

                    block_ids += 1;
                }

                row.push(col);
            }

            blocks.push(row);
        }

        Self { id, size, blocks }
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_chunk);
    }
}

fn spawn_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size: IVec3 = IVec3::new(16, 100, 16);
    let chunk = Chunk::new(0, size);

    let cube_mesh: Handle<Mesh> = create_cube_mesh(&mut meshes, &chunk);
    let material = materials.add(Color::BLUE.into());


    let cube = PbrBundle {
        mesh: cube_mesh,
        material: material.clone(),
        transform: Transform::from_xyz(0 as f32 ,0 as f32,0 as f32),
        ..default()
    };

    commands.spawn(cube);
}


fn create_cube_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    chunk: &Chunk,
) -> Handle<Mesh> {
    
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut vfaces: Vec<u32> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();

    // Check for Neighbouring Voxels, to hide Faces
    for (x, row) in chunk.blocks.iter().enumerate() {
        for (y, col) in row.iter().enumerate() {
            for (z, voxel) in col.iter().enumerate() {
                if voxel.block_type == BLOCK_AIR { continue; }

                // X Direction
                if (x + 1 < chunk.blocks.len() && chunk.blocks[x+1][y][z].block_type == BLOCK_AIR) 
                    || x == chunk.blocks.len() - 1 {

                    let elements_to_push: Vec<u32> = vec![0, 1, 2, 2, 3, 0];
                    vfaces.extend(elements_to_push);
                }
                
                if (x != 0 && chunk.blocks[x-1][y][z].block_type == BLOCK_AIR) 
                    || x == 0 {

                    let elements_to_push: Vec<u32> = vec![4, 5, 6, 6, 7, 4];
                    vfaces.extend(elements_to_push);
                }
                
                // Y Direction
                if (y + 1 < chunk.blocks[x].len() && chunk.blocks[x][y+1][z].block_type == BLOCK_AIR) 
                    || y == chunk.blocks[x].len() - 1 {

                    let elements_to_push: Vec<u32> = vec![8, 9, 10, 10, 11, 8];
                    vfaces.extend(elements_to_push);
                }
                
                if (y != 0 && chunk.blocks[x][y-1][z].block_type == BLOCK_AIR) 
                    || y == 0 {

                    let elements_to_push: Vec<u32> = vec![12, 13, 14, 14, 15, 12];
                    vfaces.extend(elements_to_push);
                }
                
                // Z Direction
                if (z + 1 < chunk.blocks[x][y].len() && chunk.blocks[x][y][z+1].block_type == BLOCK_AIR) 
                    || z == chunk.blocks[x][y].len() - 1 {

                    let elements_to_push: Vec<u32> = vec![16, 17, 18, 18, 19, 16];
                    vfaces.extend(elements_to_push);
                }

                if (z != 0 && chunk.blocks[x][y][z-1].block_type == BLOCK_AIR) 
                    || z == 0 {
                    let elements_to_push: Vec<u32> = vec![20, 21, 22, 22, 23, 20];
                    vfaces.extend(elements_to_push);
                }

                generate_cube(&mut vertices, &mut indices, &mut vfaces,&mut normals,x,y,z);
                vfaces.clear();
                
            }      
        }
    }
    

    // Generate cube vertices and indices based on voxel data
    meshes.add(Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute( Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute( Mesh::ATTRIBUTE_NORMAL, normals)
        .with_indices(Some(Indices::U32(indices)))
    )
}

fn generate_cube(
    vertices: &mut Vec<Vec3>,
    indices: &mut Vec<u32>,
    vfaces: &mut Vec<u32>,
    normals: &mut Vec<Vec3>,
    x: usize,
    y: usize,
    z: usize,
) {
    let positions: [Vec3; 24] = [
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),             
        Vec3::new(0.5, 0.5, 0.5),             
        Vec3::new(0.5, -0.5, 0.5), 

        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, -0.5), 
        Vec3::new(-0.5, -0.5, 0.5),

        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5), 
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),

        Vec3::new(-0.5, -0.5, -0.5), 
        Vec3::new(0.5, -0.5, -0.5),  
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(-0.5, -0.5, 0.5),

        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5), 

        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5), 
        Vec3::new(-0.5, -0.5, -0.5),
    ];

    let normal: [Vec3; 24] = [
        
        Vec3::new( 1.0 , 0.0 , 0.0),
        Vec3::new( 1.0 , 0.0 , 0.0),       
        Vec3::new( 1.0 , 0.0 , 0.0),            
        Vec3::new( 1.0, 0.0 , 0.0 ),

        Vec3::new( -1.0 , 0.0 , 0.0),
        Vec3::new( -1.0 , 0.0 , 0.0),       
        Vec3::new( -1.0 , 0.0 , 0.0),            
        Vec3::new( -1.0, 0.0 , 0.0 ),
	
        Vec3::new( 0.0 , 1.0 , 0.0),
        Vec3::new( 0.0, 1.0 , 0.0),       
        Vec3::new( 0.0, 1.0 , 0.0),            
        Vec3::new( 0.0, 1.0 , 0.0 ),

        Vec3::new( 0.0 , -1.0 , 0.0),
        Vec3::new( 0.0 , -1.0  , 0.0),       
        Vec3::new( 0.0 , -1.0  , 0.0),            
        Vec3::new( 0.0, -1.0  , 0.0 ),

        Vec3::new( 0.0 , 0.0 , 1.0),
        Vec3::new( 0.0 , 0.0 , 1.0),       
        Vec3::new( 0.0 , 0.0 , 1.0),            
        Vec3::new( 0.0 , 0.0 , 1.0),

        Vec3::new( 0.0 , 0.0 , -1.0),
        Vec3::new( 0.0 , 0.0 , -1.0),       
        Vec3::new( 0.0 , 0.0 , -1.0),            
        Vec3::new( 0.0 , 0.0 , -1.0),
        
    ];
    
    for normal in normal.iter() {
        normals.push(*normal);
    }

    let base_index = vertices.len();

    for position in positions.iter() {
        vertices.push(*position + Vec3::new(x as f32, y as f32, z as f32));
    }

    for &index in vfaces.iter() {
        indices.push(base_index as u32 + index);
    }
    
    
}

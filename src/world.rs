use bevy::prelude::*;
use rand::Rng;
use bevy::render::render_resource::PrimitiveTopology;

use bevy::render::mesh::Indices;
pub struct WorldPlugin;

struct Block {
    name: String,
    id: i32,
}
impl Block {
    pub fn new(name: String,id: i32) -> Self {
        Block {name,id,}
    }
}
struct Chunk {
    name: String,
    size: usize,
    blocks: Vec<Vec<Vec<Block>>>,
}

impl Chunk {
    pub fn new(name: String, size: usize) -> Self {
        let mut blocks = Vec::with_capacity(size);
        let mut rng = rand::thread_rng();

        for i in 0..size {
            let mut row = Vec::with_capacity(size);
            for j in 0..size {
                let mut col = Vec::with_capacity(size);
                for k in 0..size {
                    col.push(Block::new("test".to_owned(), rng.gen_range(0..=1)));
                }
                row.push(col);
            }
            blocks.push(row);
        }

        Self { name, size, blocks }
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_cubes);
    }
}

fn spawn_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size: usize = 50;
    let chunk = Chunk::new("test".to_owned(),size);

    let cube_mesh: Handle<Mesh> = create_cube_mesh(&mut meshes, &chunk);
    let cube = PbrBundle {
    mesh: cube_mesh,
    material: materials.add(Color::DARK_GREEN.into()),
    transform: Transform::from_xyz(0 as f32 ,0 as f32,0 as f32),
    ..default()
    };
    commands.spawn(cube);


}


fn create_cube_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    voxel_data: &Chunk,
) -> Handle<Mesh> {
    
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut vfaces: Vec<u32> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    
    for (x, row) in voxel_data.blocks.iter().enumerate() {
        for (y, col) in row.iter().enumerate() {
            for (z, voxel) in col.iter().enumerate() {
                if voxel.id !=0 {
                    if (x+1 < voxel_data.blocks.len() && voxel_data.blocks[x+1][y][z].id == 0) || x == voxel_data.blocks.len()-1{
                        let elements_to_push: Vec<u32> = vec![0, 1, 2, 2, 3, 0];
                        vfaces.extend(elements_to_push);
                    }
                    if (x != 0 && voxel_data.blocks[x-1][y][z].id == 0) || x == 0{
                        let elements_to_push: Vec<u32> = vec![4, 5, 6, 6, 7, 4];//
                        vfaces.extend(elements_to_push);
                    }
                    if (y+1 <voxel_data.blocks.len() && voxel_data.blocks[x][y+1][z].id == 0) || y == voxel_data.blocks.len()-1{
                        let elements_to_push: Vec<u32> = vec![8, 9, 10, 10, 11, 8];
                        vfaces.extend(elements_to_push);
                    }
                    if (y != 0 && voxel_data.blocks[x][y-1][z].id == 0) || y == 0{
                        let elements_to_push: Vec<u32> = vec![12, 13, 14, 14, 15, 12];//
                        vfaces.extend(elements_to_push);
                    }
                    if (z+1 < voxel_data.blocks.len() && voxel_data.blocks[x][y][z+1].id == 0) || z == voxel_data.blocks.len()-1{
                        let elements_to_push: Vec<u32> = vec![16, 17, 18, 18, 19, 16];//
                        vfaces.extend(elements_to_push);
                    }
                    if (z != 0 && voxel_data.blocks[x][y][z-1].id == 0) || z == 0{
                        let elements_to_push: Vec<u32> = vec![20, 21, 22, 22, 23, 20];//
                        vfaces.extend(elements_to_push);
                    }
                    generate_cube(&mut vertices, &mut indices, &mut vfaces,&mut normals,x,y,z);
                    vfaces.clear();
                }
                
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

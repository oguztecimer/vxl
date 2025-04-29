use fastnoise_lite::{FastNoiseLite, NoiseType};
use glam::IVec3;

const MIN_HEIGHT: i32 = 1;
const MAX_HEIGHT: i32 = 30;
const CHUNK_SIDE_SIZE: i32 = 32;
const CHUNK_SIDE_SIZE_SQR: i32 = CHUNK_SIDE_SIZE * CHUNK_SIDE_SIZE;
const CHUNK_SIZE: i32 = CHUNK_SIDE_SIZE * CHUNK_SIDE_SIZE * CHUNK_SIDE_SIZE;
pub struct Chunk {
    pub texture: Vec<u8>,
    pub position: IVec3,
}

impl Chunk {
    pub fn new(position: IVec3) -> Option<Box<Self>> {
        let mut texture = vec![0; CHUNK_SIZE as usize];

        let mut noise = FastNoiseLite::with_seed(1944);
        noise.set_noise_type(Some(NoiseType::OpenSimplex2));
        noise.set_frequency(Some(0.05));
        let mut not_empty = false;
        for x in 0..CHUNK_SIDE_SIZE {
            let x_coord = position.x * CHUNK_SIDE_SIZE + x;
            for y in 0..CHUNK_SIDE_SIZE {
                let y_coord = position.y * CHUNK_SIDE_SIZE + y;
                let mut value = noise.get_noise_2d(x_coord as f32, y_coord as f32);
                value += 1.0;
                value /= 2.0;
                value *= (MAX_HEIGHT - MIN_HEIGHT) as f32;
                value += MIN_HEIGHT as f32;
                let z_start = position.z * CHUNK_SIDE_SIZE;
                if value < z_start as f32 {
                    continue;
                }
                not_empty = true;
                for z in 0..CHUNK_SIDE_SIZE {
                    let z_coord = z_start + z;
                    if z_coord as f32 <= value {
                        let texture_index = x + y * CHUNK_SIDE_SIZE + z * CHUNK_SIDE_SIZE_SQR;
                        texture[texture_index as usize] = 1;
                    } else {
                        break;
                    }
                }
            }
        }
        if !not_empty {
            None
        } else {
            Some(Box::from(Self { texture, position }))
        }
    }
}

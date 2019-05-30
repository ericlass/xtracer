use settings::Color;
use std::fs::File;
use std::io::prelude::Read;
use std::string::String;
use std::u32;

pub struct VoxelObject {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub data: Vec<Option<Color>>,
}

impl VoxelObject {
    pub fn get(&self, x: u32, y: u32, z: u32) -> &Option<Color> {
        if x >= self.width || y >= self.height || z >= self.depth {
            println!("Out of bounds: {} {} {}", x, y, z);
            panic!("AHH!");
        }

        let index = self.index(x, y, z);
        &self.data[index]
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, color: Color) {
        let index = self.index(x, y, z);
        self.data[index] = Some(color);
    }

    fn index(&self, x: u32, y: u32, z: u32) -> usize {
        ((z * self.width * self.height) + (y * self.width) + x) as usize
    }
}

pub fn read_voxels(file_name: &str) -> Option<VoxelObject> {
    let mut file = File::open(file_name).unwrap();

    //Read and check file header
    let (name, version) = read_file_header(&mut file);
    assert!(name == "VOX ");
    assert!(version == 150);

    let (name, _content_bytes, _child_bytes) = read_chunk_header(&mut file);
    assert!(name == "MAIN");

    let (name, _content_bytes, _child_bytes) = read_chunk_header(&mut file);
    assert!(name == "SIZE");

    let (sx, sy, sz) = read_size_chunk(&mut file);
    println!("Voxel model size: {}x{}x{}", sx, sy, sz);

    let (name, _content_bytes, _child_bytes) = read_chunk_header(&mut file);
    assert!(name == "XYZI");

    let num_voxels = (sx * sy * sz) as usize;
    let mut result = VoxelObject {
        width: sx,
        height: sy,
        depth: sz,
        data: vec![None; num_voxels],
    };

    let voxels_read = read_xyzi_chunk(&mut file, &mut result);
    dbg!(voxels_read);

    Some(result)
}

fn read_file_header(file: &mut File) -> (String, u32) {
    let name = String::from_utf8_lossy(&read_four_bytes(file)).into_owned();
    let version = u32::from_le_bytes(read_four_bytes(file));

    (name, version)
}

fn read_chunk_header(file: &mut File) -> (String, u32, u32) {
    let name = String::from_utf8_lossy(&read_four_bytes(file)).into_owned();
    let chunk_bytes = u32::from_le_bytes(read_four_bytes(file));
    let child_bytes = u32::from_le_bytes(read_four_bytes(file));

    (name, chunk_bytes, child_bytes)
}

fn read_size_chunk(file: &mut File) -> (u32, u32, u32) {
    let sx = u32::from_le_bytes(read_four_bytes(file));
    let sy = u32::from_le_bytes(read_four_bytes(file));
    let sz = u32::from_le_bytes(read_four_bytes(file));

    (sx, sy, sz)
}

fn read_xyzi_chunk(file: &mut File, vox: &mut VoxelObject) -> u32 {
    let num_voxels = u32::from_le_bytes(read_four_bytes(file));

    for i in 0..num_voxels {
        let bytes = read_four_bytes(file);
        let x = bytes[0] as u32;
        let y = bytes[1] as u32;
        let z = bytes[2] as u32;
        //let c = bytes[3] as u32;

        vox.set(x, y, z, Color::white());
    }

    num_voxels
}

fn read_four_bytes(file: &mut File) -> [u8; 4] {
    let mut buffer = [0; 4];
    file.read(&mut buffer).unwrap();
    buffer
}

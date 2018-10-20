use std::fs::File;
use std::io::Write;

pub fn write_tga(filename: &str, width: u16, height: u16, pixels: &[u8]) {
  let mut file = File::create(filename).unwrap();

  //Size of image ID field. 0 means no ID.
  file.write_all(&[0 as u8]).unwrap();
  //Color map type. 0 means to color map
  file.write_all(&[0 as u8]).unwrap();
  //Image type code, 2 means raw RGB
  file.write_all(&[2 as u8]).unwrap();

  //Color map origin, not used
  file.write_all(&u16_to_bytes(0 as u16)).unwrap();
  //Color map length, not used
  file.write_all(&u16_to_bytes(0 as u16)).unwrap();
  //Color map entry size, not used
  file.write_all(&[0 as u8]).unwrap();

  //X origin of image
  file.write_all(&u16_to_bytes(0 as u16)).unwrap();
  //Y origin of image
  file.write_all(&u16_to_bytes(0 as u16)).unwrap();
  //Width of image
  file.write_all(&u16_to_bytes(width)).unwrap();
  //Height of image
  file.write_all(&u16_to_bytes(height)).unwrap();
  //Bits per pixel
  file.write_all(&[24 as u8]).unwrap();
  //Image descriptor byte, always 0
  file.write_all(&[0 as u8]).unwrap();

  //Write pixel data
  file.write_all(pixels).unwrap();
  file.flush().unwrap();
}

fn u16_to_bytes(v: u16) -> [u8; 2] {
  let mut result: [u8; 2] = [0; 2];

  result[0] = v as u8;
  result[1] = (v >> 8) as u8;

  result
}

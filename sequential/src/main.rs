use image::io::Reader as ImageReader;
use image::GenericImageView;
use std::collections::HashMap;
use std::fs;

fn main() {
    let paths = fs::read_dir("../images").unwrap();
    let mut images: HashMap<String, [[u32; 256]; 3]> = HashMap::new();

    for path in paths {
        let filepath = path.unwrap().path();
        let filename_str = filepath.display().to_string();
        let filename = filename_str.split('/').last().unwrap().to_owned();
        let img: image::DynamicImage = ImageReader::open(filepath).unwrap().decode().unwrap();
        let mut sum_red: [u32; 256] = [0; 256];
        let mut sum_green: [u32; 256] = [0; 256];
        let mut sum_blue: [u32; 256] = [0; 256];
        for (_, _, pixel) in img.pixels() {
            sum_red[pixel[0] as usize] += 1;
            sum_green[pixel[1] as usize] += 1;
            sum_blue[pixel[2] as usize] += 1;
        }

        images.insert(filename, [sum_red, sum_green, sum_blue]);
    }

    // println!("{:?}", images)
}

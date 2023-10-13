use image::io::Reader as ImageReader;
use image::GenericImageView;
use std::collections::HashMap;
use std::fs;

fn main() {
    let paths = fs::read_dir("../images").unwrap();
    let mut images: HashMap<String, [u32; 3]> = HashMap::new();

    for path in paths {
        let filepath = path.unwrap().path();
        let filename_str = filepath.display().to_string();
        let filename = filename_str.split('/').last().unwrap().to_owned();
        let img: image::DynamicImage = ImageReader::open(filepath).unwrap().decode().unwrap();
        let mut sum_red = 0;
        let mut sum_green = 0;
        let mut sum_blue = 0;
        for (_, _, pixel) in img.pixels() {
            sum_red += pixel[0] as u32;
            sum_green += pixel[1] as u32;
            sum_blue += pixel[2] as u32;
        }
        images.insert(filename, [sum_red, sum_green, sum_blue]);
    }

    println!("{:#?}", images)
}

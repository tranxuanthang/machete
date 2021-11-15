use serde_json::json;
use std::path::Path;
use clap::{App, load_yaml};
use image::ImageBuffer;
use image::imageops;

fn main() {
    let (input_file, output_directory, color_threshold, min_height, max_height) = parse_args();

    let img_read = image::open(input_file);
    let mut img = match img_read {
        Ok(img) => img.to_rgb8(),
        Err(e) => {
            panic!("Problem creating the file: {:?}", e)
        }
    };
    let (width, height) = img.dimensions();

    let split_pos = find_split_pos(width, height, color_threshold, min_height, max_height, &img);
    let results = split(&output_directory, &split_pos, &mut img, width);

    println!("{}", json!({
        "status": "processed",
        "result": results
    }));
}

fn split(output_directory: &str, cut_rows: &Vec<u32>, img: &mut ImageBuffer<image::Rgb<u8>, Vec<u8>>, width: u32) -> Vec<String> {
    let image_dir = Path::new(&output_directory);
    let mut last_row = 0;

    let mut results: Vec<String> = Vec::new();

    for (i, x) in cut_rows.iter().enumerate() {
        let crop_height = x - last_row;
        let cropped_img = imageops::crop(img, 0, last_row, width, crop_height).to_image();

        let mut filename = format!("{:0>3}", i);
        filename.push_str(".jpg");

        let image_path = image_dir.join(filename);
        let image_path_str = image_path.to_str().unwrap().to_string();
        let result = cropped_img.save_with_format(image_path, image::ImageFormat::Jpeg);
        match result {
            Ok(_) => results.push(image_path_str),
            Err(e) => panic!("Problem saving the result: {:?}", e)
        }
        last_row = *x;
    }

    results
}

fn find_split_pos(width: u32, height: u32, color_threshold: u8, min_height: u32, max_height: u32, img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Vec<u32> {
    let check_step = 40;
    let mut x = min_height;
    let mut current_height = min_height;
    let mut cut_rows: Vec<u32> = Vec::new();

    while x < height {
        let is_row_solid = check_row_solid(&img, x, width, color_threshold);
        if is_row_solid {
            cut_rows.push(x);
            x += min_height;
            current_height = min_height;
        } else if current_height + check_step > max_height {
            cut_rows.push(x);
            x += min_height;
            current_height = min_height;
        } else {
            x += check_step;
            current_height += check_step;
        }
    }

    cut_rows.push(height);
    cut_rows
}

fn parse_args() -> (String, String, u8, u32, u32) {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();


    let input_file;
    match matches.value_of("input-file") {
        Some(s) => input_file = s.to_owned(),
        None => panic!("Input file argument is empty or missing.")
    }

    let output_directory;
    match matches.value_of("output-directory") {
        Some(s) => output_directory = s.to_owned(),
        None => panic!("Output directory argument is empty or missing.")
    }

    let color_threshold: u8;
    match matches.value_of("color-threshold") {
        Some(s) => color_threshold = s.parse().unwrap(),
        None => color_threshold = 2
    }

    let min_height: u32;
    match matches.value_of("min-height") {
        Some(s) => min_height = s.parse().unwrap(),
        None => min_height = 600
    }

    let max_height: u32;
    match matches.value_of("max-height") {
        Some(s) => max_height = s.parse().unwrap(),
        None => max_height = 7000
    }

    (input_file, output_directory, color_threshold, min_height, max_height)
}

fn check_row_solid(img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>, x: u32, width: u32, color_threshold: u8) -> bool {
    let first_px = img.get_pixel(0, x).0;
    let mut current_pos = 1;
    while current_pos < width {
        let current_px = img.get_pixel(current_pos, x).0;
        let current_r: i16 = current_px[0].into();
        let current_g: i16 = current_px[1].into();
        let current_b: i16 = current_px[2].into();

        let first_r: i16 = first_px[0].into();
        let first_g: i16 = first_px[1].into();
        let first_b: i16 = first_px[2].into();

        if current_r > first_r + color_threshold as i16 || current_r < first_r - color_threshold as i16 {
            return false
        }
        if current_g > first_g + color_threshold as i16 || current_g < first_g - color_threshold as i16 {
            return false
        }
        if current_b > first_b + color_threshold as i16 || current_b < first_b - color_threshold as i16 {
            return false
        }
        current_pos += 1;
    }
    return true
}

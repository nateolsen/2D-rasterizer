use std::fmt::Write;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::env;
use std::str;
use image::{RgbaImage, Rgba, ImageBuffer};
// use sts::io:Rgba
use std::mem;

#[derive(Debug)]
struct Image {
    img_buff: ImageBuffer<Rgba<u8>, Vec<u8>>,
    name: String
}

impl Image {
    fn new(x: u32, y: u32, filename: String) -> Image{
        return Image { img_buff: RgbaImage::from_pixel(x, y, Rgba([0, 0, 0, 0])), name: filename};
    }

    fn save(self, filepath: &String) {
        self.img_buff.save(filepath.to_owned() + &self.name).unwrap()
    }
}

#[derive(Debug, Copy, Clone)]
struct Pixel {
    pos: [f64; 2],
    rgba: Rgba<u8>,
}

impl Pixel {
    fn new(x: f64, y: f64, red: u8, green: u8, blue: u8, alpha: u8) -> Pixel {
        Pixel {pos: [x, y], rgba: Rgba([red, green, blue, alpha]) }
    }
}

#[derive(Debug)]
struct Vector {
    v: [f64; 2]
}

impl Vector {
    fn new(x: f64, y: f64) -> Vector {
        Vector {v: [x, y]}
    }
    
    fn clone(&self) -> Self {
        Vector { v: self.v.clone() }
    }

    fn init_from_pixels(p1: &Pixel, p2: &Pixel) -> Vector {
        Vector {v: [(p2.pos[0] - p1.pos[0]), (p2.pos[1] - p1.pos[1])]}
    }

    fn magnitude(&self) -> f64 {
        return f64::sqrt((self.v[0].powi(2) + self.v[1].powi(2)).into());
    }

    fn swap(&mut self) {
        // let (i1, i2) = (self.v[0], self.v[1]);
        // self.v[0] = i2;
        // self.v[1] = i1;
        self.v.swap(0, 1);
    }

    fn reverse(&mut self) {
        self.v[0] = -1.0*self.v[0];
        self.v[1] = -1.0*self.v[1];
    }

    fn scale(self, factor: f64) -> Vector {
        Vector {v: [(self.v[0]*factor), (self.v[1]*factor)]}
    }

    fn add(&mut self, v2: &Vector) {
        self.v[0] = self.v[0]+ v2.v[0]; 
        self.v[1] = self.v[1] + v2.v[1];
    }
}

fn add_vector_to_pixel(p: &Pixel, v: &Vector) -> Vector {
    Vector {v: [p.pos[0] + v.v[0], p.pos[1] + v.v[1]]}
}

fn convert_hex_color_string(hex_string: String) -> (u8, u8, u8, u8) {
    let no_prefix = hex_string.strip_prefix("#").unwrap().as_bytes();
    let red = u8::from_str_radix(str::from_utf8(&no_prefix[0..2]).unwrap(), 16).unwrap();
    let green = u8::from_str_radix(str::from_utf8(&no_prefix[2..4]).unwrap(), 16).unwrap();
    let blue = u8::from_str_radix(str::from_utf8(&no_prefix[4..6]).unwrap(), 16).unwrap();
    return (red, green, blue, 255);
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn fill_pixel(pic: &mut Image, pixel: Pixel) {
    pic.img_buff.put_pixel(pixel.pos[0] as u32, pixel.pos[1] as u32, pixel.rgba)
}

fn basic_dda(pic: &mut Image, i1: &Pixel, i2: &Pixel, rgba: Option<(u8, u8, u8, u8)>) {
    let mut p1 = i1;
    let mut p2 = i2;
    let mut line = Vector::init_from_pixels(p1, p2);
    // println!("p1: {:?}", p1.pos);
    // println!("p2: {:?}", p2.pos);
    // println!("line: {:?}", line);
    let i: usize;
    let j: usize;

    if line.v[0].abs() > line.v[1].abs() {
        // Step in x/columns
        i = 0;
        j = 1;
    }
    else {
        // Step in y/rows
        i = 1;
        j = 0;
    }

    if line.v[i] < 0.0 {
        mem::swap(&mut p1, &mut p2);
        line.reverse();
        // println!("swapped");
    }

    // println!("p1: {:?}", p1.pos);
    // println!("p2: {:?}", p2.pos);
    // println!("line: {:?}", line);
    let dp_scale: f64; 
    if line.v[i] == 0.0 {
        dp_scale = line.v[j];
    }
    else {
        dp_scale = line.v[i];
    }
    let dp: Vector = line.clone().scale(1.0/dp_scale);
    let dp0: Vector = dp.clone().scale(p1.pos[i].ceil() - p1.pos[i]);
    let mut q: Vector = add_vector_to_pixel(p1, &dp0);
    // println!("step: {}", i);
    // println!("dp: {:?}", dp);
    // println!("dp0: {:?}", dp0);
    // println!("q: {:?}", q);
    
    let step_size = 1.0/(p2.pos[i] - p1.pos[i]);
    while q.v[i] < p2.pos[i] {

        // Linearly interpolate
        let pixel: Pixel;
        if rgba == None {
            
            let red = (p1.rgba.0[0] as f64 + ((p2.rgba.0[0] - p1.rgba.0[0]) as f64 *step_size)).round() as u8;
            let green = (p1.rgba.0[1] as f64 + ((p2.rgba.0[1] - p1.rgba.0[1]) as f64 *step_size)).round() as u8;
            let blue = (p1.rgba.0[2] as f64 + ((p2.rgba.0[2] - p1.rgba.0[2]) as f64 *step_size)).round() as u8;
            pixel = Pixel::new(q.v[0].round(), q.v[1].round(), red, green, blue, 255);
        }
        else {
            let (red, green, blue, alpha) = rgba.unwrap();
            pixel = Pixel::new(q.v[0].round(), q.v[1].round(), red, green, blue, alpha);
        }
        fill_pixel(pic, pixel);
        q.add(&dp);
    }

    // println!("-------------");
}

fn sort_trig_verts_in_y(v1: &Pixel, v2: &Pixel, v3: &Pixel) -> [Pixel; 3]{
    let mut arr = [*v1, *v2, *v3];
    let (left, right) = arr.split_at_mut(1);

    if left[0].pos[1] > right[0].pos[1] {
        std::mem::swap(&mut left[0], &mut right[0]);
    }
    if left[0].pos[1] > right[1].pos[1] {
        std::mem::swap(&mut left[0], &mut right[1]);
    }
    if right[0].pos[1] > right[1].pos[1] {
        right.swap(0, 1);
    }
    arr
}

fn trig(pic: &mut Image, v1: &Pixel, v2: &Pixel, v3: &Pixel) {
    let sorted_vertices = sort_trig_verts_in_y(v1, v2, v3);
    println!("{:?}, {:?}, {:?}", sorted_vertices[0], sorted_vertices[1], sorted_vertices[2]);

    let p_a = Vector::init_from_pixels(&sorted_vertices[0], &sorted_vertices[1]);
    let dq_a_scale: f64; 
    if p_a.v[1] == 0.0 {
        dq_a_scale = p_a.v[0];
    }
    else {
        dq_a_scale = p_a.v[1];
    }
    let dq_a = p_a.clone().scale(1.0/dq_a_scale);
    let dp0_a: Vector = dq_a.clone().scale(&sorted_vertices[0].pos[1].ceil() - &sorted_vertices[0].pos[1]);
    let mut q_a: Vector = add_vector_to_pixel(&sorted_vertices[0], &dp0_a);

    let p_c = Vector::init_from_pixels(&sorted_vertices[0], &sorted_vertices[2]);
    let dq_c_scale: f64; 
    if p_c.v[1] == 0.0 {
        dq_c_scale = p_c.v[0];
    }
    else {
        dq_c_scale = p_c.v[1];
    }
    let dq_c = p_c.clone().scale(1.0/dq_c_scale);
    let dp0_c: Vector = dq_c.clone().scale(&sorted_vertices[0].pos[1].ceil() - &sorted_vertices[0].pos[1]);
    let mut q_c: Vector = add_vector_to_pixel(&sorted_vertices[0], &dp0_c);

    while q_a.v[1] < sorted_vertices[1].pos[1] {

        let p1 = Pixel::new(q_a.v[0], q_a.v[1], sorted_vertices[0].rgba.0[0], sorted_vertices[0].rgba.0[1], sorted_vertices[0].rgba.0[2], 255);
        let p2 = Pixel::new(q_c.v[0], q_c.v[1], sorted_vertices[1].rgba.0[0], sorted_vertices[1].rgba.0[1], sorted_vertices[1].rgba.0[2], 255);
        basic_dda(pic, &p1, &p2, None);
        q_a.add(&dq_a);
        q_c.add(&dq_c);
    }
    println!("q_a: {:?}", q_a.v[1]);
    println!("q_c: {:?}", q_c.v[1]);

    let p_e = Vector::init_from_pixels(&sorted_vertices[1], &sorted_vertices[2]);
    let dq_e_scale: f64; 
    if p_e.v[1] == 0.0 {
        dq_e_scale = p_e.v[0];
    }
    else {
        dq_e_scale = p_e.v[1];
    }
    let dq_e = p_e.clone().scale(1.0/dq_e_scale);
    let dp0_e: Vector = dq_e.clone().scale(&sorted_vertices[1].pos[1].ceil() - &sorted_vertices[1].pos[1]);
    let mut q_e: Vector = add_vector_to_pixel(&sorted_vertices[1], &dp0_e);

    while q_e.v[1] < sorted_vertices[2].pos[1] {
        println!("q_e: {:?}", q_e.v[1]);
        println!("top: {:?}", sorted_vertices[2].pos[1]);
        let p1 = Pixel::new(q_e.v[0], q_e.v[1], sorted_vertices[2].rgba.0[0], sorted_vertices[2].rgba.0[1], sorted_vertices[2].rgba.0[2], 255);
        let p2 = Pixel::new(q_c.v[0], q_c.v[1], sorted_vertices[1].rgba.0[0], sorted_vertices[1].rgba.0[1], sorted_vertices[1].rgba.0[2], 255);
        println!("p1: {:?}", p1.pos);
        println!("p2: {:?}", p2.pos);
        basic_dda(pic, &p1, &p2, None);
        q_e.add(&dq_e);
        q_c.add(&dq_c);
    }
}

fn process_input(filename: String) -> Vec<Image>{

    let mut images: Vec<Image> = Vec::new();
    let mut vertices: Vec<Pixel> = Vec::new();
    // Push initial pixel because vertices start at 1
    vertices.push(Pixel::new(0.0, 0.0, 255, 255, 255, 255));
    let mut current_frame: usize = 0;

    if let Ok(lines) = read_lines(filename) {
        // Consumes the iterator, returns an (Optional) String

        for line in lines {
            if let Ok(cmd) = line {
                // println!("{}", cmd);
                let split_string = cmd.split_whitespace();
                let cmd_params = split_string.collect::<Vec<&str>>();
                if cmd_params.len() < 1 {
                    continue;
                }
                let keyword = cmd_params[0];

                // Process each 1st word in line, if keyword then execute function, otherwise skip
                match keyword {
                    "png" => {
                        let pic = Image::new(cmd_params[1].parse::<u32>().unwrap(), cmd_params[2].parse::<u32>().unwrap(), cmd_params[3].to_string());
                        images.push(pic);
                    }
                    "pngs" => {
                        let x: u32 = cmd_params[1].parse::<u32>().unwrap();
                        let y: u32 = cmd_params[2].parse::<u32>().unwrap();
                        let count: u32 = cmd_params[4].parse::<u32>().unwrap();
                        for c in 0..count {
                            let mut suffix = String::new();
                            write!(suffix, "{:03}.png", c).unwrap();
                            let new_filename = cmd_params[3].to_string() + &suffix;
                            let pic = Image::new(x, y, new_filename);
                            images.push(pic);                         
                        }
                    }
                    "xyrgb" => {
                        let x: f64 = cmd_params[1].parse::<f64>().unwrap();
                        let y: f64 = cmd_params[2].parse::<f64>().unwrap();
                        let red: u8 = cmd_params[3].parse::<u8>().unwrap();
                        let green: u8 = cmd_params[4].parse::<u8>().unwrap();
                        let blue: u8 = cmd_params[5].parse::<u8>().unwrap();
                        let pixel = Pixel::new(x, y, red, green, blue, 255);
                        // fill_pixel(images.get_mut(current_frame).unwrap(), pixel);
                        vertices.push(pixel);
                    }
                    "xyc" => {
                        let x: f64 = cmd_params[1].parse::<f64>().unwrap();
                        let y: f64 = cmd_params[2].parse::<f64>().unwrap();
                        let hex_string: String = cmd_params[3].to_string();
                        let rbga = convert_hex_color_string(hex_string);
                        let pixel = Pixel::new(x, y, rbga.0, rbga.1, rbga.2, rbga.3);
                        // fill_pixel(images.get_mut(current_frame).unwrap(), pixel);
                        vertices.push(pixel);
                    }
                    "frame" => {
                        current_frame = cmd_params[1].parse::<usize>().unwrap();
                    }
                    "linec" => {
                        let i1: usize = cmd_params[1].parse::<usize>().unwrap();
                        let i2: usize = cmd_params[2].parse::<usize>().unwrap();
                        let vertex1 = &vertices[i1];
                        let vertex2 = &vertices[i2];
                        let rbga = convert_hex_color_string(cmd_params[3].to_string());
                        basic_dda(images.get_mut(current_frame).unwrap(), vertex1, vertex2, Some(rbga));
                    }
                    "trig" => {
                        let i1: usize = cmd_params[1].parse::<usize>().unwrap();
                        let i2: usize = cmd_params[2].parse::<usize>().unwrap();
                        let i3: usize = cmd_params[3].parse::<usize>().unwrap();
                        let vertex1 = &vertices[i1];
                        let vertex2 = &vertices[i2];
                        let vertex3 = &vertices[i3];
                        // let rbga = convert_hex_color_string(cmd_params[3].to_string());
                        trig(images.get_mut(current_frame).unwrap(), vertex1, vertex2, vertex3);
                    }

                    // Skip
                    _ => println!("{} has been skipped", keyword.to_string()),
                }

            }
        }
    }
    return images;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let images = process_input(filename.to_string());

    // Standard output path
    let output_folder = String::from("./src/output_files/");

    for img in images {
        img.save(&output_folder);
    }    
}
use std::{fs::{self, File}, io::{self, Write}};
use serde::{Serialize, Deserialize};

use image::{DynamicImage, GenericImage, ImageBuffer};

fn main() {
    let atlas = generate_texture_atlas();

    let json_string = serde_json::to_string_pretty(&atlas).expect("Failed to serialize to JSON.");

    let mut file = File::create("../atlas.json").expect("Failed to create file.");
    file.write_all(json_string.as_bytes()).expect("Failed to write to file.");
}

pub fn generate_texture_atlas() -> UiAtlas {
    let mut images: Vec<(DynamicImage, String)> = Vec::new();
    let assets_dir = fs::read_dir(r"./assets").unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>().unwrap();
    for asset in assets_dir {
        images.push((image::open(asset.as_path()).unwrap(), asset.file_stem().unwrap().to_str().unwrap().to_string()));
    }

    let mut new_width = 0;
    let mut new_height = 0;

    let mut last_image: Option<DynamicImage> = None;
    for image in &images {
        if last_image.is_none() {
            new_height = image.0.height();
        } else {
            new_height = image.0.height().max(last_image.unwrap().height().max(new_height));
        }
        new_width += image.0.width();
        last_image = Some(image.0.clone());
    }

    let mut atlas = ImageBuffer::new(new_width, new_height);
    let mut atlas_data = UiAtlas::new(new_width, new_height);

    let mut last_coordinate = 0;
    for image in images {
        atlas_data.add_entry(UiAtlasTexture::new(image.1, last_coordinate, 0, image.0.width(), image.0.height()));
        atlas.copy_from(&image.0, last_coordinate, 0).unwrap();
        last_coordinate += &image.0.width();
    }

    atlas.save("./../atlas.png").unwrap();
    atlas_data
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiAtlas {
    entries: Vec<UiAtlasTexture>,
    width: u32,
    height: u32,
}

impl UiAtlas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            entries: Vec::new(),
            width,
            height
        }
    }

    fn add_entry(&mut self, entry: UiAtlasTexture) {
        self.entries.push(entry.generate_tex_coords(self.width, self.height));
    }

    fn get_entry_by_name(&self, name: String) -> Option<UiAtlasTexture> {
        self.entries.iter().find(|entry| entry.name == name).cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UiAtlasTexture {
    pub name: String,
    x_start: u32,
    y_start: u32,
    image_width: u32,
    image_height: u32,
    pub start_coord: Option<(f32, f32)>,
    pub end_coord: Option<(f32, f32)>
}

impl UiAtlasTexture {
    pub fn new(name: String, x_0: u32, y_0: u32, image_width: u32, image_height: u32) -> Self {
        Self {
            name,
            x_start: x_0,
            y_start: y_0,
            image_width,
            image_height,
            start_coord: None,
            end_coord: None,
        }
    }

    fn generate_tex_coords(mut self, width: u32, height: u32) -> Self {
        // Calculate a half-pixel offset based on the atlas dimensions
        let half_pixel_x = 0.5 / width as f32;
        let half_pixel_y = 0.5 / height as f32;

        let x0 = self.x_start as f32 / width as f32;
        let y0 = self.y_start as f32 / height as f32;
        let x1 = (self.x_start + self.image_width) as f32 / width as f32;
        let y1 = (self.y_start + self.image_height) as f32 / height as f32;

        // Inset the coordinates by a half-pixel to avoid the edges
        self.start_coord = Some((x0 + half_pixel_x, y0 + half_pixel_y));
        self.end_coord = Some((x1 - half_pixel_x, y1 - half_pixel_y));
        
        self
    }
}
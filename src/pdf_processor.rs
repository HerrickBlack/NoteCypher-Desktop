use image::{DynamicImage, ImageBuffer, Rgba, Rgb};
use lopdf::{Document, Object, Dictionary, Stream};
use std::collections::HashMap;
use std::sync::Arc;
use crate::Orientation;

#[derive(Clone, Debug)]
pub struct PdfDocument {
    pub name: String,
    pub size: u64,
    #[allow(dead_code)]
    pub path: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug)]
pub struct PdfPage {
    pub id: String,
    pub page_number: u32,
    pub pdf_index: usize,
    #[allow(dead_code)]
    pub file_name: String,
    #[allow(dead_code)]
    pub page_number_in_pdf: u32,
    #[allow(dead_code)]
    pub page_index: usize,
    #[allow(dead_code)]
    pub thumbnail_data: Arc<Vec<u8>>,
    #[allow(dead_code)]
    pub image_data: Arc<Vec<u8>>,
    #[allow(dead_code)]
    pub width: u32,
    #[allow(dead_code)]
    pub height: u32,
}

#[derive(Clone, Copy, Default)]
pub struct FilterOptions {
    pub invert: bool,
    pub clear_background: bool,
    pub grayscale: bool,
}

pub async fn process_pdf(
    paths: Vec<std::path::PathBuf>,
    existing_page_count: usize,
) -> Result<(Vec<PdfDocument>, Vec<PdfPage>, HashMap<usize, Arc<Vec<u8>>>), String> {
    let mut documents = Vec::new();
    let mut pages = Vec::new();
    let mut images = HashMap::new();
    
    for path in paths {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.pdf")
            .to_string();
        
        let file_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let _pdf_data = std::fs::read(&path)
            .map_err(|e| format!("Failed to read PDF {}: {}", path.display(), e))?;
        
        let page_count = if let Ok(doc) = Document::load(&path) {
            doc.get_pages().len()
        } else {
            1
        };
        
        let pdf_index = documents.len() + 1;
        
        documents.push(PdfDocument {
            name: file_name.clone(),
            size: file_size,
            path: Some(path.clone()),
        });
        
        for page_num in 1..=page_count {
            let page_index = existing_page_count + pages.len();
            
            let thumbnail = create_placeholder_thumbnail(150, 200);
            let thumbnail_data = encode_png(&thumbnail)
                .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;
            
            let high_res = create_white_image(1200, 1600);
            let image_data = encode_png(&high_res)
                .map_err(|e| format!("Failed to encode image: {}", e))?;
            
            let image_data_arc = Arc::new(image_data.clone());
            
            let page = PdfPage {
                id: format!("page-{}-{}-{}", chrono::Local::now().timestamp_nanos_opt().unwrap_or(0), page_index, page_num),
                page_number: page_num as u32,
                pdf_index,
                file_name: file_name.clone(),
                page_number_in_pdf: page_num as u32,
                page_index,
                thumbnail_data: Arc::new(thumbnail_data),
                image_data: image_data_arc.clone(),
                width: 1200,
                height: 1600,
            };
            
            images.insert(page_index, image_data_arc);
            pages.push(page);
        }
    }
    
    Ok((documents, pages, images))
}

pub async fn export_pdf(
    selected: Vec<usize>,
    pages: Vec<PdfPage>,
    images: HashMap<usize, Arc<Vec<u8>>>,
    filters: FilterOptions,
    layout: u32,
    orientation: Orientation,
    margin_cm: f32,
) -> Result<Vec<u8>, String> {
    if selected.is_empty() {
        return Err("No pages selected".to_string());
    }
    
    let mut sorted_indices = selected;
    sorted_indices.sort();
    
    let (cols, rows) = match layout {
        1 => (1, 1),
        2 => (1, 2),
        3 => (1, 3),
        4 => (2, 2),
        6 => (2, 3),
        _ => (1, 1),
    };
    
    let (page_width, page_height) = match orientation {
        Orientation::Portrait => (595.0, 842.0),
        Orientation::Landscape => (842.0, 595.0),
    };
    
    let margin_pt = margin_cm * 28.35;
    let available_width = page_width - (2.0 * margin_pt);
    let available_height = page_height - (2.0 * margin_pt);
    
    let slide_width = available_width / cols as f32;
    let slide_height = available_height / rows as f32;
    
    let mut doc = Document::with_version("1.7");
    let mut page_ids = Vec::new();
    
    let mut output_page_num = 0;
    let mut slide_idx = 0;
    
    while slide_idx < sorted_indices.len() {
        let slides_in_this_page = std::cmp::min(layout as usize, sorted_indices.len() - slide_idx);
        
        let page_id = ((output_page_num + 1) * 10) as u32;
        page_ids.push(page_id);
        
        let mut content_operations = Vec::new();
        let mut xobjects = Dictionary::new();
        
        for local_idx in 0..slides_in_this_page {
            let global_slide_idx = slide_idx + local_idx;
            let page_idx = sorted_indices[global_slide_idx];
            
            let _page = pages.get(page_idx).ok_or(format!("Page {} not found", page_idx))?;
            let image_data = images.get(&page_idx)
                .ok_or(format!("Image data not found for page {}", page_idx))?;
            
            let img = image::load_from_memory(&image_data)
                .map_err(|e| format!("Failed to decode image: {}", e))?;
            
            let filtered_img = apply_filters(&img, filters);
            let rgb_img = filtered_img.to_rgb8();
            let raw_image_data = rgb_img.as_raw().clone();
            
            let col = local_idx % cols as usize;
            let row = local_idx / cols as usize;
            
            let img_width = rgb_img.width() as f32;
            let img_height = rgb_img.height() as f32;
            let img_aspect = img_width / img_height;
            let slide_aspect = slide_width / slide_height;
            
            let (draw_width, draw_height) = if img_aspect > slide_aspect {
                (slide_width * 0.95, (slide_width * 0.95) / img_aspect)
            } else {
                (slide_height * 0.95 * img_aspect, slide_height * 0.95)
            };
            
            let x = (col as f32 * slide_width) + (slide_width - draw_width) / 2.0 + margin_pt;
            let y = page_height - margin_pt - ((row as f32 + 1.0) * slide_height) + (slide_height - draw_height) / 2.0;
            
            let image_obj_id = ((output_page_num * 100 + local_idx as u32) + 1000) as u32;
            
            let mut image_dict = Dictionary::new();
            image_dict.set("Type", "XObject");
            image_dict.set("Subtype", "Image");
            image_dict.set("Width", rgb_img.width() as i64);
            image_dict.set("Height", rgb_img.height() as i64);
            image_dict.set("ColorSpace", "DeviceRGB");
            image_dict.set("BitsPerComponent", 8);
            image_dict.set("Filter", "FlateDecode");
            
            use miniz_oxide::deflate::compress_to_vec_zlib;
            let compressed = compress_to_vec_zlib(&raw_image_data, 6);
            
            doc.objects.insert(
                (image_obj_id, 0),
                Object::Stream(Stream {
                    dict: image_dict,
                    content: compressed,
                    allows_compression: false,
                    start_position: Some(0),
                }),
            );
            
            xobjects.set(format!("Img{}", image_obj_id).as_bytes().to_vec(), 
                Object::Reference((image_obj_id, 0)));
            
            content_operations.push(Object::Array(vec![
                Object::Name(b"q".to_vec()),
            ]));
            
            content_operations.push(Object::Array(vec![
                Object::Real(draw_width),
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(draw_height),
                Object::Real(x),
                Object::Real(y),
            ]));
            content_operations.push(Object::Array(vec![
                Object::Name(b"cm".to_vec()),
            ]));
            
            content_operations.push(Object::Array(vec![
                Object::Name(format!("/Img{}", image_obj_id).as_bytes().to_vec()),
                Object::Name(b"Do".to_vec()),
            ]));
            
            content_operations.push(Object::Array(vec![
                Object::Name(b"Q".to_vec()),
            ]));
        }
        
        let mut content_bytes = Vec::new();
        for op in content_operations {
            match op {
                Object::Array(ref ops) => {
                    for o in ops {
                        match o {
                            Object::Name(n) => {
                                content_bytes.extend_from_slice(n);
                                content_bytes.push(b' ');
                            }
                            Object::Real(r) => {
                                content_bytes.extend_from_slice(format!("{} ", r).as_bytes());
                            }
                            _ => {}
                        }
                    }
                    content_bytes.push(b'\n');
                }
                _ => {}
            }
        }
        
        let content_obj_id = page_id + 1;
        doc.objects.insert(
            (content_obj_id, 0),
            Object::Stream(Stream {
                dict: Dictionary::new(),
                content: content_bytes,
                allows_compression: false,
                start_position: Some(0),
            }),
        );
        
        let mut page_dict = Dictionary::new();
        page_dict.set("Type", "Page");
        
        let mut resources = Dictionary::new();
        resources.set("XObject", xobjects);
        page_dict.set("Resources", resources);
        
        page_dict.set("Contents", Object::Reference((content_obj_id, 0)));
        page_dict.set("MediaBox", Object::Array(vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(page_width),
            Object::Real(page_height),
        ]));
        
        doc.objects.insert((page_id, 0), Object::Dictionary(page_dict));
        
        output_page_num += 1;
        slide_idx += layout as usize;
    }
    
    let pages_dict_id = 2;
    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", "Pages");
    pages_dict.set("Kids", Object::Array(
        page_ids.iter().map(|&id| Object::Reference((id, 0))).collect()
    ));
    pages_dict.set("Count", page_ids.len() as i64);
    doc.objects.insert((pages_dict_id, 0), Object::Dictionary(pages_dict));
    
    let catalog_id = 1;
    let mut catalog = Dictionary::new();
    catalog.set("Type", "Catalog");
    catalog.set("Pages", Object::Reference((pages_dict_id, 0)));
    doc.objects.insert((catalog_id, 0), Object::Dictionary(catalog));
    
    doc.trailer.set("Root", Object::Reference((catalog_id, 0)));
    
    let mut buffer = Vec::new();
    doc.save_to(&mut buffer)
        .map_err(|e| format!("Failed to save PDF: {}", e))?;
    
    Ok(buffer)
}

fn apply_filters(img: &DynamicImage, filters: FilterOptions) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    
    for pixel in rgba.pixels_mut() {
        let mut r = pixel[0] as f32;
        let mut g = pixel[1] as f32;
        let mut b = pixel[2] as f32;
        
        if filters.grayscale {
            let gray = 0.299 * r + 0.587 * g + 0.114 * b;
            r = gray;
            g = gray;
            b = gray;
        }
        
        if filters.invert {
            r = 255.0 - r;
            g = 255.0 - g;
            b = 255.0 - b;
        }
        
        if filters.clear_background {
            let avg = (r + g + b) / 3.0;
            if avg > 220.0 {
                r = 255.0;
                g = 255.0;
                b = 255.0;
            }
        }
        
        pixel[0] = r as u8;
        pixel[1] = g as u8;
        pixel[2] = b as u8;
    }
    
    DynamicImage::ImageRgba8(rgba)
}

fn create_placeholder_thumbnail(width: u32, height: u32) -> DynamicImage {
    let mut img = ImageBuffer::new(width, height);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let gradient = ((x + y) as f32 / (width + height) as f32) * 100.0;
        *pixel = Rgba([
            (gradient * 0.6) as u8,
            (gradient * 0.7) as u8,
            (gradient + 100.0) as u8,
            255,
        ]);
    }
    
    DynamicImage::ImageRgba8(img)
}

fn create_white_image(width: u32, height: u32) -> DynamicImage {
    let img = ImageBuffer::from_pixel(width, height, Rgb([255, 255, 255]));
    DynamicImage::ImageRgb8(img)
}

fn encode_png(img: &DynamicImage) -> Result<Vec<u8>, image::ImageError> {
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    img.write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(buffer)
}

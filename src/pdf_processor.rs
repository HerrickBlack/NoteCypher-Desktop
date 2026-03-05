use image::{DynamicImage, ImageBuffer, Rgba, Rgb};
use lopdf::{Document, Object, Dictionary};
use std::collections::HashMap;
use std::sync::Arc;
use crate::Orientation;

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Clone)]
pub struct PdfDocument {
    pub name: String,
    pub size: u64,
    pub path: Option<std::path::PathBuf>,
}

#[derive(Clone)]
pub struct PdfPage {
    pub id: String,
    pub page_number: u32,
    pub pdf_index: usize,
    pub file_name: String,
    pub page_number_in_pdf: u32,
    pub page_index: usize,
    pub thumbnail_data: Arc<Vec<u8>>, // PNG encoded thumbnail
    pub image_data: Arc<Vec<u8>>,     // PNG encoded high-res image
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Default)]
pub struct FilterOptions {
    pub invert: bool,
    pub clear_background: bool,
    pub grayscale: bool,
}

// ============================================================================
// PDF Processing
// ============================================================================

pub async fn process_pdf(
    paths: Vec<std::path::PathBuf>,
    existing_page_count: usize,
) -> Result<(Vec<PdfDocument>, Vec<PdfPage>, HashMap<usize, Arc<Vec<u8>>>), String> {
    let mut documents = Vec::new();
    let mut pages = Vec::new();
    let mut images = HashMap::new();
    
    let total_files = paths.len();
    
    for path in paths {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.pdf")
            .to_string();
        
        let file_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        // Read PDF file to verify it exists
        let _pdf_data = std::fs::read(&path)
            .map_err(|e| format!("Failed to read PDF {}: {}", path.display(), e))?;
        
        // Try to get page count using lopdf
        let page_count = if let Ok(doc) = Document::load(&path) {
            doc.get_pages().map(|p| p.len()).unwrap_or(1)
        } else {
            1 // Fallback to 1 page if we can't parse
        };
        
        let pdf_index = documents.len() + 1;
        
        documents.push(PdfDocument {
            name: file_name.clone(),
            size: file_size,
            path: Some(path.clone()),
        });
        
        // Generate pages with placeholder images
        for page_num in 1..=page_count {
            let page_index = existing_page_count + pages.len();
            
            // Create a placeholder thumbnail
            let thumbnail = create_placeholder_thumbnail(150, 200, page_num);
            let thumbnail_data = encode_png(&thumbnail)
                .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;
            
            // Create a placeholder high-res image (white page)
            let high_res = create_white_image(1200, 1600);
            let image_data = encode_png(&high_res)
                .map_err(|e| format!("Failed to encode image: {}", e))?;
            
            let page = PdfPage {
                id: format!("page-{}-{}-{}", chrono::Local::now().timestamp_nanos_opt().unwrap_or(0), page_index, page_num),
                page_number: page_num,
                pdf_index,
                file_name: file_name.clone(),
                page_number_in_pdf: page_num,
                page_index,
                thumbnail_data: Arc::new(thumbnail_data),
                image_data: Arc::new(image_data),
                width: 1200,
                height: 1600,
            };
            
            images.insert(page_index, Arc::new(image_data));
            pages.push(page);
        }
    }
    
    Ok((documents, pages, images))
}

// ============================================================================
// PDF Export
// ============================================================================

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
    
    // Determine grid dimensions
    let (cols, rows) = match layout {
        1 => (1, 1),
        2 => (1, 2),
        3 => (1, 3),
        4 => (2, 2),
        6 => (2, 3),
        _ => (1, 1),
    };
    
    // Page dimensions in points (A4)
    let (page_width, page_height) = match orientation {
        Orientation::Portrait => (595.0, 842.0),
        Orientation::Landscape => (842.0, 595.0),
    };
    
    // Convert margin from cm to points (1 cm = 28.35 points)
    let margin_pt = margin_cm * 28.35;
    let available_width = page_width - (2.0 * margin_pt);
    let available_height = page_height - (2.0 * margin_pt);
    
    let slide_width = available_width / cols as f32;
    let slide_height = available_height / rows as f32;
    
    // Create output PDF document
    let mut doc = Document::with_version("1.7");
    let mut page_ids = Vec::new();
    let mut image_objects = Vec::new();
    
    // Process pages in groups based on layout
    let mut output_page_num = 0;
    let mut slide_idx = 0;
    
    while slide_idx < sorted_indices.len() {
        let slides_in_this_page = std::cmp::min(layout as usize, sorted_indices.len() - slide_idx);
        
        // Create a new page
        let page_id = ((output_page_num + 1) * 10) as u32;
        page_ids.push(page_id);
        
        let mut content_operations = Vec::new();
        let mut xobjects = Dictionary::new();
        
        // Add each slide to this page
        for local_idx in 0..slides_in_this_page {
            let global_slide_idx = slide_idx + local_idx;
            let page_idx = sorted_indices[global_slide_idx];
            
            let page = pages.get(page_idx).ok_or(format!("Page {} not found", page_idx))?;
            let image_data = images.get(&page_idx)
                .ok_or(format!("Image data not found for page {}", page_idx))?;
            
            // Decode and process image
            let img = image::load_from_memory(&image_data)
                .map_err(|e| format!("Failed to decode image: {}", e))?;
            
            let filtered_img = apply_filters(&img, filters);
            
            // Convert to RGB for PDF
            let rgb_img = filtered_img.to_rgb8();
            let raw_image_data = rgb_img.as_raw().clone();
            
            // Calculate position in grid
            let col = local_idx % cols as usize;
            let row = local_idx / cols as usize;
            
            // Scale image to fit
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
            
            // Create image dictionary
            let image_obj_id = ((output_page_num * 100 + local_idx as u32) + 1000) as u32;
            image_objects.push(image_obj_id);
            
            let mut image_dict = Dictionary::new();
            image_dict.set("Type", "XObject");
            image_dict.set("Subtype", "Image");
            image_dict.set("Width", rgb_img.width() as i64);
            image_dict.set("Height", rgb_img.height() as i64);
            image_dict.set("ColorSpace", "DeviceRGB");
            image_dict.set("BitsPerComponent", 8);
            image_dict.set("Filter", "FlateDecode");
            
            // Compress image data
            use miniz_oxide::deflate::compress_to_vec_zlib;
            let compressed = compress_to_vec_zlib(&raw_image_data, 6);
            
            doc.objects.insert(
                (image_obj_id, 0),
                Object::Stream(lopdf::Stream {
                    dict: image_dict,
                    content: compressed,
                }),
            );
            
            // Add to XObjects
            xobjects.set(format!("Img{}", image_obj_id).into_bytes(), 
                Object::Reference((image_obj_id, 0)));
            
            // Add drawing operations
            content_operations.push(Object::Array(vec![
                Object::Name("q".into_bytes()),
            ]));
            
            // Transform matrix: scale and translate
            content_operations.push(Object::Array(vec![
                Object::Real(draw_width),
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(draw_height),
                Object::Real(x),
                Object::Real(y),
            ]));
            content_operations.push(Object::Array(vec![
                Object::Name("cm".into_bytes()),
            ]));
            
            // Paint image
            content_operations.push(Object::Array(vec![
                Object::Name(format!("/Img{}", image_obj_id).into_bytes()),
                Object::Name("Do".into_bytes()),
            ]));
            
            content_operations.push(Object::Array(vec![
                Object::Name("Q".into_bytes()),
            ]));
        }
        
        // Build content stream
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
        
        // Create content stream object
        let content_obj_id = page_id + 1;
        doc.objects.insert(
            (content_obj_id, 0),
            Object::Stream(lopdf::Stream {
                dict: Dictionary::new(),
                content: content_bytes,
            }),
        );
        
        // Create page dictionary
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
    
    // Create Pages dictionary
    let pages_dict_id = 2;
    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", "Pages");
    pages_dict.set("Kids", Object::Array(
        page_ids.iter().map(|&id| Object::Reference((id, 0))).collect()
    ));
    pages_dict.set("Count", page_ids.len() as i64);
    doc.objects.insert((pages_dict_id, 0), Object::Dictionary(pages_dict));
    
    // Create Catalog
    let catalog_id = 1;
    let mut catalog = Dictionary::new();
    catalog.set("Type", "Catalog");
    catalog.set("Pages", Object::Reference((pages_dict_id, 0)));
    doc.objects.insert((catalog_id, 0), Object::Dictionary(catalog));
    
    // Set trailer
    doc.trailer.set("Root", Object::Reference((catalog_id, 0)));
    
    // Save to buffer
    let mut buffer = Vec::new();
    doc.save_to(&mut buffer)
        .map_err(|e| format!("Failed to save PDF: {}", e))?;
    
    Ok(buffer)
}

// ============================================================================
// Image Processing
// ============================================================================

fn apply_filters(img: &DynamicImage, filters: FilterOptions) -> DynamicImage {
    // Convert to RGBA for processing
    let mut rgba = img.to_rgba8();
    
    for pixel in rgba.pixels_mut() {
        let mut r = pixel[0] as f32;
        let mut g = pixel[1] as f32;
        let mut b = pixel[2] as f32;
        
        // Grayscale filter
        if filters.grayscale {
            let gray = 0.299 * r + 0.587 * g + 0.114 * b;
            r = gray;
            g = gray;
            b = gray;
        }
        
        // Invert filter
        if filters.invert {
            r = 255.0 - r;
            g = 255.0 - g;
            b = 255.0 - b;
        }
        
        // Clear background filter (threshold)
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

fn create_placeholder_thumbnail(width: u32, height: u32, page_num: u32) -> DynamicImage {
    let mut img = ImageBuffer::new(width, height);
    
    // Create a gradient background
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

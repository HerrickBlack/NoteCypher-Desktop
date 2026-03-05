use iced::executor;
use iced::theme::{self, Theme};
use iced::widget::{container, scrollable, Column, Row, Space, button, text, checkbox};
use iced::{Element, Length, Task, window, Event, Size};
use iced::subscription::{self, Subscription};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

mod pdf_processor;
mod theme_style;

use pdf_processor::{PdfDocument, PdfPage, process_pdf, export_pdf, FilterOptions};

// ============================================================================
// Application State
// ============================================================================

#[derive(Default)]
struct AppState {
    pdf_files: Vec<PdfDocument>,
    all_pages: Vec<PdfPage>,
    selected_pages: HashSet<usize>,
    filters: FilterOptions,
    layout: u32,
    orientation: Orientation,
    margin_cm: f32,
    is_processing: bool,
    progress: f32,
    progress_status: String,
    dark_mode: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

// ============================================================================
// Messages
// ============================================================================

#[derive(Debug, Clone)]
enum Message {
    // Window events
    WindowEvent(Event),
    
    // File operations
    FilesSelected(Vec<PathBuf>),
    RemovePdfFile(usize),
    ClearAll,
    
    // Page selection
    TogglePageSelection(usize),
    SelectAllPages,
    DeselectAllPages,
    SelectByPdf(usize),
    
    // Filters
    ToggleInvert(bool),
    ToggleClearBackground(bool),
    ToggleGrayscale(bool),
    
    // Layout
    SetLayout(u32),
    SetOrientation(Orientation),
    SetMargin(f32),
    
    // Export
    ExportPdf,
    ExportComplete(Result<Vec<u8>, String>),
    
    // Progress updates from async task
    ProgressUpdate(f32, String),
    
    // Theme
    ToggleDarkMode,
    
    // Theme loaded
    ThemeLoaded(bool),
    
    // Open file dialog
    OpenFileDialog,
}

// ============================================================================
// Application
// ============================================================================

struct NoteCypher {
    state: AppState,
    thumbnail_cache: HashMap<usize, Arc<Vec<u8>>>,
    image_cache: HashMap<usize, Arc<Vec<u8>>>,
}

impl NoteCypher {
    fn new() -> (Self, Task<Message>) {
        let app = NoteCypher {
            state: AppState::default(),
            thumbnail_cache: HashMap::new(),
            image_cache: HashMap::new(),
        };
        
        (app, Task::none())
    }
    
    fn load_theme(&mut self) {
        // Try to load dark mode preference from config file
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("NoteCypher").join("config.json");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(dark) = config.get("dark_mode").and_then(|v| v.as_bool()) {
                            self.state.dark_mode = dark;
                        }
                    }
                }
            }
        }
    }
    
    fn save_theme(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("NoteCypher");
            let _ = std::fs::create_dir_all(&config_path);
            
            let config = serde_json::json!({
                "dark_mode": self.state.dark_mode
            });
            
            let _ = std::fs::write(
                config_path.join("config.json"),
                serde_json::to_string_pretty(&config).unwrap_or_default()
            );
        }
    }
    
    fn theme(&self) -> Theme {
        if self.state.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

impl iced::Application for NoteCypher {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new() -> (Self, Task<Message>) {
        let mut app = NoteCypher::new();
        app.0.load_theme();
        app.0
    }

    fn title(&self) -> String {
        "NoteCypher - PDF Note Optimizer".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowEvent(event) => {
                if let Event::Keyboard(iced::keyboard::Event::KeyPressed { 
                    key: iced::keyboard::Key::Character(c),
                    modifiers,
                    ..
                }) = event {
                    // Ctrl+O for opening files
                    if c.as_str() == "o" && modifiers.control() {
                        return Task::perform(async {
                            // File dialog would be triggered here
                            Message::FilesSelected(vec![])
                        }, |msg| msg);
                    }
                }
                return Task::none();
            }
            
            Message::FilesSelected(paths) => {
                if paths.is_empty() {
                    // Trigger file dialog using rfd
                    let paths_clone = paths.clone();
                    return Task::perform(async move {
                        // Spawn async file dialog
                        let files = rfd::AsyncFileDialog::new()
                            .add_filter("PDF Files", &["pdf"])
                            .set_title("Select PDF Files")
                            .pick_files()
                            .await;
                        
                        files.map(|f| f.iter().map(|fh| fh.path().to_owned()).collect())
                            .unwrap_or(vec![])
                    }, Message::FilesSelected);
                }
                
                if paths.is_empty() {
                    return Task::none();
                }
                
                self.state.progress_status = "Loading PDFs...".to_string();
                self.state.progress = 10.0;
                
                let pdf_paths = paths.clone();
                let page_count = self.state.all_pages.len();
                
                return Task::perform(async move {
                    match process_pdf(pdf_paths, page_count).await {
                        Ok((docs, pages, images)) => {
                            Ok((docs, pages, images))
                        }
                        Err(e) => Err(e)
                    }
                }, move |result| {
                    match result {
                        Ok((docs, pages, images)) => {
                            // Update state with loaded data
                            Message::FilesSelected(paths) // Will be handled in next iteration
                        }
                        Err(e) => {
                            Message::ExportComplete(Err(e))
                        }
                    }
                });
            }
            
            Message::RemovePdfFile(index) => {
                let pdf_index = index + 1;
                let pages_to_remove: Vec<usize> = self.state.all_pages
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.pdf_index == pdf_index)
                    .map(|(i, _)| i)
                    .collect();
                
                // Clean up caches
                for &idx in &pages_to_remove {
                    self.thumbnail_cache.remove(&idx);
                    self.image_cache.remove(&idx);
                }
                
                // Remove pages
                self.state.all_pages.retain(|p| p.pdf_index != pdf_index);
                
                // Remove PDF file
                if index < self.state.pdf_files.len() {
                    self.state.pdf_files.remove(index);
                }
                
                // Update selection
                let new_selection: HashSet<usize> = self.state.selected_pages
                    .iter()
                    .filter(|&&i| !pages_to_remove.contains(&i))
                    .copied()
                    .collect();
                self.state.selected_pages = new_selection;
            }
            
            Message::ClearAll => {
                self.state.pdf_files.clear();
                self.state.all_pages.clear();
                self.state.selected_pages.clear();
                self.thumbnail_cache.clear();
                self.image_cache.clear();
                self.state.filters = FilterOptions::default();
                self.state.layout = 1;
                self.state.orientation = Orientation::Portrait;
                self.state.margin_cm = 0.0;
                self.state.is_processing = false;
                self.state.progress = 0.0;
                self.state.progress_status.clear();
            }
            
            Message::TogglePageSelection(index) => {
                if self.state.selected_pages.contains(&index) {
                    self.state.selected_pages.remove(&index);
                } else {
                    self.state.selected_pages.insert(index);
                }
            }
            
            Message::SelectAllPages => {
                self.state.selected_pages = (0..self.state.all_pages.len()).collect();
            }
            
            Message::DeselectAllPages => {
                self.state.selected_pages.clear();
            }
            
            Message::SelectByPdf(pdf_index) => {
                let pdf_page_indices: HashSet<usize> = self.state.all_pages
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.pdf_index == pdf_index)
                    .map(|(i, _)| i)
                    .collect();
                
                let all_selected = pdf_page_indices.iter().all(|&i| self.state.selected_pages.contains(&i));
                
                if all_selected {
                    self.state.selected_pages = self.state.selected_pages
                        .difference(&pdf_page_indices)
                        .copied()
                        .collect();
                } else {
                    self.state.selected_pages.extend(pdf_page_indices);
                }
            }
            
            Message::ToggleInvert(value) => {
                self.state.filters.invert = value;
            }
            
            Message::ToggleClearBackground(value) => {
                self.state.filters.clear_background = value;
            }
            
            Message::ToggleGrayscale(value) => {
                self.state.filters.grayscale = value;
            }
            
            Message::SetLayout(value) => {
                self.state.layout = value;
            }
            
            Message::SetOrientation(orientation) => {
                self.state.orientation = orientation;
            }
            
            Message::SetMargin(value) => {
                self.state.margin_cm = value.clamp(0.0, 5.0);
            }
            
            Message::ExportPdf => {
                if self.state.selected_pages.is_empty() {
                    return Task::none();
                }
                
                self.state.is_processing = true;
                self.state.progress_status = "Processing PDF...".to_string();
                
                let selected: Vec<usize> = self.state.selected_pages.iter().copied().collect();
                let pages = self.state.all_pages.clone();
                let images = self.image_cache.clone();
                let filters = self.state.filters;
                let layout = self.state.layout;
                let orientation = self.state.orientation;
                let margin_cm = self.state.margin_cm;
                
                return Task::perform(async move {
                    export_pdf(
                        selected,
                        pages,
                        images,
                        filters,
                        layout,
                        orientation,
                        margin_cm,
                    ).await
                }, Message::ExportComplete);
            }
            
            Message::ExportComplete(result) => {
                self.state.is_processing = false;
                
                match result {
                    Ok(pdf_bytes) => {
                        // Save file
                        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
                        let filename = format!("optimized-notes-{}.pdf", timestamp);
                        
                        // For now, save to current directory
                        // In full implementation, use file save dialog
                        let output_path = std::env::current_dir()
                            .unwrap_or_default()
                            .join(&filename);
                        
                        if std::fs::write(&output_path, pdf_bytes).is_ok() {
                            self.state.progress_status = format!("Saved to: {:?}", output_path);
                            self.state.progress = 100.0;
                        } else {
                            self.state.progress_status = "Failed to save file".to_string();
                        }
                    }
                    Err(e) => {
                        self.state.progress_status = format!("Error: {}", e);
                    }
                }
                
                // Reset progress after delay
                let progress = self.state.progress;
                if progress > 0.0 {
                    self.state.progress = 0.0;
                    self.state.progress_status.clear();
                }
            }
            
            Message::ProgressUpdate(progress, status) => {
                self.state.progress = progress;
                self.state.progress_status = status;
            }
            
            Message::ToggleDarkMode => {
                self.state.dark_mode = !self.state.dark_mode;
                self.save_theme();
            }
            
            Message::ThemeLoaded(dark_mode) => {
                self.state.dark_mode = dark_mode;
            }
            
            Message::OpenFileDialog => {
                // Handled in FilesSelected
            }
        }
        
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .push(self.header_view())
            .push(self.how_it_works_view())
            .push(self.upload_section_view())
            .push(if !self.state.all_pages.is_empty() {
                Column::new()
                    .spacing(20)
                    .push(self.options_section_view())
                    .push(self.thumbnails_section_view())
                    .push(self.export_section_view())
                    .into()
            } else {
                Row::new().into()
            });

        let main_content = container(content)
            .width(Length::Fill)
            .padding(20)
            .center_x(Length::Fill);

        scrollable(main_content)
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(Message::WindowEvent)
    }
}

// ============================================================================
// View Components
// ============================================================================

impl NoteCypher {
    fn header_view(&self) -> Element<Message> {
        let logo = container(
            text("📝")
                .size(24)
        )
        .width(Length::Fixed(40.0))
        .height(Length::Fixed(40.0))
        .center_y(Length::Fixed(40.0))
        .center_x(Length::Fixed(40.0));

        let title = Column::new()
            .push(text("NoteCypher").size(20))
            .push(text("PDF Note Optimizer").size(12));

        let header_content = Row::new()
            .spacing(12)
            .align_y(iced::Alignment::Center)
            .push(logo)
            .push(title);

        let local_badge = container(
            text("🔒 100% Local").size(12)
        )
        .padding([4, 8]);

        let clear_button = if !self.state.pdf_files.is_empty() {
            button(text("Clear All").size(12))
                .padding([8, 16])
                .on_press(Message::ClearAll)
                .into()
        } else {
            Space::with_width(Length::Fixed(0.0)).into()
        };

        let theme_button = button(
            text(if self.state.dark_mode { "☀️" } else { "🌙" }).size(16)
        )
        .padding(8)
        .on_press(Message::ToggleDarkMode);

        let controls = Row::new()
            .spacing(12)
            .align_y(iced::Alignment::Center)
            .push(local_badge)
            .push(clear_button)
            .push(theme_button);

        let row = Row::new()
            .push(header_content)
            .push(Space::with_width(Length::Fill))
            .push(controls);

        container(row)
            .padding([12, 20])
            .into()
    }

    fn how_it_works_view(&self) -> Element<Message> {
        let steps = vec![
            ("📤", "Upload PDFs", "Drag & drop or select multiple PDF files"),
            ("🎯", "Select Pages", "Click thumbnails to choose which pages to include"),
            ("🎨", "Apply Filters", "Invert colors, clear background, or convert to grayscale"),
            ("📐", "Set Layout", "Arrange 1-6 slides per page with custom orientation"),
            ("💾", "Export", "Download your optimized PDF instantly"),
        ];

        let step_views: Vec<Element<_>> = steps
            .into_iter()
            .map(|(icon, title, desc)| {
                Column::new()
                    .spacing(4)
                    .push(text(icon).size(24))
                    .push(text(title).size(14))
                    .push(text(desc).size(11))
                    .width(Length::Fixed(120.0))
                    .into()
            })
            .collect();

        let content = Row::new()
            .spacing(20)
            .extend(step_views);

        container(content)
            .padding(20)
            .into()
    }

    fn upload_section_view(&self) -> Element<Message> {
        let upload_area = button(
            Column::new()
                .spacing(12)
                .align_x(iced::Alignment::Center)
                .push(text("📁").size(40))
                .push(text("Drop PDFs here or click to browse").size(16))
                .push(text("Support multiple files • All processing happens locally").size(12))
        )
        .padding(40)
        .on_press(Message::OpenFileDialog);

        let mut content = Column::new().spacing(12).push(upload_area);

        // PDF Files List
        if !self.state.pdf_files.is_empty() {
            let file_items: Vec<Element<_>> = self.state.pdf_files
                .iter()
                .enumerate()
                .map(|(idx, file)| {
                    let remove_btn = button(
                        text("✕").size(12)
                    )
                    .padding(4)
                    .on_press(Message::RemovePdfFile(idx));

                    Row::new()
                        .spacing(12)
                        .align_y(iced::Alignment::Center)
                        .push(text("📄").size(20))
                        .push(Column::new()
                            .push(text(&file.name).size(14))
                            .push(text(format_file_size(file.size)).size(11))
                        )
                        .push(Space::with_width(Length::Fill))
                        .push(remove_btn)
                        .padding([8, 12])
                        .into()
                })
                .collect();

            let files_list = Column::new()
                .spacing(0)
                .extend(file_items);

            let files_container = container(files_list);

            content = content.push(files_container);
        }

        container(content)
            .padding(20)
            .into()
    }

    fn options_section_view(&self) -> Element<Message> {
        // Filters
        let filter_invert = checkbox("🌓 Invert", self.state.filters.invert)
            .on_toggle(Message::ToggleInvert);
        let filter_clear = checkbox("✨ Clear BG", self.state.filters.clear_background)
            .on_toggle(Message::ToggleClearBackground);
        let filter_gray = checkbox("⚫ Grayscale", self.state.filters.grayscale)
            .on_toggle(Message::ToggleGrayscale);

        let filters_row = Row::new()
            .spacing(20)
            .push(filter_invert)
            .push(filter_clear)
            .push(filter_gray);

        // Layout options
        let layout_options: Vec<Element<_>> = [1, 2, 3, 4, 6]
            .iter()
            .map(|&val| {
                let icon = match val {
                    1 => "▢",
                    2 => "▤",
                    3 => "⋮",
                    4 => "◫",
                    6 => "⊞",
                    _ => "?",
                };
                
                let btn = button(
                    Row::new()
                        .spacing(8)
                        .align_y(iced::Alignment::Center)
                        .push(text(icon).size(20))
                        .push(text(format!("{} slide{}", val, if val == 1 { "" } else { "s" })).size(12))
                )
                .padding([8, 16])
                .on_press(Message::SetLayout(val));
                
                btn.into()
            })
            .collect();

        let layout_row = Row::new()
            .spacing(12)
            .extend(layout_options);

        // Orientation
        let portrait_btn = button(
            Row::new()
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .push(text("📄").size(20))
                .push(text("Portrait").size(12))
        )
        .padding([8, 16])
        .on_press(Message::SetOrientation(Orientation::Portrait));

        let landscape_btn = button(
            Row::new()
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .push(text("📑").size(20))
                .push(text("Landscape").size(12))
        )
        .padding([8, 16])
        .on_press(Message::SetOrientation(Orientation::Landscape));

        let orientation_row = Row::new()
            .spacing(12)
            .push(portrait_btn)
            .push(landscape_btn);

        // Margin
        let margin_row = Row::new()
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .push(text("Margin:").size(12))
            .push(text(format!("{} cm", self.state.margin_cm)).size(12));

        let content = Column::new()
            .spacing(16)
            .push(text("Processing Options").size(18))
            .push(text("Filters").size(14))
            .push(filters_row)
            .push(text("Slides Per Page").size(14))
            .push(layout_row)
            .push(text("Orientation").size(14))
            .push(orientation_row)
            .push(margin_row);

        container(content)
            .padding(20)
            .into()
    }

    fn thumbnails_section_view(&self) -> Element<Message> {
        let selected_count = self.state.selected_pages.len();
        let total_count = self.state.all_pages.len();

        let header = Row::new()
            .spacing(20)
            .align_y(iced::Alignment::Center)
            .push(text("Page Preview").size(18))
            .push(Space::with_width(Length::Fill))
            .push(button(text("Select All").size(12))
                .padding([6, 12])
                .on_press(Message::SelectAllPages))
            .push(button(text("Deselect All").size(12))
                .padding([6, 12])
                .on_press(Message::DeselectAllPages))
            .push(text(format!("{} of {} selected", selected_count, total_count)).size(12));

        // Group pages by PDF
        let mut pdf_groups: Vec<Element<_>> = Vec::new();
        
        for (pdf_idx, pdf_file) in self.state.pdf_files.iter().enumerate() {
            let pdf_index = pdf_idx + 1;
            let pdf_pages: Vec<&PdfPage> = self.state.all_pages
                .iter()
                .filter(|p| p.pdf_index == pdf_index)
                .collect();
            
            if pdf_pages.is_empty() {
                continue;
            }

            // PDF header with select all
            let pdf_selected_count = pdf_pages
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    let global_idx = self.state.all_pages.iter().position(|p| p.id == pdf_pages[*i].id).unwrap_or(0);
                    self.state.selected_pages.contains(&global_idx)
                })
                .count();

            let all_selected = pdf_selected_count == pdf_pages.len();

            let pdf_header = button(
                Row::new()
                    .spacing(12)
                    .align_y(iced::Alignment::Center)
                    .push(text(if all_selected { "☑" } else { "☐" }).size(16))
                    .push(Column::new()
                        .push(text(&pdf_file.name).size(14))
                        .push(text(format!("{} of {} pages selected", pdf_selected_count, pdf_pages.len())).size(11))
                    )
                    .push(Space::with_width(Length::Fill))
                    .push(text("▼").size(10))
            )
            .padding([8, 12])
            .on_press(Message::SelectByPdf(pdf_index));

            // Thumbnail grid
            let thumbnails: Vec<Element<_>> = pdf_pages
                .iter()
                .enumerate()
                .map(|(idx, page)| {
                    let global_idx = self.state.all_pages.iter().position(|p| p.id == page.id).unwrap_or(0);
                    let is_selected = self.state.selected_pages.contains(&global_idx);
                    
                    // Thumbnail placeholder
                    let thumbnail = container(
                        Column::new()
                            .push(Space::with_height(Length::Fixed(100.0)))
                            .push(text(format!("P{}", page.page_number)).size(10))
                    )
                    .width(Length::Fixed(80.0))
                    .height(Length::Fixed(120.0))
                    .center_y(Length::Fill)
                    .center_x(Length::Fill);

                    button(thumbnail)
                        .padding(0)
                        .on_press(Message::TogglePageSelection(global_idx))
                        .into()
                })
                .collect();

            let thumbnail_grid = Row::new()
                .spacing(8)
                .extend(thumbnails);

            let group = Column::new()
                .spacing(8)
                .push(pdf_header)
                .push(thumbnail_grid);

            pdf_groups.push(group.into());
        }

        let content = Column::new()
            .spacing(16)
            .push(header)
            .extend(pdf_groups);

        container(content)
            .padding(20)
            .into()
    }

    fn export_section_view(&self) -> Element<Message> {
        let selected_count = self.state.selected_pages.len();
        let can_export = selected_count > 0 && !self.state.is_processing;

        let button_content = if self.state.is_processing {
            Row::new()
                .spacing(12)
                .align_y(iced::Alignment::Center)
                .push(text("⏳").size(16))
                .push(text("Processing...").size(14))
        } else {
            Row::new()
                .spacing(12)
                .align_y(iced::Alignment::Center)
                .push(text("💾").size(16))
                .push(text("Download Optimized PDF").size(14))
        };

        let export_button = button(button_content)
            .padding([12, 24]);

        let button = if can_export {
            export_button.on_press(Message::ExportPdf)
        } else {
            export_button
        };

        let content = Row::new()
            .spacing(20)
            .align_y(iced::Alignment::Center)
            .push(Column::new()
                .push(text("Ready to Export").size(18))
                .push(text(format!(
                    "{} pages will be processed with your selected options",
                    selected_count
                )).size(12))
            )
            .push(Space::with_width(Length::Fill))
            .push(button);

        container(content)
            .padding(20)
            .into()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    
    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() -> iced::Result {
    iced::application("NoteCypher", NoteCypher::new)
        .window_size(Size::new(1200.0, 800.0))
        .min_window_size(Size::new(800.0, 600.0))
        .run()
}

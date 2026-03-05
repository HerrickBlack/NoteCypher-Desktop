use iced::widget::{
    button, checkbox, column, container, horizontal_space, pick_list, progress_bar, row, scrollable,
    text, vertical_space, Column, Row,
};
use iced::{
    application, Color, Element, Length, Subscription, Task, Theme, Size,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

mod pdf_processor;
use pdf_processor::{PdfDocument, PdfPage, process_pdf, export_pdf, FilterOptions};

// Zed-inspired color palette
const ZED_BG: Color = Color::from_rgb(0.098, 0.098, 0.098);
const ZED_BG_LIGHT: Color = Color::from_rgb(0.137, 0.137, 0.137);
const ZED_SURFACE: Color = Color::from_rgb(0.176, 0.176, 0.176);
const ZED_BORDER: Color = Color::from_rgb(0.235, 0.235, 0.235);
const ZED_TEXT: Color = Color::from_rgb(0.898, 0.898, 0.898);
const ZED_TEXT_MUTED: Color = Color::from_rgb(0.588, 0.588, 0.588);
const ZED_ACCENT: Color = Color::from_rgb(0.259, 0.608, 0.941);
const ZED_SUCCESS: Color = Color::from_rgb(0.294, 0.784, 0.549);
const ZED_ERROR: Color = Color::from_rgb(0.918, 0.333, 0.333);

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
    theme_mode: ThemeMode,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum ThemeMode {
    #[default]
    System,
    Dark,
    Light,
}

impl ThemeMode {
    fn all() -> &'static [ThemeMode] {
        &[ThemeMode::System, ThemeMode::Dark, ThemeMode::Light]
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeMode::System => write!(f, "System"),
            ThemeMode::Dark => write!(f, "Dark"),
            ThemeMode::Light => write!(f, "Light"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    FilesLoaded(Vec<PathBuf>),
    FilesLoadedComplete {
        docs: Vec<PdfDocument>,
        pages: Vec<PdfPage>,
        images: HashMap<usize, Arc<Vec<u8>>>,
    },
    RemovePdfFile(usize),
    ClearAll,
    TogglePageSelection(usize),
    SelectAllPages,
    DeselectAllPages,
    SelectByPdf(usize),
    ToggleInvert(bool),
    ToggleClearBackground(bool),
    ToggleGrayscale(bool),
    SetLayout(u32),
    SetOrientation(Orientation),
    ExportPdf,
    ExportComplete(Result<Vec<u8>, String>),
    SetThemeMode(ThemeMode),
    OpenFile,
}

struct NoteCypher {
    state: AppState,
    image_cache: HashMap<usize, Arc<Vec<u8>>>,
}

impl Default for NoteCypher {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            image_cache: HashMap::new(),
        }
    }
}

impl NoteCypher {
    fn new() -> (Self, Task<Message>) {
        let mut app = NoteCypher {
            state: AppState::default(),
            image_cache: HashMap::new(),
        };
        app.load_theme();
        (app, Task::none())
    }

    fn load_theme(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("NoteCypher").join("config.json");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(mode) = config.get("theme_mode").and_then(|v| v.as_str()) {
                            self.state.theme_mode = match mode {
                                "Dark" => ThemeMode::Dark,
                                "Light" => ThemeMode::Light,
                                _ => ThemeMode::System,
                            };
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
                "theme_mode": self.state.theme_mode.to_string()
            });
            let _ = std::fs::write(
                config_path.join("config.json"),
                serde_json::to_string_pretty(&config).unwrap_or_default(),
            );
        }
    }

    fn is_dark(&self) -> bool {
        match self.state.theme_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => true, // Default to dark for system
        }
    }

    fn bg_color(&self) -> Color {
        if self.is_dark() { ZED_BG } else { Color::WHITE }
    }

    fn surface_color(&self) -> Color {
        if self.is_dark() { ZED_BG_LIGHT } else { Color::from_rgb(0.96, 0.96, 0.96) }
    }

    fn border_color(&self) -> Color {
        if self.is_dark() { ZED_BORDER } else { Color::from_rgb(0.85, 0.85, 0.85) }
    }

    fn text_color(&self) -> Color {
        if self.is_dark() { ZED_TEXT } else { Color::BLACK }
    }

    fn muted_color(&self) -> Color {
        if self.is_dark() { ZED_TEXT_MUTED } else { Color::from_rgb(0.4, 0.4, 0.4) }
    }

    fn accent_color(&self) -> Color {
        ZED_ACCENT
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => {
                return Task::perform(
                    async {
                        let files = rfd::AsyncFileDialog::new()
                            .add_filter("PDF Files", &["pdf"])
                            .set_title("Select PDF Files")
                            .pick_files()
                            .await;
                        files
                            .map(|f| f.iter().map(|fh| fh.path().to_owned()).collect())
                            .unwrap_or(vec![])
                    },
                    Message::FilesLoaded,
                );
            }
            Message::FilesLoaded(paths) => {
                if paths.is_empty() {
                    return Task::none();
                }

                self.state.progress_status = "Loading PDFs...".to_string();
                self.state.progress = 10.0;

                let pdf_paths = paths.clone();
                let page_count = self.state.all_pages.len();

                return Task::perform(
                    async move { process_pdf(pdf_paths, page_count).await },
                    |result| match result {
                        Ok((docs, pages, images)) => Message::FilesLoadedComplete {
                            docs,
                            pages,
                            images,
                        },
                        Err(e) => Message::ExportComplete(Err(e)),
                    },
                );
            }
            Message::FilesLoadedComplete {
                docs,
                pages,
                images,
            } => {
                self.state.pdf_files.extend(docs);
                let base_idx = self.state.all_pages.len();
                for (i, page) in pages.into_iter().enumerate() {
                    self.state.all_pages.push(page);
                    self.state.selected_pages.insert(base_idx + i);
                }
                for (k, v) in images {
                    self.image_cache.insert(k, v);
                }
                self.state.progress = 0.0;
                self.state.progress_status.clear();
            }
            Message::RemovePdfFile(index) => {
                if index >= self.state.pdf_files.len() {
                    return Task::none();
                }
                let pdf_index = index + 1;
                let pages_to_remove: Vec<usize> = self
                    .state
                    .all_pages
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.pdf_index == pdf_index)
                    .map(|(i, _)| i)
                    .collect();
                for &idx in &pages_to_remove {
                    self.image_cache.remove(&idx);
                }
                self.state
                    .all_pages
                    .retain(|p| p.pdf_index != pdf_index);
                self.state.pdf_files.remove(index);
                let new_selection: HashSet<usize> = self
                    .state
                    .selected_pages
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
                let pdf_page_indices: HashSet<usize> = self
                    .state
                    .all_pages
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.pdf_index == pdf_index)
                    .map(|(i, _)| i)
                    .collect();
                let all_selected = pdf_page_indices
                    .iter()
                    .all(|&i| self.state.selected_pages.contains(&i));
                if all_selected {
                    self.state.selected_pages = self
                        .state
                        .selected_pages
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
                return Task::perform(
                    async move {
                        export_pdf(selected, pages, images, filters, layout, orientation, margin_cm)
                            .await
                    },
                    Message::ExportComplete,
                );
            }
            Message::ExportComplete(result) => {
                self.state.is_processing = false;
                match result {
                    Ok(pdf_bytes) => {
                        let timestamp =
                            chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
                        let filename = format!("optimized-notes-{}.pdf", timestamp);
                        let output_path =
                            std::env::current_dir().unwrap_or_default().join(&filename);
                        if std::fs::write(&output_path, pdf_bytes).is_ok() {
                            self.state.progress_status =
                                format!("✓ Saved: {}", filename);
                            self.state.progress = 100.0;
                        } else {
                            self.state.progress_status =
                                "✗ Failed to save file".to_string();
                        }
                    }
                    Err(e) => {
                        self.state.progress_status = format!("✗ Error: {}", e);
                    }
                }
            }
            Message::SetThemeMode(mode) => {
                self.state.theme_mode = mode;
                self.save_theme();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let main_content = column![
            self.header_view(),
            scrollable(
                Column::new()
                    .spacing(24)
                    .padding([24, 40])
                    .push(self.hero_section_view())
                    .push(self.upload_section_view())
                    .push(if !self.state.all_pages.is_empty() {
                        column![
                            self.options_section_view(),
                            self.thumbnails_section_view(),
                            self.export_section_view(),
                        ]
                        .spacing(24)
                    } else {
                        column![]
                    })
                    .push(vertical_space())
            )
            .height(Length::Fill),
        ]
        .height(Length::Fill);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn header_view(&self) -> Element<Message> {
        let logo = row![
            text("◈").size(20).color(self.accent_color()),
            text("NoteCypher").size(18).color(self.text_color())
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let theme_pick = pick_list(
            ThemeMode::all(),
            Some(self.state.theme_mode),
            Message::SetThemeMode,
        )
        .placeholder("Theme")
        .width(Length::Fixed(100.0));

        let header_content = row![logo, horizontal_space(), theme_pick]
            .spacing(16)
            .align_y(iced::Alignment::Center);

        container(header_content)
            .padding([16, 40])
            .into()
    }

    fn hero_section_view(&self) -> Element<Message> {
        if !self.state.all_pages.is_empty() {
            return column![].into();
        }

        let title = text("Optimize Your PDF Notes")
            .size(32)
            .color(self.text_color());

        let subtitle = text("Create clean, printable study materials from your lecture PDFs")
            .size(16)
            .color(self.muted_color());

        let features = row![
            self.feature_badge("📄 Multi-PDF"),
            self.feature_badge("🎨 Smart Filters"),
            self.feature_badge("📐 Custom Layouts"),
            self.feature_badge("🔒 100% Local"),
        ]
        .spacing(16);

        column![title, subtitle, features]
            .spacing(12)
            .align_x(iced::Alignment::Center)
            .into()
    }

    fn feature_badge<'a>(&self, icon: &'a str) -> Element<'a, Message> {
        container(text(icon).size(12).color(self.muted_color()))
            .padding([6, 12])
            .into()
    }

    fn upload_section_view(&self) -> Element<Message> {
        let upload_content = if self.state.pdf_files.is_empty() {
            column![
                text("📁").size(48),
                text("Drop PDFs here or click to browse")
                    .size(16)
                    .color(self.text_color()),
                text("Multiple files supported • All processing happens locally")
                    .size(13)
                    .color(self.muted_color()),
            ]
            .spacing(12)
            .align_x(iced::Alignment::Center)
        } else {
            column![
                text(format!("{} files loaded", self.state.pdf_files.len()))
                    .size(14)
                    .color(self.accent_color()),
            ]
            .spacing(8)
        };

        let upload_button = button(upload_content)
            .padding(40)
            .on_press(Message::OpenFile);

        let mut content = Column::new().spacing(16).push(upload_button);

        if !self.state.pdf_files.is_empty() {
            let file_list: Vec<Element<_>> = self
                .state
                .pdf_files
                .iter()
                .enumerate()
                .map(|(idx, file)| {
                    let remove_btn = button(text("✕").size(12))
                        .padding([4, 10])
                        .on_press(Message::RemovePdfFile(idx));

                    row![
                        text("📄").size(18),
                        Column::new()
                            .push(text(&file.name).size(14).color(self.text_color()))
                            .push(text(format_size(file.size)).size(12).color(self.muted_color())),
                        horizontal_space(),
                        remove_btn,
                    ]
                    .spacing(12)
                    .align_y(iced::Alignment::Center)
                    .into()
                })
                .collect();

            let files_container = container(Column::with_children(file_list).spacing(8))
                .padding(16);

            content = content.push(files_container);
        }

        container(content)
            .width(Length::Fill)
            .into()
    }

    fn options_section_view(&self) -> Element<Message> {
        let section_title = text("Processing Options").size(16).color(self.text_color());

        // Filters
        let filter_title = text("Filters").size(13).color(self.muted_color());
        let filters = row![
            self.toggle_chip("Invert", self.state.filters.invert, Message::ToggleInvert),
            self.toggle_chip(
                "Clear Background",
                self.state.filters.clear_background,
                Message::ToggleClearBackground
            ),
            self.toggle_chip(
                "Grayscale",
                self.state.filters.grayscale,
                Message::ToggleGrayscale
            ),
        ]
        .spacing(10);

        // Layout
        let layout_title = text("Slides per Page").size(13).color(self.muted_color());
        let layout_buttons: Vec<Element<_>> = [1, 2, 3, 4, 6]
            .iter()
            .map(|&val| {
                button(text(format!("{}", val)).size(13))
                    .padding([8, 16])
                    .on_press(Message::SetLayout(val))
                    .into()
            })
            .collect();
        let layout_row = Row::with_children(layout_buttons).spacing(8);

        // Orientation
        let orientation_title = text("Orientation").size(13).color(self.muted_color());

        let portrait_btn = button(text("Portrait").size(13))
            .padding([8, 16])
            .on_press(Message::SetOrientation(Orientation::Portrait));

        let landscape_btn = button(text("Landscape").size(13))
            .padding([8, 16])
            .on_press(Message::SetOrientation(Orientation::Landscape));

        let orientation_row = row![portrait_btn, landscape_btn].spacing(8);

        let options_container = column![
            section_title,
            column![
                filter_title, filters,
                layout_title, layout_row,
                orientation_title, orientation_row,
            ]
            .spacing(8)
        ]
        .spacing(16);

        container(options_container)
            .padding(20)
            .width(Length::Fill)
            .into()
    }

    fn toggle_chip(
        &self,
        label: &str,
        is_active: bool,
        msg: fn(bool) -> Message,
    ) -> Element<Message> {
        let chip = checkbox(label, is_active)
            .on_toggle(msg);

        container(chip)
            .padding([8, 12])
            .into()
    }

    fn thumbnails_section_view(&self) -> Element<Message> {
        let selected_count = self.state.selected_pages.len();
        let total_count = self.state.all_pages.len();

        let header = row![
            text("Pages").size(16).color(self.text_color()),
            horizontal_space(),
            button(text("Select All").size(12))
                .padding([6, 12])
                .on_press(Message::SelectAllPages),
            button(text("Deselect All").size(12))
                .padding([6, 12])
                .on_press(Message::DeselectAllPages),
            text(format!("{} / {} selected", selected_count, total_count))
                .size(12)
                .color(self.muted_color()),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let mut pdf_groups: Vec<Element<_>> = Vec::new();
        for (pdf_idx, pdf_file) in self.state.pdf_files.iter().enumerate() {
            let pdf_index = pdf_idx + 1;
            let pdf_pages: Vec<&PdfPage> = self
                .state
                .all_pages
                .iter()
                .filter(|p| p.pdf_index == pdf_index)
                .collect();

            if pdf_pages.is_empty() {
                continue;
            }

            let pdf_selected_count = pdf_pages
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    let global_idx = self
                        .state
                        .all_pages
                        .iter()
                        .position(|p| p.id == pdf_pages[*i].id)
                        .unwrap_or(0);
                    self.state.selected_pages.contains(&global_idx)
                })
                .count();

            let all_selected = pdf_selected_count == pdf_pages.len();

            let pdf_header = button(
                row![
                    text(if all_selected { "◉" } else { "○" })
                        .size(14)
                        .color(if all_selected { self.accent_color() } else { self.muted_color() }),
                    Column::new()
                        .push(text(&pdf_file.name).size(13).color(self.text_color()))
                        .push(
                            text(format!(
                                "{} pages",
                                pdf_pages.len()
                            ))
                            .size(11)
                            .color(self.muted_color()),
                        ),
                    horizontal_space(),
                    text(format!("{} selected", pdf_selected_count))
                        .size(11)
                        .color(self.muted_color()),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
            .padding([10, 14])
            .on_press(Message::SelectByPdf(pdf_index));

            let thumbnails: Vec<Element<_>> = pdf_pages
                .iter()
                .enumerate()
                .map(|(_idx, page)| {
                    let global_idx = self
                        .state
                        .all_pages
                        .iter()
                        .position(|p| p.id == page.id)
                        .unwrap_or(0);
                    let is_selected = self.state.selected_pages.contains(&global_idx);

                    let thumb_content = column![
                        container(
                            text(format!("P{}", page.page_number))
                                .size(11)
                                .color(if is_selected { Color::WHITE } else { self.muted_color() })
                        )
                        .center_x(Length::Fixed(70.0))
                        .center_y(Length::Fixed(50.0)),
                    ]
                    .align_x(iced::Alignment::Center);

                    let thumb = container(thumb_content)
                        .width(Length::Fixed(70.0))
                        .height(Length::Fixed(70.0));

                    button(thumb)
                        .padding(0)
                        .on_press(Message::TogglePageSelection(global_idx))
                        .into()
                })
                .collect();

            let thumbnail_grid = Row::with_children(thumbnails).spacing(8);
            let group = column![pdf_header, thumbnail_grid].spacing(10);
            pdf_groups.push(group.into());
        }

        let content = column![header, Column::with_children(pdf_groups).spacing(16)].spacing(16);

        container(content)
            .padding(20)
            .width(Length::Fill)
            .into()
    }

    fn export_section_view(&self) -> Element<Message> {
        let selected_count = self.state.selected_pages.len();
        let can_export = selected_count > 0 && !self.state.is_processing;

        let status_color = if self.state.progress_status.starts_with('✓') {
            ZED_SUCCESS
        } else if self.state.progress_status.starts_with('✗') {
            ZED_ERROR
        } else {
            self.muted_color()
        };

        let export_button = button(
            row![
                if self.state.is_processing {
                    text("⟳").size(16)
                } else {
                    text("⬇").size(16)
                },
                text(if self.state.is_processing {
                    "Processing..."
                } else {
                    "Export PDF"
                })
                .size(14),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .padding([12, 24]);

        let button = if can_export {
            export_button.on_press(Message::ExportPdf)
        } else {
            export_button
        };

        let mut export_content = column![
            row![
                text("Ready to Export").size(18).color(self.text_color()),
                horizontal_space(),
                button,
            ]
            .spacing(16)
            .align_y(iced::Alignment::Center),
            text(format!("{} pages will be processed", selected_count))
                .size(13)
                .color(self.muted_color()),
        ]
        .spacing(8);

        if !self.state.progress_status.is_empty() {
            export_content = export_content.push(
                text(&self.state.progress_status)
                    .size(13)
                    .color(status_color),
            );
        }

        if self.state.is_processing {
            export_content = export_content.push(
                progress_bar(0.0..=100.0, self.state.progress).height(4),
            );
        }

        container(export_content)
            .padding(24)
            .width(Length::Fill)
            .into()
    }
}

fn format_size(bytes: u64) -> String {
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

fn main() -> iced::Result {
    application("NoteCypher", NoteCypher::update, NoteCypher::view)
        .subscription(NoteCypher::subscription)
        .theme(NoteCypher::theme)
        .window_size(Size::new(1100.0, 750.0))
        .run_with(NoteCypher::new)
}

impl NoteCypher {
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn theme(&self) -> Theme {
        if self.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

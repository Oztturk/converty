#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    Photo,
    Video,
    Gif,
}

pub fn get_category(ext: &str) -> Option<Category> {
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff" | "tif" | "ico" | "avif" | "heic" => {
            Some(Category::Photo)
        }
        "mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" | "wmv" => Some(Category::Video),
        "gif" => Some(Category::Gif),
        _ => None,
    }
}

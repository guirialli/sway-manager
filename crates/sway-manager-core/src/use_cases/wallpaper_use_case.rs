use crate::domain::traits::WallpaperRepository;
use std::path::Path;

pub struct WallpaperUseCase<R: WallpaperRepository> {
    repo: R,
}

impl<R: WallpaperRepository> WallpaperUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn set_wallpaper(&self, image_path: &Path) -> Result<(), String> {
        self.repo.set_wallpaper(image_path)
    }
}

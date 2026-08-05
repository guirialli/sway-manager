use crate::domain::entities::DisplayLayout;
use crate::domain::traits::DisplayRepository;

pub struct DisplayUseCase<R: DisplayRepository> {
    repo: R,
}

impl<R: DisplayRepository> DisplayUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn apply_layout(&self, layout: DisplayLayout) -> Result<(), String> {
        self.repo.apply_layout(layout)
    }
}

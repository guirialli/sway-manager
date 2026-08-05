use crate::domain::entities::IdleState;
use crate::domain::traits::IdleRepository;

pub struct IdleUseCase<R: IdleRepository> {
    repo: R,
}

impl<R: IdleRepository> IdleUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn get_status(&self) -> Result<IdleState, String> {
        self.repo.get_state()
    }

    pub fn toggle_idle(&self) -> Result<bool, String> {
        self.repo.toggle_idle()
    }
}

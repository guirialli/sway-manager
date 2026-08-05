use crate::domain::entities::BatteryState;
use crate::domain::traits::BatteryRepository;

pub struct BatteryUseCase<R: BatteryRepository> {
    repo: R,
}

impl<R: BatteryRepository> BatteryUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn get_status(&self) -> Result<BatteryState, String> {
        self.repo.get_state()
    }

    pub fn toggle_conservation_mode(&self) -> Result<bool, String> {
        self.repo.toggle_conservation_mode()
    }
}

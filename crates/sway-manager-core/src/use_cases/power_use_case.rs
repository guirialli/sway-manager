use crate::domain::entities::PowerProfile;
use crate::domain::traits::PowerRepository;

pub struct PowerUseCase<R: PowerRepository> {
    repo: R,
}

impl<R: PowerRepository> PowerUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn get_profile(&self) -> Result<PowerProfile, String> {
        self.repo.get_profile()
    }

    pub fn set_profile(&self, profile: PowerProfile) -> Result<(), String> {
        self.repo.set_profile(profile)
    }

    pub fn toggle_profile(&self) -> Result<PowerProfile, String> {
        let current = self.repo.get_profile()?;
        let next = match current {
            PowerProfile::Balanced => PowerProfile::Performance,
            PowerProfile::Performance => PowerProfile::PowerSaver,
            PowerProfile::PowerSaver => PowerProfile::Balanced,
            PowerProfile::Unknown(_) => PowerProfile::Balanced,
        };
        self.repo.set_profile(next.clone())?;
        Ok(next)
    }
}

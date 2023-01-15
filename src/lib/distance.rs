pub struct DistanceMeasurer {
    rising: Option<u16>,
    distance: u16,
}

impl DistanceMeasurer {
    pub fn get_distance_cm(&self) -> u16 {
        self.distance / 58
    }

    /// The t should be given in microseconds
    pub fn update_measurment(&mut self, t: u16) {
        self.rising = match self.rising {
            None => Some(t),
            Some(p) => {
                self.distance = t.wrapping_sub(p);
                None
            }
        }
    }

    pub const fn new() -> Self {
        Self {
            rising: None,
            distance: u16::MAX,
        }
    }
}

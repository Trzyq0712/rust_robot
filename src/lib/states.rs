use super::robot::{Motor, SensorReadings};
pub trait State {
    fn process_state(&self, sr: &SensorReadings, left: &mut Motor, right: &mut Motor)
        -> &dyn State;
}

struct FollowLine;

impl State for FollowLine {
    fn process_state(
        &self,
        sr: &SensorReadings,
        left: &mut Motor,
        right: &mut Motor,
    ) -> &dyn State {
        self
    }
}

struct ObstacleStop;

impl State for ObstacleStop {
    fn process_state(
        &self,
        sr: &SensorReadings,
        left: &mut Motor,
        right: &mut Motor,
    ) -> &dyn State {
        if sr.front_distance.unwrap_or(u16::MAX) < 15 {
            left.stop();
            right.stop();
        } else {
            left.forward(left.max_duty());
            right.forward(right.max_duty());
        }
        self
    }
}

struct Stopped;

impl State for Stopped {
    fn process_state(
        &self,
        _sr: &SensorReadings,
        left: &mut Motor,
        right: &mut Motor,
    ) -> &dyn State {
        left.stop();
        right.stop();
        self
    }
}

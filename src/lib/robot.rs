use super::states::State;

type Cm = u16;

#[derive(Default)]
pub struct SensorReadings {
    pub front_distance: Option<Cm>,
    pub left_distance: Option<Cm>,
    pub left_infrared: u16,
    pub right_infrared: u16,
}

#[derive(Clone, Copy)]
pub enum Dir {
    Fd,
    Bk,
}

pub struct Motor<'a> {
    duty: u16,
    dir: Dir,
    get_max_duty: &'a dyn Fn() -> u16,
    set_duty: &'a dyn Fn(u16, Dir),
}

impl<'a> Motor<'a> {
    pub fn new(get_max_duty: &'a dyn Fn() -> u16, set_duty: &'a dyn Fn(u16, Dir)) -> Motor<'a> {
        set_duty(0, Dir::Fd);
        Self {
            duty: 0,
            dir: Dir::Fd,
            get_max_duty,
            set_duty,
        }
    }

    pub fn forward(&mut self, duty: u16) {
        self.duty = duty;
        self.dir = Dir::Fd;
        (self.set_duty)(duty, Dir::Fd);
    }

    pub fn backward(&mut self, duty: u16) {
        self.duty = duty;
        self.dir = Dir::Bk;
        (self.set_duty)(duty, Dir::Bk);
    }

    pub fn stop(&mut self) {
        self.duty = 0;
        (self.set_duty)(0, self.dir);
    }

    pub fn get_info(&self) -> (u16, Dir) {
        (self.duty, self.dir)
    }

    pub fn max_duty(&self) -> u16 {
        (self.get_max_duty)()
    }
}

pub struct Robot<'a> {
    sensors: SensorReadings,
    motors: (Motor<'a>, Motor<'a>),
    state: &'a dyn State,
}

impl<'a> Robot<'a> {
    pub fn new(motors: (Motor<'a>, Motor<'a>), state: &'a dyn State) -> Self {
        Self {
            sensors: SensorReadings::default(),
            motors,
            state,
        }
    }

    pub fn update_sensors(&mut self, sr: SensorReadings) {
        self.sensors = sr;
    }

    pub fn step_state(&mut self) {
        self.state =
            self.state
                .process_state(&self.sensors, &mut self.motors.0, &mut self.motors.1);
    }
}

use super::robot::Robot;
use cortex_m::asm;

const MAX: u16 = 40_000;

pub enum State {
    FollowingLine,
    FollowingLineAndAvoiding,
    Forward,
    TurningLeft,
    TurningRight(u8),
    Avoiding,
    Stopped,
}

impl State {
    pub fn process_state(self, robot: &mut Robot) -> Self {
        match self {
            State::FollowingLine => following_line(robot),
            State::Stopped => State::Stopped,
            State::FollowingLineAndAvoiding => following_line_and_avoiding(robot),
            State::TurningRight(next) => turning_right(robot, next),
            State::TurningLeft => turning_left(robot),
            State::Avoiding => avoiding(robot),
            State::Forward => forward(robot),
        }
    }
}

fn following_line(robot: &mut Robot) -> State {
    let readings = robot.get_sensor_readings();
    let left_is_black = readings.left_infrared > 200;
    let right_is_black = readings.right_infrared > 200;
    let max_duty = robot.left_motor().get_max_duty() - 10_000;
    match (left_is_black, right_is_black) {
        (true, false) => {
            robot.left_motor().backward(max_duty);
            robot.right_motor().forward(max_duty);
        }
        (false, true) => {
            robot.left_motor().forward(max_duty);
            robot.right_motor().backward(max_duty);
        }
        _ => {
            robot.left_motor().forward(max_duty);
            robot.right_motor().forward(max_duty);
        }
    }
    State::FollowingLine
}

fn following_line_and_avoiding(robot: &mut Robot) -> State {
    if robot.get_sensor_readings().front_distance > 15 {
        following_line(robot);
        State::FollowingLineAndAvoiding
    } else {
        State::TurningRight(0)
    }
}

fn turning_right(robot: &mut Robot, next: u8) -> State {
    robot.left_motor().forward(MAX);
    robot.right_motor().backward(MAX - 5_000);
    asm::delay(10_000_000);
    match next {
        0 => {
            robot.left_motor().forward(MAX);
            robot.right_motor().forward(MAX - 5_000);
            asm::delay(7_000_000);
            State::Avoiding
        }
        1 => State::FollowingLineAndAvoiding,
        _ => State::Stopped,
    }
}

fn turning_left(robot: &mut Robot) -> State {
    robot.left_motor().backward(MAX);
    robot.right_motor().forward(MAX - 5_000);
    asm::delay(9_000_000);
    State::Forward
}

fn avoiding(robot: &mut Robot) -> State {
    let sr = robot.get_sensor_readings();
    if sr.left_infrared > 300 || sr.right_infrared > 300 {
        // robot.left_motor().backward(MAX);
        // robot.right_motor().backward(MAX - 5_000);
        // asm::delay(5_000_000);
        State::TurningRight(1)
        // robot.left_motor().stop();
        // robot.right_motor().stop();
        // State::Stopped
    } else if sr.left_distance > 30 {
        asm::delay(3_000_000);
        State::TurningLeft
    } else if sr.front_distance < 15 {
        State::TurningRight(0)
    } else {
        robot.left_motor().forward(MAX);
        robot.right_motor().forward(MAX - 7_000);
        State::Avoiding
    }
}

fn forward(robot: &mut Robot) -> State {
    if robot.get_sensor_readings().left_distance > 30 {
        robot.left_motor().forward(MAX);
        robot.right_motor().forward(MAX - 7_000);
        State::Forward
    } else {
        State::Avoiding
    }
}

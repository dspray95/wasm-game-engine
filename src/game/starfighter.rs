use cgmath::{ vec3, Vector3 };

pub struct Starfighter {
    current_direction: String,
    upper_limit: f32,
    lower_limit: f32,
    left_limit: f32,
    right_limit: f32,
}

impl Starfighter {
    pub fn new(start_position: Vector3<f32>) -> Starfighter {
        Starfighter {
            current_direction: "up".to_string(),
            upper_limit: -0.45,
            lower_limit: -0.55,
            left_limit: start_position.x + 1.0,
            right_limit: start_position.x - 1.0,
        }
    }

    pub fn player_control(
        &mut self,
        is_left_pressed: bool,
        is_right_pressed: bool,
        current_position: Vector3<f32>,
        delta_time: f32
    ) -> Vector3<f32> {
        let movement_speed = 4.0;

        let x_movement = {
            if is_left_pressed {
                movement_speed * delta_time
            } else if is_right_pressed {
                -movement_speed * delta_time
            } else {
                0.0
            }
        };

        let new_x_pos = (current_position.x + x_movement)
            .max(self.right_limit)
            .min(self.left_limit);

        vec3(new_x_pos, current_position.y, current_position.z)
    }

    pub fn animate(&mut self, current_position: Vector3<f32>, delta_time: f32) -> Vector3<f32> {
        let mut new_position = current_position.clone();
        if self.current_direction == "up" {
            new_position.y = current_position.y + 0.2 * delta_time;
            if new_position.y >= self.upper_limit {
                self.current_direction = "down".to_string();
            }
        } else {
            new_position.y = current_position.y - 0.2 * delta_time;
            if new_position.y <= self.lower_limit {
                self.current_direction = "up".to_string();
            }
        }

        new_position
    }
}

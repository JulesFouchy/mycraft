use winit::event::*;      

pub struct Camera {
      position: cgmath::Point3<f32>,
      angle_ground: cgmath::Rad<f32>,
      angle_up: cgmath::Rad<f32>,
      pub aspect: f32,
      fovy: f32,
      znear: f32,
      zfar: f32,
}

impl Camera {
      pub fn new(aspect: f32) -> Self {
            Self {
                  position: (-10.0, 2.0, 1.0).into(),
                  angle_ground: cgmath::Rad(0.),
                  angle_up: cgmath::Rad(0.),
                  aspect,
                  fovy: 45.0,
                  znear: 0.1,
                  zfar: 100.0,
            }
      }

      pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
            let view = cgmath::Matrix4::look_at_rh(self.position, self.position + self.look_direction(), cgmath::Vector3::unit_z());
            let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
            proj * view
      }

      fn look_direction(&self) -> cgmath::Vector3<f32> {
            use cgmath::Angle;
            return (
                  Angle::cos(self.angle_up) * Angle::cos(self.angle_ground),
                  Angle::cos(self.angle_up) * Angle::sin(self.angle_ground),
                  Angle::sin(self.angle_up),
            ).into()
      }

      fn forward_direction(&self) -> cgmath::Vector3<f32> {
            use cgmath::Angle;
            return (
                  Angle::cos(self.angle_ground),
                  Angle::sin(self.angle_ground),
                  0.,
            ).into()
      }

      fn right_direction(&self) -> cgmath::Vector3<f32> {
            use cgmath::Angle;
            return (
                  Angle::sin(self.angle_ground),
                  -Angle::cos(self.angle_ground),
                  0.,
            ).into()
      }
}

pub struct CameraController {
      speed: f32,
      angle_ground_delta: cgmath::Rad<f32>,
      angle_up_delta: cgmath::Rad<f32>,
      is_up_pressed: bool,
      is_down_pressed: bool,
      is_forward_pressed: bool,
      is_backward_pressed: bool,
      is_left_pressed: bool,
      is_right_pressed: bool,
}

impl CameraController {
      pub fn new(speed: f32) -> Self {
            Self {
                  speed,
                  angle_ground_delta: cgmath::Rad(0.),
                  angle_up_delta: cgmath::Rad(0.),
                  is_up_pressed: false,
                  is_down_pressed: false,
                  is_forward_pressed: false,
                  is_backward_pressed: false,
                  is_left_pressed: false,
                  is_right_pressed: false,
            }
      }

      pub fn process_events(&mut self, event: &WindowEvent) -> bool {
            match event {
                  WindowEvent::KeyboardInput {
                  input:
                        KeyboardInput {
                              state,
                              scancode,
                              ..
                        },
                  ..
                  } => {
                  let is_pressed = *state == ElementState::Pressed;
                  match scancode {
                        57 /*space*/ => {
                              self.is_up_pressed = is_pressed;
                              true
                        }
                        42 /*shift*/ => {
                              self.is_down_pressed = is_pressed;
                              true
                        }
                        17 /*W*/ => {
                              self.is_forward_pressed = is_pressed;
                              true
                        }
                        30 /*A*/ => {
                              self.is_left_pressed = is_pressed;
                              true
                        }
                        31 /*S*/ => {
                              self.is_backward_pressed = is_pressed;
                              true
                        }
                        32 /*D*/ => {
                              self.is_right_pressed = is_pressed;
                              true
                        }
                        _ => false,
                  }
                  }
                  _ => false,
            }
      }

      pub fn process_device_event(&mut self, event: &DeviceEvent, is_cursor_captured: bool) -> bool {
            match event {
                  DeviceEvent::MouseMotion {
                        delta,
                        ..
                  } => {
                        if is_cursor_captured {
                              self.angle_ground_delta -= cgmath::Rad(delta.0 as f32);
                              self.angle_up_delta     -= cgmath::Rad(delta.1 as f32);
                              true
                        }
                        else {
                              false
                        }
                  }
                  _ => false,
            }
      }

      pub fn update_camera(&mut self, camera: &mut Camera) {
            const ZERO: cgmath::Vector3<f32> = cgmath::Vector3{x: 0., y: 0., z: 0.};
            let direction =
                  if self.is_forward_pressed  {  camera.forward_direction() } else { ZERO } +
                  if self.is_backward_pressed { -camera.forward_direction() } else { ZERO } +
                  if self.is_right_pressed    {  camera.right_direction  () } else { ZERO } +
                  if self.is_left_pressed     { -camera.right_direction  () } else { ZERO } +
                  if self.is_up_pressed       {  cgmath::Vector3::unit_z () } else { ZERO } +
                  if self.is_down_pressed     { -cgmath::Vector3::unit_z () } else { ZERO }
            ;
            let magnitude = cgmath::InnerSpace::magnitude(direction);
            if magnitude > 0.001 {
                  camera.position += direction / magnitude * self.speed;
            }
            camera.angle_ground += self.angle_ground_delta * 0.001;
            camera.angle_up     += self.angle_up_delta     * 0.001; 
            self.angle_ground_delta = cgmath::Rad(0.);
            self.angle_up_delta = cgmath::Rad(0.);
      }
}
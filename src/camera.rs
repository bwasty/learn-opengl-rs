#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cgmath;
use cgmath::{vec3};
use cgmath::prelude::*;

type Point3 = cgmath::Point3<f32>;
type Vector3 = cgmath::Vector3<f32>;
type Matrix4 = cgmath::Matrix4<f32>;

// Defines several possible options for camera movement. Used as abstraction to stay away from window-system specific input methods
pub enum Camera_Movement {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT
}

// Default camera values
const YAW: f32        = -90.0;
const PITCH: f32      =  0.0;
const SPEED: f32      =  2.5;
const SENSITIVTY: f32 =  0.1;
const ZOOM: f32       =  45.0;

struct Camera {
    // Camera Attributes
    pub Position: Vector3,
    pub Front: Vector3,
    pub Up: Vector3,
    pub Right: Vector3,
    pub WorldUp: Vector3,
    // Euler Angles
    pub Yaw: f32,
    pub Pitch: f32,
    // Camera options
    pub MovementSpeed: f32,
    pub MouseSensitivity: f32,
    pub Zoom: f32,
}

impl Camera {
    fn new(position: Vector3, up: Vector3, yaw: f32, pitch: f32) -> Camera {
        Camera {
            Position: position,
            WorldUp: up,
            Yaw: yaw,
            Pitch: pitch,
            ..Camera::default()
        }
    }

    // TODO? constructor with scalar values

    /// Calculates the front vector from the Camera's (updated) Eular Angles
    fn updateCameraVectors(&mut self) {
        // Calculate the new Front vector
        let front = Vector3 {
            x: self.Yaw.to_radians().cos() * self.Pitch.to_radians().cos(),
            y: self.Pitch.to_radians().sin(),
            z: self.Yaw.to_radians().sin() * self.Pitch.to_radians().cos()
        };
        self.Front = front.normalize();
        // Also re-calculate the Right and Up vector
        self.Right = self.Front.cross(self.WorldUp).normalize(); // Normalize the vectors, because their length gets closer to 0 the more you look up or down which results in slower movement.
        self.Up = self.Right.cross(self.Front).normalize();
    }
}

impl Default for Camera {
    fn default() -> Camera {
        let mut camera = Camera {
            Position: Vector3::zero(),
            Front: vec3(0.0, 0.0, -1.0),
            Up: Vector3::zero(),        // initialized later
            Right: Vector3::zero(),     // initialized later
            WorldUp: Vector3::unit_y(),
            Yaw: YAW,
            Pitch: PITCH,
            MovementSpeed: SPEED,
            MouseSensitivity: SENSITIVTY,
            Zoom: ZOOM,
        };
        camera.updateCameraVectors();
        camera
    }
}

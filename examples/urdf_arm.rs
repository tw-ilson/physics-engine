use std::f32::consts::PI;

use std::str::FromStr;
use wgpu_robotic_simulator::bindings::*;
use wgpu_robotic_simulator::geometry::{BoxMesh, CylinderMesh, Polyhedron, TriMesh};
use wgpu_robotic_simulator::graphics::GraphicsProgram;
use wgpu_robotic_simulator::robot::RobotGraphics;
use wgpu_robotic_simulator::shader::CreatePipeline;
use wgpu_robotic_simulator::urdf::*;
use wgpu_robotic_simulator::wgpu_program::{MeshBuffer, WGPUGraphics};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub fn run() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = winit::window::Window::new(&event_loop)?;
    let mut program = WGPUGraphics::new(1240, 860, &window);
    program.get_backend_info();

    let mut robot = RobotDescriptor::from_str(include_str!("../assets/xarm.urdf"))
        .expect("unable to read urdf");

    //Initialize uniform buffers
    let camera_buffer = program.create_camera_buffer();
    let light_buffer = program.create_light_buffer();
    let transform_buffers = program.robot_create_transform_buffers(&robot);
    let mesh_buffers = program.robot_create_mesh_buffers(&robot);
    program.create_bindings(&light_buffer, &camera_buffer, &transform_buffers);

    // Create pipeline from vertex, fragment shaders
    let pipeline = program
        .create_render_pipeline(include_str!("../shaders/shader.wgsl"))
        .expect("failed to get render pipeline!");

    let mut increment = 0.0;
    program.preloop(&mut |_| {
        println!("Called one time before the loop!");
    });
    event_loop.run(move |event, control_flow| {
        match event {
            // INPUT
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == program.window.id() => {
                match event {
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(keycode),
                                ..
                            },
                        ..
                    } => match keycode {
                        KeyCode::Escape | KeyCode::KeyQ => control_flow.exit(),
                        keycode => {
                            program.process_keyboard(keycode);
                        }
                    },
                    WindowEvent::RedrawRequested => {
                        program.window.request_redraw();
                        //UPDATE
                        program.update(&mut |p| {
                            p.update_camera(&camera_buffer);
                            p.update_light(&light_buffer);
                            increment = (increment + 0.02) % (2.0 * PI);
                            robot.set_joint_position(
                                &[
                                    0.,
                                    0.,
                                    increment.cos(),
                                    -increment.cos(),
                                    -increment.cos(),
                                    0.,
                                    0.,
                                    0.,
                                    0.,
                                    0.,
                                    0.,
                                    0.,
                                ],
                                false,
                            );
                            robot.build();
                            p.robot_assign_transform_buffers(&robot, &transform_buffers);
                        });

                        // RENDER
                        program.render(&mut |p| {
                            p.draw_robot(&robot, &mesh_buffers, &pipeline);
                        });
                    }
                    // WindowEvent::DeviceEvent {
                    //     event: DeviceEvent::MouseMotion{ delta, },
                    //     .. // We're not using device_id currently
                    // } =>  {
                    //     // program.mouse_look(
                    //     //     delta.0 as f32,
                    //     //     0.0
                    //     //     // delta.1 as f32
                    //     //     )
                    // },
                    _ => {}
                }
            }
            _ => {}
        }
    })?;
    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    run()?;
    Ok(())
}

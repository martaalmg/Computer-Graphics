// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::f32::consts::PI;
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  pointer_to_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()


// == // Generate your VAO here
// assignment 2 - adjust to take another float vector as parameter -> normals
// should contain color values -> rgba
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, rgba: &Vec<f32>) -> u32 {
    // function must take vector of 3d vertex coord.
    // vertex coord -> parameter 1, array of indices -> param 2
    
    // contents of buffer can be assumed to exclusively contain triangles

    let mut vertexArrayID: u32 = 0; //type defined from paramters
    let mut vertexBufferID: u32 = 0;
    let mut vboColor: u32 = 0; //added for adding color

    //VAO setup    
    //gets the ID of the generated VAO, must use ID to refer to the array
    gl::GenVertexArrays(1, &mut vertexArrayID); //requires a pointer to a location where the IDs can be stored
    //need to bind before linking VBO
    gl::BindVertexArray(vertexArrayID);

    //VBO setup
    //create the VBO:
    gl::GenBuffers(1, &mut vertexBufferID);
    //need to bind before can be modified
    gl::BindBuffer(gl::ARRAY_BUFFER, vertexBufferID);
    //filling the buffer
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(&vertices), pointer_to_array(&vertices), gl::STATIC_DRAW);

    let vap_index = 0; //specifies the index of the vertex pointer to set
    //create a VAP (specifies where the vertex shader can obtain the data for a particular vertex attribute and how it is formatted)
    gl::VertexAttribPointer(vap_index, 3, gl::FLOAT, gl::FALSE, 0, ptr::null()); //size 3 because we want xyz, single point so just using null ptr
    //enabling the vertex attributes
    gl::EnableVertexAttribArray(vap_index); 

    //additional float vector should be put into a VBO, treat like earlier VBO setup
    let color_index = 1;
    gl::GenBuffers(1, &mut vboColor);
    gl::BindBuffer(gl::ARRAY_BUFFER, vboColor);
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(&rgba), pointer_to_array(&rgba), gl::STATIC_DRAW);
    gl::VertexAttribPointer(color_index, 4, gl::FLOAT, gl::FALSE, 0, ptr::null()); //bc colors consists of 4 floats
    gl::EnableVertexAttribArray(color_index);


    //in order to combine vertices into primitives:
    //generate and bind another buffer:
    let mut indexBufferID: u32 = 0;
    gl::GenBuffers(1, &mut indexBufferID);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indexBufferID);
    //fill it with data
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, byte_size_of_array(&indices), pointer_to_array(&indices), gl::STATIC_DRAW);
    //dont need to call VertexAttributPointer to set up IBO
    
    //return VAO ID
    vertexArrayID
}


fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO around here
        //vertices for the vao
        //first set of pointsv -> 1.c
        let vertices: Vec<f32> = vec![
            // -0.3, -0.3, 0.5,
            // 0.7, -0.3, 0.5,
            // 0.2, 0.7, 0.5,

            // -0.5, -0.5, 0.0,
            // 0.5, -0.5, 0.0,
            // 0.0, 0.5, 0.0,

            // -0.2, -0.2, -0.5,
            // 0.3, -0.2, -0.5,
            // 0.8, 0.3, -0.5,

            0.6, 0.1, 0.0,
            0.6, 0.3, 0.0,
            0.2, 0.2, 0.0,

            -0.6, 0.8, 0.0,
            -0.6, 0.3, 0.0,
            -0.4, 0.9, 0.0,

            0.3, -0.2, 0.0,
            0.1, -0.2, 0.0,
            0.2, -0.4, 0.0,

            // -0.7, -0.3, 0.0,
            // -0.3, -0.7, 0.0,
            // -0.2, -0.4, 0.0,

            // 0.3, 0.4, 0.0,
            // -0.4, 0.6, 0.0,
            // 0.2, 0.3, 0.0,

            // //task 2
            // 0.6, -0.8, -3.0,
            // 0.0, 0.4, 0.0,
            // -0.8, -0.2, 3.0

        ];

        //set up rgba
        // format -> [r g b a] , a(alpha) is transparency 
        let rgba: Vec<f32> = vec![
            0.0, 1.0, 0.0, 0.3,
            0.0, 1.0, 0.0, 0.3,
            0.0, 1.0, 0.0, 0.3,

            1.0, 0.0, 0.0, 0.5,
            1.0, 0.0, 0.0, 0.5,
            1.0, 0.0, 0.0, 0.5,

            0.0, 0.0, 1.0, 0.5,
            0.0, 0.0, 1.0, 0.5,
            0.0, 0.0, 1.0, 0.5,
        ];

        //indicies for the vao -> 15 bc 3*5 for total vertices
        let indices: Vec<u32> = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8
        ];

        //create vao using vertices & indices
        let my_vao = unsafe { create_vao(&vertices, &indices, &rgba) };


        // == // Set up your shaders here

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once

        //part 2 ->loading & linking shader (3.3 in openGL book)
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("../shaders/simple.vert")
                .attach_file("../shaders/simple.frag")
                .link()
        };

        //enabling the program object
        unsafe {
            gl::UseProgram(simple_shader.program_id);
        };
        


        // Used to demonstrate keyboard handling for exercise 2.
        let mut _arbitrary_number = 0.0; // feel free to remove

        let mut pos = glm::vec3(0.0, 0.0, 0.0);
        let mut rot = glm::vec2(0.0, 0.0);


        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // Handle keyboard input
            // need 2 keys for each rotations/translation axis. one for forward direction of motion and one for backward
            // default -> WASD, Space, and Lshift
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        //using WASD
                        // A, move -x
                        // D, move +x
                        // W, move +y
                        // S, move -y
                        // Lshift, move +z
                        // Space, move -z
                        // for rotating view, using LRUD convention
                        // Left, rotate left
                        // Right, rotate right
                        // Up, rotate forwards
                        // Down, rotate backwards
                        VirtualKeyCode::A => {
                            pos[0] += delta_time;
                        }
                        VirtualKeyCode::D => {
                            pos[0] -= delta_time;
                        }
                        VirtualKeyCode::W => {
                            pos[1] += delta_time;
                        }
                        VirtualKeyCode::S => {
                            pos[1] -= delta_time;
                        }
                        VirtualKeyCode::LShift => {
                            pos[2] += delta_time;
                        }
                        VirtualKeyCode::Space => {
                            pos[2] -= delta_time;
                        }
                        VirtualKeyCode::Right => {
                            rot[1] += delta_time;
                        }
                        VirtualKeyCode::Left => {
                            rot[1] -= delta_time;
                        }
                        VirtualKeyCode::Down => {
                            rot[0] += delta_time;
                        }
                        VirtualKeyCode::Up => {
                            rot[0] -= delta_time;
                        }
                        // default handler:
                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)
            let mut transf_matrix: glm::Mat4 = glm::identity();
           // let mut theta = 0.0;

           transf_matrix *= glm::perspective(window_aspect_ratio, PI/2.0, 1.0, 100.0); //flips the z-axis
           //to ensure drawing isnt out of view:
           transf_matrix *= glm::translation(&glm::vec3(0.0, 0.0, -1.5)) * glm::translation(&pos);
           //mimic behavior of camera- wasd, lrup
           transf_matrix *= glm::rotation(rot[0], &glm::vec3(1.0, 0.0, 0.0)) * glm::rotation(rot[1], &glm::vec3(0.0, 1.0, 0.0));

            //4b, projection

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                //creating transf matrix using uniform matrix property gl, p.33
                //using location 2 from simple.vert
                gl::UniformMatrix4fv(2, 1, gl::FALSE, transf_matrix.as_ptr()); //need to use as ptr as recommended in assignment

                // == // Issue the necessary gl:: commands to draw your scene here
                //first step of drawing a VAO is to bind it
                gl::BindVertexArray(my_vao);
                gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_INT, ptr::null());


            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}

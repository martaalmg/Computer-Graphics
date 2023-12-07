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
use glm::Mat4;
use std::{ mem, ptr, os::raw::c_void };
use std::f32::consts::PI;
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod mesh;
mod util;
mod scene_graph;
mod toolbox;
use scene_graph::SceneNode;

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
// adjust to take another float vector as parameter -> normals
// should contain color values -> rgba
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, rgba: &Vec<f32>, normals: &Vec<f32>) -> u32 {
    // function must take vector of 3d vertex coord.
    // vertex coord -> parameter 1, array of indices -> param 2
    
    // contents of buffer can be assumed to exclusively contain triangles

    let mut vertexArrayID: u32 = 0; //type defined from paramters
    let mut vertexBufferID: u32 = 0;
    let mut vboColor: u32 = 0; //added for adding color
    let mut vboNormal: u32 = 0; //added for the normals

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

    //in order to combine vertices into primitives:
    //generate and bind another buffer:
    let mut indexBufferID: u32 = 0;
    gl::GenBuffers(1, &mut indexBufferID);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indexBufferID);
    //fill it with data
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, byte_size_of_array(&indices), pointer_to_array(&indices), gl::STATIC_DRAW);
    //dont need to call VertexAttributPointer to set up IBO

    //additional float vector should be put into a VBO, treat like earlier VBO setup
    let color_index = 1;
    gl::GenBuffers(1, &mut vboColor);
    gl::BindBuffer(gl::ARRAY_BUFFER, vboColor);
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(&rgba), pointer_to_array(&rgba), gl::STATIC_DRAW);
    gl::VertexAttribPointer(color_index, 4, gl::FLOAT, gl::FALSE, 0, ptr::null()); //bc colors consists of 4 floats
    gl::EnableVertexAttribArray(color_index);

    //generate a VBO to take in a vector of floats containing the normal vectors
    gl::GenBuffers(1, &mut vboNormal);
    gl::BindBuffer(gl::ARRAY_BUFFER, vboNormal);
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(&normals), pointer_to_array(&normals), gl::STATIC_DRAW);
    gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, 0, ptr::null()); //3 because -> x, y and z coordinates
    gl::EnableVertexAttribArray(3);
 
    //return VAO ID
    vertexArrayID
}


// Create it to it to determine what to draw instead of just calling the draw function for each VAO manually
unsafe fn draw_scene(node: &scene_graph::SceneNode, view_projection_matrix: &glm::Mat4, transformation_so_far: &glm::Mat4) {
    
// Perform any logic needed before drawing the node
    let mut transformation_matrix: glm::Mat4 = glm::identity();
    transformation_matrix = glm::translation(&-node.reference_point) * transformation_matrix; // translation to reference point
    transformation_matrix = glm::rotation(node.rotation.x, &glm::vec3(1.0, 0.0, 0.0)) * transformation_matrix; // rotation on x
    transformation_matrix = glm::rotation(node.rotation.y, &glm::vec3(0.0, 1.0, 0.0)) * transformation_matrix; // rotation on y
    transformation_matrix = glm::rotation(node.rotation.z, &glm::vec3(0.0, 0.0, 1.0)) * transformation_matrix; // rotation on z
    transformation_matrix = glm::translation(&node.position) * transformation_matrix; // translation
    transformation_matrix = glm::translation(&node.reference_point) * transformation_matrix; // translation to origin

    transformation_matrix = transformation_so_far * transformation_matrix; // multiplying with transformation so far


    // Check if node is drawable, if so: set uniforms, bind VAO and draw VAO
    if node.index_count > 0 {
        let uniform_matrix = view_projection_matrix * transformation_matrix;
        gl::BindVertexArray(node.vao_id);        
        gl::UniformMatrix4fv(2, 1, gl::FALSE, (uniform_matrix).as_ptr()); // Model matrix to layout 1
        gl::UniformMatrix4fv(4, 1, gl::FALSE, (transformation_matrix).as_ptr()); // Model View Projection matrix to layout 2

        gl::DrawElements(gl::TRIANGLES, node.index_count, gl::UNSIGNED_INT, ptr::null());
    }
    // Recurse
    for &child in &node.children {
        draw_scene(&*child, view_projection_matrix, &transformation_matrix);
    }
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
        //create vao using vertices & indices
        // let my_vao = unsafe { create_vao(&vertices, &indices, &rgba) };

        //load terrain surface
        let terrain_mesh = mesh::Terrain::load("./resources/lunarsurface.obj");

	    //create vao for the terrain
        let terrain_vao = unsafe { 
            create_vao(
                &terrain_mesh.vertices,
                &terrain_mesh.indices,
                &terrain_mesh.colors,
                &terrain_mesh.normals
            ) 
        };

        //create the helicopter object
        let helicopter_mesh = mesh::Helicopter::load("./resources/helicopter.obj");

        //create a VAO for each part of the helicopter
        //create vao for the body
        let body_vao = unsafe { 
            create_vao(
                &helicopter_mesh.body.vertices,
                &helicopter_mesh.body.indices,
                &helicopter_mesh.body.colors,
                &helicopter_mesh.body.normals
            ) 
        };

        //create vao for the door
        let door_vao = unsafe { 
            create_vao(
                &helicopter_mesh.door.vertices,
                &helicopter_mesh.door.indices,
                &helicopter_mesh.door.colors,
                &helicopter_mesh.door.normals
            ) 
        };

        //create vao for the main rotor
        let main_rotor_vao = unsafe { 
            create_vao(
                &helicopter_mesh.main_rotor.vertices,
                &helicopter_mesh.main_rotor.indices,
                &helicopter_mesh.main_rotor.colors,
                &helicopter_mesh.main_rotor.normals
            ) 
        };

        //create vao for the tail rotor 
        let tail_rotor_vao = unsafe { 
            create_vao(
                &helicopter_mesh.tail_rotor.vertices,
                &helicopter_mesh.tail_rotor.indices,
                &helicopter_mesh.tail_rotor.colors,
                &helicopter_mesh.tail_rotor.normals
            ) 
        };

	    //create the root of the scene
	    let mut root_scene= SceneNode::new();

        //create the terrain of the scene
        let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_mesh.index_count); 

        //create a vector that has the helicopters in it
        let mut helicopter_5nodes: Vec<scene_graph::Node> = Vec::new();
        for n in 0..5 {
            //Step 1: Generate one SceneNode for each object
            let mut helicopter_body_node = SceneNode::from_vao(body_vao, helicopter_mesh.body.index_count);
            let mut helicopter_door_node = SceneNode::from_vao(door_vao, helicopter_mesh.door.index_count);
            let mut helicopter_main_rotor_node = SceneNode::from_vao(main_rotor_vao, helicopter_mesh.main_rotor.index_count);
            let mut helicopter_tail_rotor_node = SceneNode::from_vao(tail_rotor_vao, helicopter_mesh.tail_rotor.index_count);

            //Step 3: Initialise the values in the SceneNode data structure to their respective initial values, such as the position and starting rotations.
            //helicopter_body_node.position = glm::vec3(10.0, 0.0, 0.0);;
            helicopter_tail_rotor_node.reference_point = glm::Vec3::new(0.35, 2.3, 10.4); // Give to tail rotor node a reference point 
	
   	        //Step 2: Organise the objects into a Scene Graph by adding child nodes to their parentss list of children
	        terrain_node.add_child(&helicopter_body_node);

	        //the helicopter is relative to the terrain and it is in obvious
            //that the door, main rotor and tail rotor are relative to the body
	        helicopter_body_node.add_child(&helicopter_door_node);
            helicopter_body_node.add_child(&helicopter_main_rotor_node);
            helicopter_body_node.add_child(&helicopter_tail_rotor_node);

            //Finally pushing the helicopter to the vector
            helicopter_5nodes.push(helicopter_body_node)
        }

	    //Step 1: Generate one SceneNode for each object
        //let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_mesh.index_count); 
        //let mut helicopter_body_node = SceneNode::from_vao(body_vao, helicopter_mesh.body.index_count);
        //let mut helicopter_door_node = SceneNode::from_vao(door_vao, helicopter_mesh.door.index_count);
        //let mut helicopter_main_rotor_node = SceneNode::from_vao(main_rotor_vao, helicopter_mesh.main_rotor.index_count);
        //let mut helicopter_tail_rotor_node = SceneNode::from_vao(tail_rotor_vao, helicopter_mesh.tail_rotor.index_count);
	
        //Step 3: Initialise the values in the SceneNode data structure to their respective initial values, such as the position and starting rotations.
        //helicopter_body_node.position = glm::vec3(10.0, 0.0, 0.0);;
        //helicopter_tail_rotor_node.reference_point = glm::Vec3::new(0.35, 2.3, 10.4); // Give to tail rotor node a reference point 
	
	    //Step 2: Organise the objects into a Scene Graph by adding child nodes to their parentss list of children
	    //terrain_node.add_child(&helicopter_body_node);

	    //the helicopter is relative to the terrain and it is in obvious
        //that the door, main rotor and tail rotor are relative to the body
	    //helicopter_body_node.add_child(&helicopter_door_node);
        //helicopter_body_node.add_child(&helicopter_main_rotor_node);
        //helicopter_body_node.add_child(&helicopter_tail_rotor_node);

        //Step4: Connect the terrain to a single root node for the entire scene
        root_scene.add_child(&terrain_node);



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
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
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

            //(Step 3) Set values for helicopter body and rotors movement/rotations
            //let heading = toolbox::simple_heading_animation(elapsed);
            //helicopter_body_node.position.x = heading.x;
            //helicopter_body_node.position.z = heading.z;
            //helicopter_body_node.rotation.x = heading.pitch;
            //helicopter_body_node.rotation.y = heading.yaw;
            //helicopter_body_node.rotation.z = heading.roll;
            //helicopter_tail_rotor_node.rotation.x = 2.0 * elapsed;
            //helicopter_main_rotor_node.rotation.y = 2.0 * elapsed;

            //(Step 3) Set values for 5 helicopter bodies and rotors movement/rotations
            for n in 0..5 {
                let heading = toolbox::simple_heading_animation(elapsed + (n as f32 * 0.95));
                helicopter_5nodes[n].position.x = heading.x;
                helicopter_5nodes[n].position.z = heading.z;
                helicopter_5nodes[n].rotation.x = heading.pitch;
                helicopter_5nodes[n].rotation.y = heading.yaw;
                helicopter_5nodes[n].rotation.z = heading.roll;
                helicopter_5nodes[n].get_child(1).rotation.y = 2.0 * elapsed; //main rotor
                helicopter_5nodes[n].get_child(2).rotation.x = 2.0 * elapsed; //tail rotor
            }

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
                            pos[0] += 10.0*delta_time;
                        }
                        VirtualKeyCode::D => {
                            pos[0] -= 10.0*delta_time;
                        }
                        VirtualKeyCode::Space => {
                            pos[1] -= 10.0*delta_time;
                        }
                        VirtualKeyCode::LShift => {
                            pos[1] += 10.0*delta_time;
                        }
                        VirtualKeyCode::W => {
                            pos[2] += 20.0*delta_time;
                        }
                        VirtualKeyCode::S => {
                            pos[2] -= 20.0*delta_time;
                        }
                        VirtualKeyCode::Right => {
                            rot[1] += 2.0*delta_time;
                        }
                        VirtualKeyCode::Left => {
                            rot[1] -= 2.0*delta_time;
                        }
                        VirtualKeyCode::Down => {
                            rot[0] += 2.0*delta_time;
                        }
                        VirtualKeyCode::Up => {
                            rot[0] -= 2.0*delta_time;
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

           transf_matrix *= glm::perspective(1.0, PI/2.0, 1.0, 1000.0); //flips the z-axis
           //to ensure drawing isnt out of view:
           transf_matrix *= glm::translation(&glm::vec3(0.0, 0.0, -1.5)) * glm::translation(&pos);
           //mimic behavior of camera- wasd, lrup
           transf_matrix *= glm::rotation(rot[0], &glm::vec3(1.0, 0.0, 0.0)) * glm::rotation(rot[1], &glm::vec3(0.0, 1.0, 0.0));

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                //creating transf matrix using uniform matrix property gl, p.33
                //using location 2 from simple.vert
                gl::UniformMatrix4fv(2, 1, gl::FALSE, transf_matrix.as_ptr()); //need to use as ptr as recommended in assignment

                // == // Issue the necessary gl:: commands to draw your scene here
                //first step of drawing a VAO is to bind it
                //gl::BindVertexArray(terrain_vao);
                //gl::DrawElements(gl::TRIANGLES, terrain_mesh.index_count, gl::UNSIGNED_INT, ptr::null())
		        //helicopter_body_node.position = glm::vec3(10.0, 0.0, 0.0);

                
		        let mut transformation: glm::Mat4 = glm::identity();
                draw_scene(&root_scene, &transf_matrix, &transformation);
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

use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint};
use glfw::{Action, Context, Key};
use std::f32::consts::FRAC_PI_2;
use std::ffi::CString;
use std::io::Write;
use std::os::raw::c_void;
use std::sync::mpsc::Receiver;
use std::{io, mem, ptr};

const WINDOW_RESOLUTION: (u32, u32) = (800, 600);

const VERTEX_SHADER_SOURCE: &str = include_str!("../shader.vert");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("../shader.frag");

fn main() {
    // Create a GLFW window and hook to OpenGL function pointers
    let (mut glfw, mut window, events) = init_and_create_glfw_window();

    // Compile shaders
    let shader_program = unsafe { compiler_shader() };
    // Populate a VAO with a triangle
    let vao = unsafe { setup_vertex_data() };

    let mut last_frame = glfw.get_time();

    // Render loop
    while !window.should_close() {
        // Handle events
        process_events(&mut window, &events);

        // Render
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Configure shader
            let time = glfw.get_time() as f32 * 5.0;
            let red_value = (time.cos() / 2.0) + 0.5;
            let green_value = (time.sin() / 2.0) + 0.5;
            let blue_value = (time.cos() / 2.0 + FRAC_PI_2) + 0.5;
            let vertex_color_location =
                gl::GetUniformLocation(shader_program, CString::new("albedo").unwrap().as_ptr());

            // Enable shader
            gl::UseProgram(shader_program);
            // Send data to shader
            gl::Uniform4f(
                vertex_color_location,
                red_value,
                green_value,
                blue_value,
                1.0,
            );

            gl::BindVertexArray(vao); // Not needed 'cause its the only VAO but that's how its supposed to work
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            // gl::BindVertexArray(0); // No need to unbind every time
        }

        // GLFW stuff
        window.swap_buffers();
        glfw.poll_events();

        print!("\rFPS: {}", 1.0 / (glfw.get_time() - last_frame));
        last_frame = glfw.get_time();
        io::stdout().flush().unwrap();
    }
}

fn init_and_create_glfw_window() -> (glfw::Glfw, glfw::Window, Receiver<(f64, glfw::WindowEvent)>) {
    // Initialize GLFW
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Configure OpenGL version to use
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    // Force i3 to show it as a floating window
    glfw.window_hint(glfw::WindowHint::Resizable(false));

    #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // Create the window
    let (mut window, events) = glfw
        .create_window(
            WINDOW_RESOLUTION.0,
            WINDOW_RESOLUTION.1,
            "OpenGL playground",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window !");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // Cap the window at 60 FPS
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    // Initialize OpenGL functions
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    (glfw, window, events)
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height)
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true);
            }
            _ => {}
        }
    }
}

unsafe fn compiler_shader() -> GLuint {
    // Build vertex shader
    let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
    let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
    gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
    compile_shader_with_debug(vertex_shader, "VERTEX");

    // Build fragment shader
    let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
    let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
    gl::ShaderSource(frag_shader, 1, &c_str_frag.as_ptr(), ptr::null());
    compile_shader_with_debug(frag_shader, "FRAGMENT");

    // Link shaders
    let shader_program = gl::CreateProgram();
    gl::AttachShader(shader_program, vertex_shader);
    gl::AttachShader(shader_program, frag_shader);
    gl::LinkProgram(shader_program);

    // Check for linking errors
    check_linking_errors(shader_program);

    // Cleanup
    gl::DeleteShader(vertex_shader);
    gl::DeleteShader(frag_shader);

    shader_program
}

unsafe fn compile_shader_with_debug(shader_id: GLuint, message: &str) {
    // Compile
    gl::CompileShader(shader_id);

    // Check for compilation errors
    let mut success = gl::FALSE as GLint;
    let mut info_log = Vec::with_capacity(512);
    info_log.set_len(512 - 1); // Skip the trailing null character

    gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        gl::GetShaderInfoLog(
            shader_id,
            512,
            ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        panic!(
            "ERROR::SHADER::{}::COMPILATION_FAILED\n{}",
            message,
            std::str::from_utf8(&info_log).unwrap()
        );
    }
}

unsafe fn check_linking_errors(shader_program: GLuint) {
    let mut success = gl::FALSE as GLint;
    let mut info_log = Vec::with_capacity(512);
    info_log.set_len(512 - 1); // Skip the trailing null character

    gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        gl::GetProgramInfoLog(
            shader_program,
            512,
            ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        panic!(
            "ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}",
            std::str::from_utf8(&info_log).unwrap()
        );
    }
}

unsafe fn setup_vertex_data() -> GLuint {
    let vertices: [f32; 9] = [
        -0.5, -0.5, 0.0, // bottom left
        0.5, -0.5, 0.0, // bottom right
        0.0, 0.5, 0.0, // up
    ];

    // Create a VAO and its VBO
    // VBO: Vertex Buffer Objects
    // VAO: Vertex Array Object
    let (vbo, vao) = {
        let (mut vbo, mut vao) = (0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        (vbo, vao)
    };

    // Bind the VAO first, then bind the VBO and configure them
    gl::BindVertexArray(vao);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    // Send vertices to the GPU
    gl::BufferData(
        gl::ARRAY_BUFFER,
        (vertices.len() * mem::size_of_val(&vertices[0])) as GLsizeiptr,
        &vertices[0] as *const f32 as *const c_void,
        gl::STATIC_DRAW,
    );

    // Describe an attribute of the vertex array
    gl::VertexAttribPointer(
        0, // Attribute 0
        3, // with 3 values
        gl::FLOAT, // of type float
        gl::FALSE, // not normalized
        3 * mem::size_of::<GLfloat>() as GLsizei, // stride: how many byte between vertices
        ptr::null(), // offset to start at (in bytes)
    );
    // Enable attribute 0, will be in the location instruction of the vertex shader
    gl::EnableVertexAttribArray(0);

    // VBO is associated with the VAO, we can safely unbind it
    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    // Unbind VAO to avoid accidental modification of it even though its kinda hard to mess it up
    gl::BindVertexArray(0);

    // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

    // The VBO is bound to the VAO so we only need to care of the VAO
    vao
}

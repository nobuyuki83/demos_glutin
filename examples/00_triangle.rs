use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

use del_gl::gl as gl;

fn main() {
    let vtx2xyzrgb: Vec<f32> = vec![
        -0.5, -0.5, 1.0, 0.0, 0.0,
        0.0, 0.5, 0.0, 1.0, 0.0,
        0.5, -0.5, 0.0, 0.0, 1.0];
    //
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("A fantastic window!");
    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    println!("Pixel format of the window's GL context: {:?}", windowed_context.get_pixel_format());

    let gl = gl::Gl::load_with(|ptr| windowed_context.context().get_proc_address(ptr) as *const _);
    let version = unsafe {
        let data = std::ffi::CStr::from_ptr(gl.GetString(gl::VERSION) as *const _).to_bytes().to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL version {}", version);

    let mut drawer = del_gl::array_vtxcolor::Drawer { program: 0, mode: gl::TRIANGLES };
    drawer.compile_shader(&gl);
    drawer.initialize(
        &gl,
        &vtx2xyzrgb);

    el.run(move |event, _, control_flow| {
        // println!("{:?}", event);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => (),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl.ClearColor(0.8, 0.8, 1.0, 1.0);
                    gl.Clear(gl::COLOR_BUFFER_BIT);
                }
                drawer.draw_frame(&gl);
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}

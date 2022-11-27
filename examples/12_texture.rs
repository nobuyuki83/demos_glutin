use del_gl::gl as gl;
use image::io::Reader as ImageReader;
use image::EncodableLayout;

fn main() {
    let img = ImageReader::open("asset/tesla.png").unwrap();
    println!("{:?}", img.format());
    let img = img.decode().unwrap().to_rgb8();
    let img = image::imageops::flip_vertical(&img);
    println!("{:?}", img.dimensions());
    //println!("{:?}",img.color());

    let (mut viewer, event_loop) = del_gl::glutin::viewer2::Viewer2::open();

    unsafe {
        del_gl::utility::gen_texture(&viewer.gl,
                                     img.width().try_into().unwrap(),
                                     img.height().try_into().unwrap(),
                                     img.as_bytes(), gl::RGB);
    }

    let mut drawer = del_gl::mesh_tex::Drawer::new();
    {
        let vtx2xy = vec!(-1., -1., 1., -1., 1., 1., -1., 1.);
        let tri2vtx = vec!(0, 1, 2, 0, 2, 3);
        let vtx2tex = vec!(0., 0., 1., 0., 1., 1., 0., 1.);
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx2xy, 2);
        drawer.set_texture_uv(&viewer.gl, &vtx2tex);
        drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx, None);
        {
            let line2vtx: Vec<usize> = del_msh::line2vtx::from_epecific_edges_of_uniform_mesh(
                &tri2vtx, 3,
                &[0, 1, 1, 2, 2, 0],
                vtx2xy.len() / 2);
            drawer.add_element(&viewer.gl, gl::LINES, &line2vtx, Some([0., 0., 0.]));
        }
    }

    // this clousure captures drawer, viewer and 'move' them. drawer and viewer cannot be usable anymore
    let event_handle_closure = move |event: glutin::event::Event<()>,
                                     _elwt: &glutin::event_loop::EventLoopWindowTarget<()>,
                                     control_flow: &mut glutin::event_loop::ControlFlow| {
        viewer.event_handle(&event);
        use glutin::event_loop::ControlFlow::{Wait, ExitWithCode};
        if viewer.should_close { *control_flow = ExitWithCode(0); } else { *control_flow = Wait; }
        //
        if viewer.should_draw {
            let mat_projection = viewer.nav.projection_matrix(viewer.ui_state.win_width, viewer.ui_state.win_height);
            let mat_modelview = viewer.nav.modelview_matrix();
            drawer.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}

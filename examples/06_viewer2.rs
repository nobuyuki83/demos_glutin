use del_gl::gl as gl;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer2::Viewer2::open();
    println!("OpenGL Version: {}", viewer.get_opengl_version());
    println!("Pixel format of the window's GL context: {:?}", viewer.windowed_context.get_pixel_format());

    let mut drawer = del_gl::mesh::Drawer::new();
    {
        let (mut vtx2xyz, quad2vtx) = del_msh::primitive::grid_quad2::<f32>(
            32,32);
        vtx2xyz = del_misc::nalgebra::msh_misc::centerize_normalize_boundingbox(vtx2xyz, 2);
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx2xyz, 2);
        {
            let tri2vtx = del_msh::tri2vtx::from_quad_mesh(&quad2vtx);
            drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx, [1., 0., 0.]);
        }
        {
            let line2vtx: Vec<usize> = del_msh::line2vtx::from_sepecific_edges_of_uniform_mesh(
                &quad2vtx, 4,
                &[0, 1, 1, 2, 2, 3, 3, 0],
                vtx2xyz.len() / 2);
            drawer.add_element(&viewer.gl, gl::LINES, &line2vtx, [0., 0., 0.]);
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

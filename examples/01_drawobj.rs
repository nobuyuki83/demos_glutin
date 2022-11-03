use del_gl::gl as gl;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let mut drawer = del_gl::mesh::Drawer::new();
    {
        let (tri2vtx, vtx2xyz) = del_msh::io_obj::load_tri_mesh(
            "asset/bunny_11k.obj", Some(1.5));
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx2xyz, 3);
        drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx, [1., 0., 0.]);
        {
            let line2vtx: Vec<usize> = del_msh::topology_uniform::mshline(
                &tri2vtx, 3,
                &[0, 1, 1, 2, 2, 0],
                vtx2xyz.len() / 3);
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
            let mat_projection = viewer.nav.projection_matrix(
                viewer.ui_state.win_width, viewer.ui_state.win_height);
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

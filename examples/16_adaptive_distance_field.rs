use del_gl::gl as gl;

fn main() {
    use del_misc::adaptive_distance_field::AdaptiveDistanceField;
    let mut adf = AdaptiveDistanceField::new();
    {
        let sdf = |x: [f64; 3]| (x[0] * x[0] + x[1] * x[1] + x[2] * x[2]).sqrt() - 0.5;
        adf.build(sdf, [0.; 3], 1.123);
    }

    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();
    println!("Pixel format of the window's GL context: {:?}", viewer.windowed_context.get_pixel_format());

    let mut drawer = del_gl::mesh::Drawer::new();
    {
        let (tri2vtx, vtx2xyz) = del_msh::primitive::sphere_tri3(
            0.5_f32, 32, 32);
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx2xyz, 3);
        drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx, [1., 0., 0.]);
        {
            let line2vtx: Vec<usize> = del_msh::line2vtx::from_sepecific_edges_of_uniform_mesh(
                &tri2vtx, 3,
                &[0, 1, 1, 2, 2, 0],
                vtx2xyz.len() / 3);
            drawer.add_element(&viewer.gl, gl::LINES, &line2vtx, [0., 0., 0.]);
        }
    }

    let edge2xyz = adf.edge_to_xyz();
    let mut drawer_edge = del_gl::array::Drawer {
        program: 0,
        mode: gl::LINES,
        elem_size: edge2xyz.len(),
        vao: 0,
        loc_color: -1,
        loc_mat_projection: -1,
        loc_mat_modelview: -1,
        color: [0., 0., 1.],
    };
    drawer_edge.compile_shader(&viewer.gl);
    drawer_edge.initialize(&viewer.gl, &edge2xyz);

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
            drawer_edge.draw_frame(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}

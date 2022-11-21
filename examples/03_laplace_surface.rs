use del_gl::gl as gl;

fn main() {
    let (vtx2xyz, tri2vtx) = del_msh::primitive::cylinder_closed_end_tri3(
        0.2, 1.8, 16, 32);

    let mut ls = del_ls::linearsystem::Solver::new();
    {  // set pattern to sparse matrix
        let vtx2vtx = del_msh::vtx2vtx::from_uniform_mesh2(
            &tri2vtx, 3, vtx2xyz.len() / 3);
        ls.initialize(&vtx2vtx.0, &vtx2vtx.1);
    }

    // sparse.set_zero();
    ls.begin_mearge();
    del_misc::mesh_laplacian::merge_trimesh3(
        &mut ls.sparse,
        &mut ls.merge_buffer,
        &tri2vtx, &vtx2xyz);

    {
        let penalty = 1.0e+3;
        for iv in 0..vtx2xyz.len() / 3 {
            let y = vtx2xyz[iv * 3 + 1];
            if y < -0.89 {
                ls.r_vec[iv] = penalty * 0.;
                ls.sparse.row2val[iv] += penalty;
            } else if y > 0.89 {
                ls.r_vec[iv] = penalty * 1.;
                ls.sparse.row2val[iv] += penalty;
            }
        }
    }

    ls.solve_cg();
    let vtx_val = ls.u_vec.clone();

    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();
    let mut drawer = del_gl::mesh_colormap::Drawer::new();
    {
        drawer.color_map = vec![
            [0.0, 0.0, 0.0], // 0
            [0.5, 0.0, 0.0], // 0.2
            [1.0, 0.0, 0.0], // 0.4
            [1.0, 0.5, 0.0], // 0.6
            [1.0, 1.0, 0.0], // 0.8
            [1.0, 1.0, 1.0]]; // 1.0
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx2xyz, 3);
        drawer.update_value(&viewer.gl, &vtx_val);
        drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx);
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

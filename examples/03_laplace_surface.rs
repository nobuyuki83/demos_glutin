use del_gl::gl as gl;

fn main() {
    let (vtx2xyz, tri2vtx) = del_msh::primitive::cylinder_closed_end_tri3(
        0.2, 1.8, 16, 32);

    let mut ls = del_ls::linearsystem::Solver::new();
    {  // set pattern to sparse matrix
        let vtx2tri = del_msh::topology_uniform::elsup(
            &tri2vtx, 3, vtx2xyz.len() / 3);
        let vtx2vtx = del_msh::topology_uniform::psup(
            &tri2vtx,
            &vtx2tri.0, &vtx2tri.1,
            3,
            vtx2xyz.len() / 3);
        ls.initialize(&vtx2vtx.0, &vtx2vtx.1);
    }

    // sparse.set_zero();
    ls.begin_mearge();
    for it in 0..tri2vtx.len() / 3 {
        let i0 = tri2vtx[it * 3 + 0];
        let i1 = tri2vtx[it * 3 + 1];
        let i2 = tri2vtx[it * 3 + 2];
        let cots = del_geo::tri::cot3(
            &vtx2xyz[(i0 * 3 + 0)..(i0 * 3 + 3)],
            &vtx2xyz[(i1 * 3 + 0)..(i1 * 3 + 3)],
            &vtx2xyz[(i2 * 3 + 0)..(i2 * 3 + 3)]);
        let emat: [f32; 9] = [
            cots[1] + cots[2], -cots[2], -cots[1],
            -cots[2], cots[2] + cots[0], -cots[0],
            -cots[1], -cots[0], cots[0] + cots[1]];
        ls.sparse.merge(
            &[i0, i1, i2], &[i0, i1, i2], &emat,
            &mut ls.merge_buffer);
    }

    {
        let penalty = 1.0e+3;
        for iv in 0..vtx2xyz.len() / 3 {
            let y = vtx2xyz[iv * 3 + 1];
            if y < -0.89 {
                ls.r_vec[iv] = penalty * 0.;
                ls.sparse.val_dia[iv] += penalty;
            } else if y > 0.89 {
                ls.r_vec[iv] = penalty * 1.;
                ls.sparse.val_dia[iv] += penalty;
            }
        }
    }

    ls.solve_cg();
    let vtx_val = ls.u_vec.clone();

    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();
    let mut drawer = del_gl::drawer_meshposcolor::DrawerMeshPosColor::new();
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
